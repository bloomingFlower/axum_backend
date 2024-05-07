use crate::model::{ModelController, Ticket, TicketForCreate};
use crate::Result;
use axum::extract::State;
use axum::Json;

// region: Restful Handler
async fn create_ticket(
    State(mc): State<ModelController>,
    Json(ticket_for_create): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    println!("--> {:<12} - create_ticket", "HANDLER");

    let ticket = mc.create_ticket(ticket_for_create).await?;

    Ok(Json(ticket))
}
