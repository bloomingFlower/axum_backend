use scylla::{FromRow, IntoTypedRows, SerializeRow, Session, SessionBuilder, ValueList};
use serde::Deserialize;

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

// Create keyspace
static CREATE_KEYSPACE_QUERY: &str = r#"
  CREATE KEYSPACE IF NOT EXISTS fast_logger
    WITH REPLICATION = {
      'class': 'NetworkTopologyStrategy',
      'replication_factor': 1
    };
"#;

// Create table
static CREATE_HNSTORY_TABLE_QUERY: &str = r#"
  CREATE TABLE IF NOT EXISTS fast_logger.hnstory (
    id text PRIMARY KEY,
    title text,
    author text,
    url text,
    story_text text,
    tags list<text>,
    points int
);
"#;

// Add story
static ADD_HNSTORY_QUERY: &str = r#"
  INSERT INTO fast_logger.hnstory (
  author, 
  id, 
  title, 
  url, 
  story_text, 
  tags, 
  points) 
  VALUES (?, ?, ?, ?, ?, ?, ?);
"#;

// Select story
static SELECT_HNSTORY_QUERY: &str = r#"
  SELECT * FROM fast_logger.hnstory
    WHERE id = ?;
"#;

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
    session
        .query(CREATE_KEYSPACE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_hnstory_table(session: &Session) -> Result<()> {
    session
        .query(CREATE_HNSTORY_TABLE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn add_hnstory(session: &Session, hnstory: HNStory) -> Result<()> {
    session
        .query(ADD_HNSTORY_QUERY, hnstory)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn select_hnstory(session: &Session, id: String) -> Result<Vec<HNStory>> {
    session
        .query(SELECT_HNSTORY_QUERY, (id,))
        .await?
        .rows
        .unwrap_or_default()
        .into_typed::<HNStory>()
        .map(|v| v.map_err(From::from))
        .collect()
}
