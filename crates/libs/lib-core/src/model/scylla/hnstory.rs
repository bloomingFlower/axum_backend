use scylla::{FromRow, IntoTypedRows, SerializeRow, Session, SessionBuilder, ValueList};
use serde::Deserialize;
use std::{fs, path::PathBuf};

use crate::model::scylla::result::Result;

#[derive(PartialEq, Clone, Debug, SerializeRow, Deserialize, FromRow, ValueList)]
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
    let query = read_sql_file("hnstory/02-select-story.sql")?;
    session
        .query(query, (id,))
        .await?
        .rows
        .unwrap_or_default()
        .into_typed::<HNStory>()
        .map(|v| v.map_err(From::from))
        .collect()
}

fn read_sql_file(file_name: &str) -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let v: Vec<_> = current_dir.components().collect();
    let path_comp = v.get(v.len().wrapping_sub(3));
    let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
        v[..v.len() - 3].iter().collect::<PathBuf>()
    } else {
        current_dir.clone()
    };
    let sql_file_path = base_dir.join(SQL_DIR).join(file_name);
    fs::read_to_string(sql_file_path).map_err(From::from)
}
