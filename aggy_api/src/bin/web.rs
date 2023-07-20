use deps::*;

use aggy_api::*;

fn main() {
    dotenvy::dotenv().ok();
    common::setup_tracing().unwrap();
    #[cfg(feature = "dylink")]
    tracing::warn!("dylink enabled");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_log()
        .block_on(async {
            let app = axum::Router::new()
                .merge({
                    let config = Config {
                        pass_salt_hash: uuid::Uuid::new_v4().as_bytes().to_vec(),
                        argon2_conf: argon2::Config::default(),
                        auth_token_lifespan: time::Duration::new(
                            common::utils::get_env_var("AUTH_TOKEN_LIFESPAN_SECS")
                                .unwrap_or_log()
                                .parse()
                                .unwrap_or_log(),
                            0,
                        ),
                        web_session_lifespan: time::Duration::new(
                            common::utils::get_env_var("WEB_SESSION_LIFESPAN_SECS")
                                .unwrap_or_log()
                                .parse()
                                .unwrap_or_log(),
                            0,
                        ),
                        service_secret: common::utils::get_env_var("SESSION_SECRET")
                            .unwrap_or_log(),
                    };
                    let db_url = common::utils::get_env_var("DATABASE_URL").unwrap_or_log();
                    let db_pool = sqlx::PgPool::connect(&db_url).await.unwrap_or_log();
                    let cx = Context {
                        db: Db::Pg { db_pool },
                        config,
                    };
                    let cx = std::sync::Arc::new(cx);
                    axum::Router::new()
                        .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui").url(
                            "/api-doc/openapi.json",
                            <ApiDoc as utoipa::OpenApi>::openapi(),
                        ))
                        .merge(aggy_api::router(cx))
                })
                .layer(
                    tower_http::trace::TraceLayer::new_for_http()
                        .on_response(
                            tower_http::trace::DefaultOnResponse::new()
                                .level(tracing::Level::INFO)
                                .latency_unit(tower_http::LatencyUnit::Micros),
                        )
                        .on_failure(
                            tower_http::trace::DefaultOnFailure::new()
                                .level(tracing::Level::ERROR)
                                .latency_unit(tower_http::LatencyUnit::Micros),
                        ),
                );

            let address = std::net::SocketAddr::from((
                std::net::Ipv4Addr::UNSPECIFIED,
                common::utils::get_env_var("PORT")
                    .and_then(|str| {
                        str.parse().map_err(|err| {
                            eyre::format_err!("error parsing port env var ({str}): {err}")
                        })
                    })
                    .unwrap_or(8080),
            ));
            tracing::info!("Server listening at {address:?}");
            axum::Server::bind(&address)
                .serve(app.into_make_service())
                .await
        })
        .unwrap_or_log()
}
