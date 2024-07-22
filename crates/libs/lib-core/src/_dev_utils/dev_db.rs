use crate::core_config;
use crate::ctx::Ctx;
use crate::model::psql::user::{User, UserBmc};
use crate::model::psql::ModelManager;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};
use tracing::info;

type Db = Pool<Postgres>;

// sql file path from the project root
const SQL_RECREATE_DB: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/psql/dev_initial";

// Password for demo user
const DEMO_PWD: &str = "demo";

pub async fn init_dev_db() -> Result<(), Box<dyn Error>> {
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");
    info!(
        "{:<12} - init_dev_db() - {:?}",
        "FOR-DEV-ONLY",
        env::current_dir().expect("Can not get current dir")
    );

    let current_dir = env::current_dir()?;
    let v: Vec<_> = current_dir.components().collect();
    let path_comp = v.get(v.len().wrapping_sub(3));
    let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
        v[..v.len() - 3].iter().collect::<PathBuf>()
    } else {
        current_dir.clone()
    };
    info!("{:<12} - init_dev_db() - {:?}", "FOR-DEV-ONLY", base_dir);
    let sql_dir = base_dir.join(SQL_DIR);
    info!("{:<12} - init_dev_db() - {:?}", "FOR-DEV-ONLY", sql_dir);

    // Create the database pool
    {
        let sql_recreate_db_file = sql_dir.join(SQL_RECREATE_DB);
        let pg_dev_postgres_url = &core_config().PSQL_DB_URL_DEV;
        let root_db = new_dev_db_pool(pg_dev_postgres_url).await?;
        p_exec(&root_db, &sql_recreate_db_file).await?;
    }

    // Get sql files
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    // SQL Execution
    let pg_dev_app_url = &core_config().PSQL_DB_URL;
    let app_db = new_dev_db_pool(pg_dev_app_url).await?;
    for path in paths {
        let path_str = path.to_string_lossy();

        if path_str.ends_with(".sql") && !path_str.ends_with(SQL_RECREATE_DB) {
            p_exec(&app_db, &path).await?;
        }
    }

    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    let demo1_user: User = UserBmc::first_by_username(&ctx, &mm, "demo1")
        .await?
        .unwrap();

    UserBmc::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD).await?;
    info!("{:<12} - init_dev_db() - set demo1 pwd", "FOR-DEV-ONLY");

    Ok(())
}

async fn p_exec(db: &Db, file: &Path) -> Result<(), sqlx::Error> {
    info!("{:12} - p_exec - {file:?}", "FOR-DEV_ONLY");
    let content = fs::read_to_string(file)?;

    // Split the content by semicolon character
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }

    Ok(())
}

async fn new_dev_db_pool(db_conn_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_conn_url)
        .await
}
