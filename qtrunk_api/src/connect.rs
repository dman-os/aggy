use crate::interlude::*;

use axum::extract::{
    ws::{CloseFrame, Message as WsMsg, WebSocket, WebSocketUpgrade},
    ConnectInfo, State,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use tokio::{sync::mpsc::Sender, sync::RwLock};

use crate::event::{Event, Filter};

#[derive(Debug)]
pub struct Subscription {
    id: CHeapStr,
    filters: Vec<Filter>,
}

#[derive(Debug)]
pub struct Client {
    id: Uuid,
    connected_at: OffsetDateTime,
    addr: std::net::SocketAddr,
    subs: RwLock<Vec<Subscription>>,
    tx: Sender<Value>,
}

#[derive(Default, educe::Educe)]
#[educe(Debug)]
pub struct Switchboard {
    #[educe(Debug(ignore))]
    clients: RwLock<HashMap<Uuid, Client>>,
}

pub async fn pub_event(cx: &Context, event: &Event) -> eyre::Result<()> {
    use redis::AsyncCommands;
    let mut conn = cx.redis.get().await?;
    conn.publish(cx.config.event_hose_redis_channel.as_str(), event)
        .await?;
    Ok(())
}

pub async fn start_switchboard(cx: SharedContext) -> eyre::Result<()> {
    /* // even though we recieve all events through a redis pub/sub (to allow horizontal scaling)
    // we buffer the redis events in a tokio mpsc for...idk, pre-optimzation reasons
    let (sw_tx, mut sw_rx) = tokio::sync::mpsc::unbounded_channel::<Event>(); */

    // subscribe to the redis channel while on the main "task"
    let mut conn = cx.redis.dedicated_connection().await?.into_pubsub();
    conn.subscribe(cx.config.event_hose_redis_channel.as_str())
        .await?;

    /* // span a separate task for the polling though
    let redis_loop_handle = tokio::spawn(async move {
        let mut stream = conn.into_on_message();
        while let Some(msg) = stream.next().await {
            let event: Event = msg.get_payload().unwrap_or_log();
            sw_tx.send(event).unwrap_or_log()
        }
    }); */

    // while let Some(event) = sw_rx.recv().await {
    let mut stream = conn.into_on_message();
    while let Some(msg) = stream.next().await {
        let event: Event = msg.get_payload().unwrap_or_log();
        info!(?event, "event recieved for switiching");
        // FIXME: stress test this
        let clients = cx.sw.clients.read().await;
        clients
            .values()
            // creat a future for each client
            .map(|client| async {
                let subs = client.subs.read().await;
                for sub in &*subs {
                    if let Some(filter) = sub.filters.iter().find(|filter| filter.matches(&event)) {
                        let res = client.tx.send(json!(["EVENT", *sub.id, event])).await;
                        // client must have disconnected
                        if res.is_err() {
                            break;
                        }
                        info!(?event.id, ?client.id, ?filter, "event sent to client according to filter");
                    }
                }
            })
            // mass poll them concurrently
            .collect::<futures::stream::futures_unordered::FuturesUnordered<_>>()
            .for_each_concurrent(None, |_| async {})
            .await
    }
    // redis_loop_handle.abort();
    Ok(())
}

pub async fn handler(
    State(cx): State<SharedContext>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // tokio::sync::RwLock<>
    ws.on_upgrade(move |socket| handle_client(cx, socket, addr))
}

async fn handle_client(cx: SharedContext, socket: WebSocket, addr: std::net::SocketAddr) {
    // the web socket pipes
    let (mut ws_tx, mut ws_rx) = socket.split();
    // the switchboard pipes
    let (sw_tx, mut sw_rx) = tokio::sync::mpsc::channel::<Value>(32);
    let (close_tx, mut close_rx) = tokio::sync::mpsc::channel::<(u16, String)>(1);
    let id = Uuid::new_v4();
    {
        let mut clients = cx.sw.clients.write().await;
        clients.insert(
            id,
            Client {
                id,
                addr,
                connected_at: OffsetDateTime::now_utc(),
                subs: default(),
                tx: sw_tx.clone(),
            },
        );
    }
    let mut tx_task = tokio::spawn(async move {
        'sel: loop {
            tokio::select! {
                biased; // handle close messages first since they're originate from rx errors
                Some((code, reason)) = close_rx.recv() => {
                    ws_tx.send(WsMsg::Close(Some(
                            CloseFrame{ code, reason:reason.into() }
                        )))
                        .await.unwrap_or_log();
                    break 'sel;
                },
                msg = sw_rx.recv() => {
                    let Some(msg) = msg else {
                        break 'sel;
                    };
                    ws_tx
                        // TODO: consider using feed here
                        .send(WsMsg::Text(serde_json::to_string(&msg).unwrap_or_log()))
                        .await
                        .unwrap_or_log()
                }
            }
        }
    });
    let mut rx_task = {
        let cx2 = cx.clone();
        tokio::spawn(async move {
            let cx = cx2;
            while let Some(Ok(msg)) = ws_rx.next().await {
                let mut msg: Vec<Value> = match msg {
                    WsMsg::Text(str) => serde_json::from_str(&str)
                        .map_err(|err| eyre::eyre!("unexpected msg recieved: {err} | {str}"))?,
                    WsMsg::Binary(buf) => serde_json::from_slice(&buf[..])
                        .map_err(|err| eyre::eyre!("unexpected msg recieved: {err}"))?,
                    // end the loop on close
                    WsMsg::Close(_) => break,
                    _ => continue,
                };
                // process the msg as per NIP-01
                let Some(kind) = msg[0].as_str() else {
                    return Err(eyre::eyre!("invalid msg recieved: {msg:?}"));
                };
                match kind {
                    "EVENT" if msg.len() == 2 => {
                        let event = serde_json::from_value(msg.pop().unwrap()).map_err(|err| {
                            eyre::eyre!(
                                "unexpected msg recieved: invalid EVENT msg {msg:?} | {err}"
                            )
                        })?;
                        let res = crate::event::create::CreateEvent.handle(&cx, event).await;
                        let res = match res {
                            Ok(ok) => ok.to_nostr_ok(),
                            Err(err) => err.to_nostr_ok(),
                        };
                        sw_tx.send(res).await.unwrap_or_log();
                    }
                    "REQ" if msg.len() >= 3 => {
                        let sub_id = msg[1].as_str().ok_or_else(|| {
                            eyre::eyre!("invalid REQ msg: invalid subscription id on {msg:?}")
                        })?;
                        let filters = msg[2..]
                            .iter()
                            .map(|val| {
                                serde_json::from_value(val.clone()).map_err(|err| {
                                    eyre::eyre!(
                                        "invalid REQ msg: invalid filter on {msg:?} | {err}"
                                    )
                                })
                            })
                            .collect::<Result<Vec<Filter>, _>>()?;
                        let events = crate::event::list::ListEvents
                            .handle(&cx, filters.clone())
                            .await
                            .map_err(|err| match err {
                                crate::event::list::Error::InvalidInput { issues } => {
                                    eyre::eyre!("error during initial list for REQ: {issues}")
                                }
                                err => Err(err).unwrap_or_log(),
                            })?;
                        for event in events {
                            sw_tx
                                .send(json!(["EVENT", sub_id, event]))
                                .await
                                .unwrap_or_log();
                        }
                        sw_tx.send(json!(["EOSE", sub_id])).await.unwrap_or_log();

                        let clients = cx.sw.clients.read().await;
                        let client = clients.get(&id).expect_or_log("client not found under id");
                        let mut subs = client.subs.write().await;

                        let sub_id = CHeapStr::new(sub_id.to_string());
                        if let Some(sub) = subs.iter_mut().find(|sub| sub.id == sub_id) {
                            sub.filters = filters;
                        } else {
                            subs.push(Subscription {
                                id: sub_id,
                                filters,
                            });
                        }
                    }
                    // FIXME: test this
                    "CLOSE" if msg.len() == 2 => {
                        let sub_id = msg[1].as_str().ok_or_else(|| {
                            eyre::eyre!("invalid CLOSE msg: invalid subscription id on {msg:?}")
                        })?;
                        let clients = cx.sw.clients.read().await;
                        let client = clients.get(&id).expect_or_log("client not found under id");
                        let mut subs = client.subs.write().await;

                        let mut found = false;
                        for (ii, sub) in subs.iter().enumerate() {
                            if *sub.id == sub_id {
                                subs.swap_remove(ii);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            sw_tx
                                .send(json!([
                                    "NOTICE",
                                    format!("no subscription found to close under id {sub_id}")
                                ]))
                                .await
                                .unwrap_or_log();
                        }
                    }
                    _ => return Err(eyre::eyre!("invalid msg recieved: {msg:?}")),
                }
            }
            Ok::<_, eyre::Report>(())
        })
    };
    tokio::select! {
        res = (&mut tx_task) => {
            match res {
                Ok(()) => {},
                Err(err) => debug!(?err, ?id, ?addr, "error in client send loop"),
            }
            rx_task.abort();
        }
        res = (&mut rx_task) => {
            match res {
                Ok(Ok(())) => {},
                Ok(Err(err)) => {
                    _ = close_tx.send((400, format!(": {err}"))).await;
                }
                Err(err) => {
                    _ = close_tx.send((500, format!("server error: {err}"))).await;
                },
            }
            tx_task.abort();
        }
    }
    {
        let mut clients = cx.sw.clients.write().await;
        clients.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::testing::*;

    const TEST_PRIVKEY: &str = "95dfc6261ec6c66b3ec68e1b019cf6420e1d676c29c1241ec5dea551ed89e338";

    fn fixture_request_json() -> serde_json::Value {
        let content = "The stars are a burning sun";

        let prikey = TEST_PRIVKEY;
        let prikey = data_encoding::HEXLOWER.decode(prikey.as_bytes()).unwrap();
        let prikey = k256::schnorr::SigningKey::from_bytes(&prikey[..]).unwrap();

        let pubkey = prikey.verifying_key().to_bytes();
        let pubkey = data_encoding::HEXLOWER.encode(&pubkey[..]);

        // let created_at = OffsetDateTime::from_unix_timestamp(1_690_962_268).unwrap();
        let created_at = OffsetDateTime::now_utc();

        let tags = vec![
            vec!["author".to_string(), "bridget".to_string()],
            vec!["e".to_string(), EVENT_01_ID.to_string()],
        ];

        let kind = 1;
        let (id, sig) = crate::event::hex_id_and_sig_for_event(
            &prikey,
            &pubkey[..],
            created_at,
            kind,
            &tags,
            content,
        );
        serde_json::json!({
            "id": id,
            "pubkey": pubkey,
            "created_at": created_at.unix_timestamp(),
            "kind": kind,
            "tags": tags,
            "content": content,
            "sig": sig,
        })
    }

    #[test]
    fn suite() {
        common::utils::testing::setup_tracing_once();
        let future = async {
            use tokio_tungstenite::tungstenite::Message as WsMsg;
            let (testing, cx) = crate::utils::testing::cx_fn(common::function_full!()).await;
            {
                let addr = "127.0.0.1:19000";
                let router = crate::router(cx);
                let server_handle =
                    tokio::spawn(axum::Server::bind(&addr.parse().unwrap()).serve(
                        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
                    ));
                let (mut ws_stream, response) =
                    tokio_tungstenite::connect_async("ws://127.0.0.1:19000")
                        .await
                        .map_err(|err| {
                            match &err {
                                tokio_tungstenite::tungstenite::Error::Http(err) => {
                                    if let Some(body) = err.body() {
                                        let body = String::from_utf8(body.clone());
                                        error!(?body, ?err);
                                    }
                                }
                                _ => {}
                            };
                            err
                        })?;
                info!(?response);
                let sub_id = Uuid::new_v4().to_string();
                // test REQ
                {
                    ws_stream
                        .send(WsMsg::Binary(serde_json::to_vec(&json!([
                            "REQ",
                            sub_id,
                            {}
                        ]))?))
                        .await?;
                    let mut event_ctr = 0;
                    while let Some(Ok(msg)) = ws_stream.next().await {
                        let resp = match msg {
                            WsMsg::Text(val) => val,
                            WsMsg::Pong(_) | WsMsg::Ping(_) | WsMsg::Close(_) => continue,
                            msg => panic!("unexpected message {msg}"),
                        };
                        let resp: Vec<Value> = serde_json::from_str(&resp[..])?;
                        let kind = resp[0].as_str().unwrap();
                        match kind {
                            "EVENT" => event_ctr += 1,
                            "EOSE" => {
                                check_json(
                                    ("expected", &json!(["EOSE", sub_id])),
                                    ("response", &Value::Array(resp)),
                                );
                                assert_eq!(event_ctr, 5);
                                break;
                            }
                            _ => panic!("unexpected event kind {kind}: {resp:?}"),
                        }
                    }
                }
                // test EVENT
                {
                    let event = fixture_request_json();
                    ws_stream
                        .send(WsMsg::Binary(serde_json::to_vec(&json!(["EVENT", event]))?))
                        .await?;
                    while let Some(Ok(msg)) = ws_stream.next().await {
                        let resp = match msg {
                            WsMsg::Text(val) => val,
                            WsMsg::Pong(_) | WsMsg::Ping(_) | WsMsg::Close(_) => continue,
                            msg => panic!("unexpected message {msg}"),
                        };
                        let resp: Vec<Value> = serde_json::from_str(&resp[..])?;
                        let kind = resp[0].as_str().unwrap();
                        match kind {
                            "EVENT" => {
                                check_json(
                                    ("expected", &json!(["EVENT", sub_id, event])),
                                    ("response", &Value::Array(resp)),
                                );
                                break;
                            }
                            "OK" => {
                                check_json(
                                    ("expected", &json!(["OK", event["id"], true])),
                                    ("response", &Value::Array(resp)),
                                );
                            }
                            _ => panic!("unexpected event kind {kind}: {resp:?}"),
                        }
                    }
                }
                server_handle.abort();
                // let (mut ws_tx, mut ws_rx) = ws_stream.split();
            }
            testing.close().await;
            Ok::<_, eyre::Report>(())
        };
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { tokio::time::timeout(std::time::Duration::new(30, 0), future).await })
            .unwrap_or_log()
            .unwrap_or_log();
    }
}
