use scylla::{FromRow, IntoTypedRows, SerializeRow, Session, SessionBuilder, ValueList};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path, path::PathBuf};

use crate::model::scylla::result::Result;

#[derive(PartialEq, Clone, Debug, SerializeRow, Deserialize, FromRow, ValueList, Serialize)]
pub struct HNStory {
    pub author: String,
    #[serde(alias = "objectID")]
    pub id: String,
    pub title: String,
    url: Option<String>,
    pub story_text: Option<String>,
    #[serde(alias = "_tags")]
    pub tags: Option<Vec<String>>,
    pub points: i32,
}

const SQL_DIR: &str = "sql/csql";

pub async fn create_session(uri: &str) -> Result<Session> {
    SessionBuilder::new()
        .known_node(uri)
        .build()
        .await
        .map_err(From::from)
}

pub async fn initialize(session: &Session) -> Result<()> {
    create_keyspace(session).await?;
    create_hnstory_table(session).await?;
    Ok(())
}

async fn create_keyspace(session: &Session) -> Result<()> {
    let query = read_sql_file("dev_initial/00-recreate-db.sql")?;
    session
        .query(query, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_hnstory_table(session: &Session) -> Result<()> {
    let query = read_sql_file("dev_initial/01-create-schema.sql")?;
    session
        .query(query, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn add_hnstory(session: &Session, hnstory: HNStory) -> Result<()> {
    let query = read_sql_file("hnstory/01-add-story.sql")?;
    session
        .query(query, hnstory)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn select_hnstory(session: &Session, id: String) -> Result<Vec<HNStory>> {
    let query = read_sql_file("hnstory/00-select-story.sql")?;
    session
        .query(query, (id,))
        .await?
        .rows
        .unwrap_or_default()
        .into_typed::<HNStory>()
        .map(|v| v.map_err(From::from))
        .collect()
}

pub async fn select_all_hnstories(session: &Session) -> Result<Vec<HNStory>> {
    let query = read_sql_file("hnstory/02-select-all-stories.sql")?;
    session
        .query(query, ())
        .await?
        .rows
        .unwrap_or_default()
        .into_typed::<HNStory>()
        .map(|v| v.map_err(From::from))
        .collect()
}

pub async fn select_all_hnstories_with_pagination(
    session: &Session,
    page: u32,
    limit: u32,
) -> Result<Vec<HNStory>> {
    let offset = (page - 1) * limit;
    let query = read_sql_file("hnstory/03-select-stories-with-pagination.sql")?;
    session
        .query(query, (limit as i64, offset as i64))
        .await?
        .rows
        .unwrap_or_default()
        .into_typed::<HNStory>()
        .map(|v| v.map_err(From::from))
        .collect()
}

fn read_sql_file(file_name: &str) -> Result<String> {
    let project_root = find_project_root()?;
    let sql_file_path = project_root.join(SQL_DIR).join(file_name);
    fs::read_to_string(sql_file_path).map_err(From::from)
}

fn find_project_root() -> Result<PathBuf> {
    // First, check environment variable
    if let Ok(project_root) = env::var("PROJECT_ROOT") {
        let path = PathBuf::from(project_root);
        if path.exists() {
            return Ok(path);
        }
    }

    // Check common container application root directories
    let possible_roots = vec!["/usr/src/app", "/app", "/home/app", "/opt/app"];

    for root in possible_roots {
        let path = PathBuf::from(root);
        if is_project_root(&path) {
            return Ok(path);
        }
    }

    // If all else fails, use the current directory
    env::current_dir().map_err(From::from)
}

fn is_project_root(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        || path.join(".git").exists()
        || path.join("sql").exists()
        || path.join("crates").exists()
}
