use crate::model::{ModelController, Ticket, TicketForCreate};
use crate::Result;
use axum::extract::{FromRef, Path, State};
use axum::routing::{delete, post};
use axum::{Json, Router};

#[derive(Clone, FromRef)]
struct AppState {
    mc: ModelController,
}

pub fn routes(mc: ModelController) -> Router {
    let app_state = AppState { mc };
    Router::new()
        .route("/api/tickets", post(create_ticket).get(list_tickets))
        .route("/api/tickets/:id", delete(delete_ticket))
        .with_state(app_state)
}

// region: Restful Handler
async fn create_ticket(
    State(mc): State<ModelController>,
    Json(ticket_for_create): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    println!("--> {:<12} - create_ticket", "HANDLER");

    let ticket = mc.create_ticket(ticket_for_create).await?;

    Ok(Json(ticket))
}

async fn list_tickets(State(mc): State<ModelController>) -> Result<Json<Vec<Ticket>>> {
    println!("--> {:<12} - list_tickets", "HANDLER");

    let tickets = mc.list_tickets().await?;

    Ok(Json(tickets))
}

async fn delete_ticket(
    State(mc): State<ModelController>,
    Path(ticket_id): Path<u64>,
) -> Result<Json<Ticket>> {
    println!("--> {:<12} - delete_ticket", "HANDLER");

    let ticket = mc.delete_ticket(ticket_id).await?;

    Ok(Json(ticket))
}
