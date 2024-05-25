use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

type Db = Pool<Postgres>;

// Note: The database URL is hardcoded for preventing the deployed system database from updating
const PG_DEV_POSTGRES_URL: &str = "postgres://dev:dev@localhost:5432/dev";
const PG_DEV_APP_URL: &str = "postgres://dev_app:dev_app@localhost:5432/dev_app";

// sql file path
const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

pub async fn init_dev_db() -> Result<(), Box<dyn Error>> {
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    // Create the database pool
    {
        let root_db = new_dev_db_pool(PG_DEV_POSTGRES_URL).await?;
        p_exec(&root_db, SQL_RECREATE_DB).await?;
    }

    // Get sql files
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    // SQL Execution
    let app_db = new_dev_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/");

            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                p_exec(&app_db, &path).await?;
            }
        }
    }

    Ok(())
}

async fn p_exec(db: &Db, sql_file: &str) -> Result<(), sqlx::Error> {
    info!("{:12} - p_exec - {sql_file}", "FOR-DEV_ONLY");
    let content = fs::read_to_string(sql_file)?;

    let sqls: Vec<&str> = content.split(";").collect();

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
