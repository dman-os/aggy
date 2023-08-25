use deps::*;

shadow_rs::shadow!(build);

mod playground;

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
            let epigram_cx = {
                use epigram_api::*;
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
                    service_secret: common::utils::get_env_var("SERVICE_SECRET").unwrap_or_log(),
                };
                let db_url = common::utils::get_env_var("EPIGRAM_DATABASE_URL").unwrap_or_log();
                let db_pool = sqlx::PgPool::connect(&db_url).await.unwrap_or_log();
                let cx = Context {
                    db: Db::Pg { db_pool },
                    config,
                };
                std::sync::Arc::new(cx)
            };
            let app = axum::Router::new()
                .route(
                    "/up",
                    axum::routing::get(|| async {
                        axum::Json(serde_json::json! ({
                            "buildTime": build::BUILD_TIME,
                            "pkgVersion": build::PKG_VERSION,
                            "commitDate": build::COMMIT_DATE,
                            "commitHash": build::COMMIT_HASH,
                            "rustVersion": build::RUST_VERSION,
                            "rustChannel": build::RUST_CHANNEL,
                            "cargoVersion": build::CARGO_VERSION,
                        }))
                    }),
                )
                .nest("/aggy", {
                    use aggy_api::*;
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
                        service_secret: common::utils::get_env_var("SERVICE_SECRET")
                            .unwrap_or_log(),
                    };
                    let db_url = common::utils::get_env_var("AGGY_DATABASE_URL").unwrap_or_log();
                    let db_pool = sqlx::PgPool::connect(&db_url).await.unwrap_or_log();
                    let cx = Context {
                        db: Db::Pg { db_pool },
                        config,
                        epigram: Box::new(epigram_api::InProcClient {
                            cx: epigram_cx.clone(),
                        }),
                    };
                    let cx = std::sync::Arc::new(cx);
                    axum::Router::new().merge(aggy_api::router(cx))
                })
                .nest("/epigram", {
                    axum::Router::new().merge(epigram_api::router(epigram_cx))
                })
                .merge(
                    utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                        .url(
                            "/aggy/openapi.json",
                            <aggy_api::ApiDoc as utoipa::OpenApi>::openapi(),
                        )
                        .url(
                            "/epigram/openapi.json",
                            <epigram_api::ApiDoc as utoipa::OpenApi>::openapi(),
                        ),
                )
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
                        )
                        .make_span_with(
                            tower_http::trace::DefaultMakeSpan::new().include_headers(true),
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
