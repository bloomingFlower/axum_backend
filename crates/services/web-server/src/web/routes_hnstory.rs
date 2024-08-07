use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, options},
    Json, Router,
};
use lib_core::model::redis_cache::RedisManager;
use lib_core::model::scylla::hnstory::{
    select_all_hnstories_with_pagination, select_hnstory, PagingState,
};
use lib_core::model::scylla::ScyllaManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error};

pub fn routes(sm: Arc<ScyllaManager>, rm: Arc<RedisManager>) -> Router {
    Router::new()
        .route("/", get(list_hnstories))
        .route("/:id", get(get_hnstory))
        .route("/", options(handle_options))
        .with_state((sm, rm))
}

// HNStory select handler
async fn get_hnstory(
    State((sm, rm)): State<(Arc<ScyllaManager>, Arc<RedisManager>)>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let session = sm.session();

    match select_hnstory(session, rm.as_ref(), id).await {
        Ok(stories) => {
            if stories.is_empty() {
                (StatusCode::NOT_FOUND, "Story not found").into_response()
            } else {
                Json(stories[0].clone()).into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve story",
        )
            .into_response(),
    }
}

// HNStory list select handler
#[derive(Deserialize)]
pub struct PaginationParams {
    page_size: u32,
    paging_state: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    data: Vec<T>,
    next_paging_state: Option<String>,
}

async fn list_hnstories(
    State((sm, rm)): State<(Arc<ScyllaManager>, Arc<RedisManager>)>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    debug!("--> Route_HNStory: Listing HNStories with pagination");
    let session = sm.session();

    let paging_state = params.paging_state.map(PagingState::new);

    match select_all_hnstories_with_pagination(
        session,
        rm.as_ref(),
        params.page_size as i32,
        paging_state,
    )
    .await
    {
        Ok((stories, new_paging_state)) => {
            debug!(
                "--> Route_HNStory: Successfully retrieved {} stories",
                stories.len()
            );
            Json(PaginatedResponse {
                data: stories,
                next_paging_state: new_paging_state.map(|ps| ps.0),
            })
            .into_response()
        }
        Err(e) => {
            error!(
                "--> Route_HNStory: Failed to retrieve stories. Error: {:?}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve stories",
            )
                .into_response()
        }
    }
}

async fn handle_options() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
