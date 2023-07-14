use deps::*;

use aggy_api::*;

shadow_rs::shadow!(build);

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
            let config = Config {
                pass_salt_hash: uuid::Uuid::new_v4().as_bytes().to_vec(),
                argon2_conf: argon2::Config::default(),
                auth_token_lifespan: time::Duration::new(
                    std::env::var("AUTH_TOKEN_LIFESPAN_SECS")
                        .unwrap_or_log()
                        .parse()
                        .unwrap_or_log(),
                    0,
                ),
            };
            let db_url = common::utils::get_env_var("AGGY_DATABASE_URL").unwrap_or_log();
            let db_pool = common::sqlx::PgPool::connect(&db_url).await.unwrap_or_log();
            let aggy_cx = Context { db_pool, config };
            let aggy_cx = std::sync::Arc::new(aggy_cx);
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
                .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/*tail").url(
                    "/api-doc/openapi.json",
                    <ApiDoc as utoipa::OpenApi>::openapi(),
                ))
                .nest("/aggy", aggy_api::router().with_state(aggy_cx))
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
