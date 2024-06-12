use serde::{Deserialize, Serialize};

/// 
#[derive(Debug, Serialize, Deserialize)]
pub struct HackerNewsResponse {
    pub hits: Vec<HNSearchResult>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct HNSearchResult {
    pub author: String,
    #[serde(alias = "objectID")]
    pub id: String,
    pub title: String,
    url: Option<String>,
    pub story_text: Option<String>,
    #[serde(alias = "_tags")]
    pub tags: Option<Vec<String>>,
    pub points: u32,
}

pub async fn fetch_hn_stories(
    search_term: String,
    search_result_limit: u32,
) -> Result<HackerNewsResponse, reqwest::Error> {
    let url_encoded_search_term = urlencoding::encode(&search_term);
    let url_str = format!(
        "https://hn.algolia.com/api/v1/search_by_date?query={}&tags=story&hitsPerPage={}",
        url_encoded_search_term, search_result_limit
    );
    let client = reqwest::Client::new();
    let request = client.get(url_str).build().unwrap();
    let json_response = client
        .execute(request)
        .await?
        .json::<HackerNewsResponse>()
        .await?;

    Ok(json_response)
}