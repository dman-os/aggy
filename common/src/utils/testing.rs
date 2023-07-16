use deps::*;

pub use axum::http;
pub use axum::http::StatusCode;
pub use tower::ServiceExt;

pub fn setup_tracing() -> eyre::Result<()> {
    color_eyre::install()?;
    if std::env::var("RUST_LOG_TEST").is_err() {
        std::env::set_var("RUST_LOG_TEST", "info");
    }

    tracing_subscriber::fmt()
        // .pretty()
        .compact()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG_TEST"))
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .try_init()
        .map_err(|err| eyre::eyre!(err))?;

    Ok(())
}

// Ensure that the `tracing` stack is only initialised once using `once_cell`
// isn't required in cargo-nextest since each test runs in a new process
pub fn setup_tracing_once() {
    use once_cell::sync::Lazy;
    static TRACING: Lazy<()> = Lazy::new(|| {
        dotenvy::dotenv().ok();
        setup_tracing().unwrap();
    });
    Lazy::force(&TRACING);
}

pub struct ExtraAssertionAgs<'a> {
    pub test_cx: &'a mut TestContext,
    pub auth_token: Option<String>,
    pub response_head: axum::http::response::Parts,
    pub response_json: Option<serde_json::Value>,
}

pub type EAArgs<'a> = ExtraAssertionAgs<'a>;

/// BoxFuture type that's not send
pub type LocalBoxFuture<'a, T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + 'a>>;

pub type ExtraAssertions<'c, 'f> = dyn Fn(ExtraAssertionAgs<'c>) -> LocalBoxFuture<'f, ()>;

pub struct TestContext {
    pub test_name: String,
    pub db_pool: Option<sqlx::postgres::PgPool>,
    // clean_up_closure: Option<Box<dyn FnOnce(Context) -> ()>>,
    clean_up_closure:
        Option<Box<dyn FnOnce(sqlx::postgres::PgPool) -> futures::future::BoxFuture<'static, ()>>>,
}

/// NOTE: this is only good for tests and doesn't handle re-runnable migs well
#[derive(Debug)]
struct FlywayMigrationSource<'a>(&'a std::path::Path);

impl<'a> sqlx::migrate::MigrationSource<'a> for FlywayMigrationSource<'a> {
    fn resolve(
        self,
    ) -> futures::future::BoxFuture<
        'a,
        Result<Vec<sqlx::migrate::Migration>, sqlx::error::BoxDynError>,
    > {
        Box::pin(async move {
            struct WalkCx<'a> {
                migrations: &'a mut Vec<sqlx::migrate::Migration>,
                rerunnable_ctr: i64,
            }
            fn walk_dir<'a>(
                path: &'a std::path::Path,
                mut cx: &'a mut WalkCx,
            ) -> futures::future::BoxFuture<'a, Result<(), sqlx::error::BoxDynError>> {
                Box::pin(async move {
                    let mut s = tokio::fs::read_dir(path).await?;
                    while let Some(entry) = s.next_entry().await? {
                        // std::fs::metadata traverses symlinks
                        let metadata = std::fs::metadata(&entry.path())?;
                        if metadata.is_dir() {
                            walk_dir(&entry.path(), &mut cx).await?;
                            return Ok(());
                        }
                        if !metadata.is_file() {
                            // not a file; ignore
                            continue;
                        }

                        let file_name = entry.file_name().to_string_lossy().into_owned();

                        let parts = file_name.splitn(2, "__").collect::<Vec<_>>();

                        if parts.len() != 2
                            || !parts[1].ends_with(".sql")
                            || !(parts[0].starts_with('m') || parts[0].starts_with('r'))
                        {
                            // not of the format: <VERSION>_<DESCRIPTION>.sql; ignore
                            continue;
                        }

                        let version: i64 = if parts[0].starts_with('m') {
                            let Ok(v_parts) = parts[0][1..]
                                .split('.')
                                .into_iter()
                                .map(|str| str.parse())
                                .collect::<Result<Vec<i64>, _>>() else
                            {
                                 continue;
                            };
                            if v_parts.len() != 3 {
                                continue;
                            }
                            (v_parts[0] * 1_000_000) + (v_parts[1] * 1000) + v_parts[2]
                        } else {
                            // run rerunnable migrations last
                            cx.rerunnable_ctr += 1;
                            i64::MAX - cx.rerunnable_ctr
                        };

                        let migration_type = sqlx::migrate::MigrationType::from_filename(parts[1]);
                        // remove the `.sql` and replace `_` with ` `
                        let description = parts[1]
                            .trim_end_matches(migration_type.suffix())
                            .replace('_', " ")
                            .to_owned();

                        let sql = tokio::fs::read_to_string(&entry.path()).await?;

                        cx.migrations.push(sqlx::migrate::Migration::new(
                            version,
                            std::borrow::Cow::Owned(description),
                            migration_type,
                            std::borrow::Cow::Owned(sql),
                        ));
                    }

                    Ok(())
                })
            }
            let mut migrations = Vec::new();
            walk_dir(
                &self.0.canonicalize()?,
                &mut (WalkCx {
                    rerunnable_ctr: 0,
                    migrations: &mut migrations,
                }),
            )
            .await?;
            tracing::info!(?migrations, ?self, "migrations");
            // ensure that we are sorted by `VERSION ASC`
            migrations.sort_by_key(|m| m.version);

            Ok(migrations)
        })
    }
}

