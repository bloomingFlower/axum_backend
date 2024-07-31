use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use scylla::{
    prepared_statement::PreparedStatement, query::Query, Bytes, FromRow, IntoTypedRows,
    SerializeRow, Session, SessionBuilder, ValueList,
};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path, path::PathBuf};

use crate::model::scylla::error::{Error, Result};
use tracing::{debug, error};

#[derive(PartialEq, Clone, Debug, SerializeRow, Deserialize, FromRow, ValueList, Serialize)]
pub struct HNStory {
    #[serde(alias = "objectID")]
    pub id: String,
    pub title: String,
    pub author: String,
    pub url: Option<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PagingState(pub String);

impl PagingState {
    pub fn new(encoded: String) -> Self {
        PagingState(encoded)
    }

    pub fn from_bytes(bytes: &Bytes) -> Self {
        PagingState(BASE64.encode(bytes))
    }

    pub fn into_bytes(&self) -> Result<Bytes> {
        BASE64
            .decode(&self.0)
            .map(Bytes::from)
            .map_err(|e| Error::Base64(e))
    }
}

pub async fn select_all_hnstories_with_pagination(
    session: &Session,
    page_size: i32,
    paging_state: Option<PagingState>,
) -> Result<(Vec<HNStory>, Option<PagingState>)> {
    debug!("--> HNStory: Selecting HNStories with pagination");
    let query_str = read_sql_file("hnstory/03-select-stories-with-pagination.sql")?;

    debug!("--> HNStory: page_size: {}", page_size);
    debug!("--> HNStory: paging_state: {:?}", paging_state);

    // Prepare the statement
    let prepared: PreparedStatement = session
        .prepare(Query::new(query_str).with_page_size(page_size))
        .await?;
    debug!("--> HNStory: prepared: {:?}", prepared);

    // Execute the query with paging state
    let result = session
        .execute_paged(
            &prepared,
            &[],
            paging_state.and_then(|ps| ps.into_bytes().ok()),
        )
        .await?;

    // Convert rows to HNStory instances with improved error handling and logging
    let stories: Vec<HNStory> = result
        .rows
        .unwrap_or_default()
        .into_typed::<HNStory>()
        .map(|row_result| match row_result {
            Ok(story) => {
                debug!("Successfully parsed HNStory: {:?}", story);
                Ok(story)
            }
            Err(e) => {
                error!("Error converting row to HNStory: {:?}", e);
                Err(Error::from(e))
            }
        })
        .collect::<Result<Vec<HNStory>>>()?;

    let new_paging_state = result.paging_state.as_ref().map(PagingState::from_bytes);

    debug!("--> HNStory: stories: {:?}", stories);

    Ok((stories, new_paging_state))
}

fn read_sql_file(file_name: &str) -> Result<String> {
    debug!("--> HNStory: Reading SQL file: {}", file_name);
    let project_root = find_project_root()?;
    let sql_file_path = project_root.join(SQL_DIR).join(file_name);
    fs::read_to_string(sql_file_path).map_err(Error::from)
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
    env::current_dir().map_err(Error::from)
}

fn is_project_root(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        || path.join(".git").exists()
        || path.join("sql").exists()
        || path.join("crates").exists()
}
