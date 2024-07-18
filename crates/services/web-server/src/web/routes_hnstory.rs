use axum::{extract::Path, extract::Query, http::StatusCode, response::IntoResponse, Json};
use lib_core::model::scylla::db_conn;
use lib_core::model::scylla::hnstory::{select_all_hnstories_with_pagination, select_hnstory};
use serde::{Deserialize, Serialize};

// HNStory select handler
pub async fn get_hnstory(Path(id): Path<String>) -> impl IntoResponse {
    let session = match db_conn().await {
        Ok(session) => session,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection failed",
            )
                .into_response()
        }
    };

    match select_hnstory(&session, id).await {
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
    page: u32,
    limit: u32,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    data: Vec<T>,
    page: u32,
    limit: u32,
}

pub async fn list_hnstories(Query(params): Query<PaginationParams>) -> impl IntoResponse {
    let session = match db_conn().await {
        Ok(session) => session,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection failed",
            )
                .into_response()
        }
    };

    match select_all_hnstories_with_pagination(&session, params.page, params.limit).await {
        Ok(stories) => Json(PaginatedResponse {
            data: stories,
            page: params.page,
            limit: params.limit,
        })
        .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve stories",
        )
            .into_response(),
    }
}