impl TestContext {
    pub async fn new(test_name: &'static str) -> Self {
        setup_tracing_once();
        let test_name = test_name.replace("::tests::", "").replace("::", "_");

        use sqlx::prelude::*;
        let opts = sqlx::postgres::PgConnectOptions::default()
            .host(
                std::env::var("TEST_DB_HOST")
                    .expect("TEST_DB_HOST wasn't found in enviroment")
                    .as_str(),
            )
            .port(
                std::env::var("TEST_DB_PORT")
                    .expect("TEST_DB_PORT wasn't found in enviroment")
                    .parse()
                    .expect("TEST_DB_PORT is not a valid number"),
            )
            .username(
                std::env::var("TEST_DB_USER")
                    .expect("TEST_DB_USER wasn't found in enviroment")
                    .as_str(),
            )
            .log_statements("DEBUG".parse().unwrap());

        let opts = if let Ok(pword) = std::env::var("TEST_DB_PASS") {
            opts.password(pword.as_str())
        } else {
            opts
        };

        let mut connection = opts
            .clone()
            .connect()
            .await
            .expect("Failed to connect to Postgres without db");

        connection
            .execute(&format!(r###"DROP DATABASE IF EXISTS {}"###, test_name)[..])
            .await
            .expect("Failed to drop old database.");

        connection
            .execute(&format!(r###"CREATE DATABASE {}"###, test_name)[..])
            .await
            .expect("Failed to create database.");

        let opts = opts.database(&test_name[..]);

        // migrate database
        let db_pool = sqlx::PgPool::connect_with(opts)
            .await
            .expect("Failed to connect to Postgres as test db.");

        // sqlx::migrate!("./migrations")
        sqlx::migrate::Migrator::new(FlywayMigrationSource(std::path::Path::new("./migrations")))
            .await
            .expect("error setting up migrator for ./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to migrate the database");
        // sqlx::migrate!("./fixtures")
        sqlx::migrate::Migrator::new(std::path::Path::new("./fixtures"))
            .await
            .expect("error setting up migrator for ./fixtures")
            .set_ignore_missing(true) // don't inspect migrations store
            .run(&db_pool)
            .await
            .expect("Failed to add test data");

        Self {
            test_name: test_name.clone(), // someone needs it downwind
            db_pool: Some(db_pool),
            clean_up_closure: Some(Box::new(move |db_pool| {
                Box::pin(async move {
                    db_pool.close().await;
                    connection
                        .execute(&format!(r###"DROP DATABASE {test_name}"###)[..])
                        .await
                        .expect("Failed to drop test database.");
                })
            })),
        }
    }

    /// Call this after all holders of the [`SharedContext`] have been dropped.
    pub async fn close(mut self) {
        let db_pool = self.db_pool.take().unwrap_or_log();
        (self.clean_up_closure.take().unwrap())(db_pool);
    }
    /* pub async fn new_with_service<F>(
        test_name: &'static str,
        cfg_callback: F,
    ) -> (
        Self,
        // TestServer,
        impl FnOnce(Self) -> futures::future::BoxFuture<'static, ()>,
    )
    where
        F:Fn(& Context) -> axum::Router + 'static + Send + Sync + Clone + Copy
    {
        let router = cfg_callback
        Self {
            test_name,
        }
    } */
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if self.db_pool.is_some() {
            tracing::warn!(
                "test context dropped without cleaning up for: {}",
                self.test_name
            )
        }
    }
}

/// Not deep equality but deep "`is_subset_of`" check.
pub fn check_json(
    (check_name, check): (&str, &serde_json::Value),
    (json_name, json): (&str, &serde_json::Value),
) {
    use serde_json::Value::*;
    match (check, json) {
        (Array(r_arr), Array(arr)) => {
            for ii in 0..arr.len() {
                check_json(
                    (&format!("{check_name}[{ii}]"), &r_arr[ii]),
                    (&format!("{json_name}[{ii}]"), &arr[ii]),
                );
            }
        }
        (Object(check), Object(response)) => {
            for (key, val) in check {
                check_json(
                    (&format!("{check_name}.{key}"), val),
                    (
                        &format!("{json_name}.{key}"),
                        response
                            .get(key)
                            .ok_or_else(|| {
                                format!("key {key} wasn't found on {json_name}: {response:?}")
                            })
                            .unwrap(),
                    ),
                );
            }
        }
        (check, json) => assert_eq!(check, json, "{check_name} != {json_name}"),
    }
}

pub trait JsonExt {
    fn remove_keys_from_obj(self, keys: &[&str]) -> Self;
    fn destructure_into_self(self, from: Self) -> Self;
}
impl JsonExt for serde_json::Value {
    fn remove_keys_from_obj(self, keys: &[&str]) -> Self {
        match self {
            serde_json::Value::Object(mut map) => {
                for key in keys {
                    map.remove(*key);
                }
                serde_json::Value::Object(map)
            }
            json => panic!("provided json was not an object: {:?}", json),
        }
    }
    fn destructure_into_self(self, from: Self) -> Self {
        match (self, from) {
            (serde_json::Value::Object(mut first), serde_json::Value::Object(second)) => {
                for (key, value) in second.into_iter() {
                    first.insert(key, value);
                }
                serde_json::Value::Object(first)
            }
            (first, second) => panic!(
                "provided jsons weren't objects: first {:?}, second: {:?}",
                first, second
            ),
        }
    }
}
