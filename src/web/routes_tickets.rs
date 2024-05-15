use crate::ctx::Ctx;
use crate::model::{ModelController, Ticket, TicketForCreate};
use crate::Result;
use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Json, Router};
use tracing::debug;

/// Create the Tickets Routes and return the Router
pub fn routes(mc: ModelController) -> Router {
    Router::new()
        // Create the Tickets Route with the POST method and the create_ticket handler
        // Get the list of tickets with the GET method and the list_tickets handler
        .route("/tickets", post(create_ticket).get(list_tickets))
        // Create the Tickets Route with the DELETE method and the delete_ticket handler
        .route("/tickets/:id", delete(delete_ticket))
        // Add the Model Controller as a State
        .with_state(mc)
}

// region: Restful Handler
/// Create Ticket Handler that returns a JSON response with the created ticket
async fn create_ticket(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Json(ticket_for_create): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    debug!(" {:<12} - create_ticket", "HANDLER");
    // Create a ticket with the ticket_for_create data
    let ticket = mc.create_ticket(ctx, ticket_for_create).await?;

    Ok(Json(ticket))
}

/// List Tickets Handler that returns a JSON response with the list of tickets
async fn list_tickets(State(mc): State<ModelController>, ctx: Ctx) -> Result<Json<Vec<Ticket>>> {
    debug!(" {:<12} - list_tickets", "HANDLER");
    // Get the list of tickets
    let tickets = mc.list_tickets(ctx).await?;

    Ok(Json(tickets))
}

/// Delete Ticket Handler that returns a JSON response with the deleted ticket
async fn delete_ticket(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(ticket_id): Path<u64>,
) -> Result<Json<Ticket>> {
    debug!(" {:<12} - delete_ticket", "HANDLER");
    // Delete the ticket by the ticket_id
    let ticket = mc.delete_ticket(ctx, ticket_id).await?;

    Ok(Json(ticket))
}
