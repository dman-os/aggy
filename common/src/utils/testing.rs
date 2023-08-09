use std::collections::HashMap;

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

pub struct TestDb {
    pub db_name: String,
    pub pool: sqlx::postgres::PgPool,
    clean_up_closure: Option<Box<dyn FnOnce() -> futures::future::BoxFuture<'static, ()>>>,
}

impl TestDb {
    pub async fn new(db_name: String, migrations_root: &std::path::Path) -> Self {
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
            .execute(&format!(r###"DROP DATABASE IF EXISTS {db_name}"###)[..])
            .await
            .expect("Failed to drop old database.");

        connection
            .execute(&format!(r###"CREATE DATABASE {db_name}"###)[..])
            .await
            .expect("Failed to create database.");

        let opts = opts.database(&db_name[..]);

        // migrate database
        let pool = sqlx::PgPool::connect_with(opts)
            .await
            .expect("Failed to connect to Postgres as test db.");

        // sqlx::migrate!("./migrations")
        sqlx::migrate::Migrator::new(FlywayMigrationSource(&migrations_root.join("migrations")))
            .await
            .expect(
                format!("error setting up migrator for {migrations_root:?}/migrations").as_str(),
            )
            .run(&pool)
            .await
            .expect("Failed to migrate the database");
        // sqlx::migrate!("./fixtures")
        sqlx::migrate::Migrator::new(migrations_root.join("./fixtures"))
            .await
            .expect(format!("error setting up migrator for {migrations_root:?}/fixtures").as_str())
            .set_ignore_missing(true) // don't inspect migrations store
            .run(&pool)
            .await
            .expect("Failed to add test data");

        Self {
            db_name: db_name.clone(),
            pool,
            clean_up_closure: Some(Box::new(move || {
                Box::pin(async move {
                    connection
                        .execute(&format!(r###"DROP DATABASE {db_name}"###)[..])
                        .await
                        .expect("Failed to drop test database.");
                })
            })),
        }
    }

    /// Call this after all holders of the [`SharedContext`] have been dropped.
    pub async fn close(self) {
        let Self {
            pool,
            mut clean_up_closure,
            ..
        } = self;
        pool.close().await;
        (clean_up_closure.take().unwrap())();
    }
}

pub struct TestContext {
    pub test_name: String,
    pub pools: HashMap<String, TestDb>,
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
                cx: &'a mut WalkCx,
            ) -> futures::future::BoxFuture<'a, Result<(), sqlx::error::BoxDynError>> {
                Box::pin(async move {
                    let mut s = tokio::fs::read_dir(path).await?;
                    while let Some(entry) = s.next_entry().await? {
                        // std::fs::metadata traverses symlinks
                        let metadata = std::fs::metadata(&entry.path())?;
                        if metadata.is_dir() {
                            walk_dir(&entry.path(), cx).await?;
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
            // ensure that we are sorted by `VERSION ASC`
            migrations.sort_by_key(|m| m.version);

            Ok(migrations)
        })
    }
}

impl TestContext {
    pub fn new(test_name: String, pools: impl Into<HashMap<String, TestDb>>) -> Self {
        setup_tracing_once();
        Self {
            test_name,
            pools: pools.into(),
        }
    }

    /// Call this after all holders of the [`SharedContext`] have been dropped.
    pub async fn close(mut self) {
        for (_, db) in self.pools.drain() {
            db.close().await;
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        for (db_name, _) in &self.pools {
            tracing::warn!("test context dropped without cleaning up for db: {db_name}",)
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
