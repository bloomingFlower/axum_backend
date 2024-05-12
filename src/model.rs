use crate::ctx::Ctx;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

//region Ticket Type
/// Ticket Struct
#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64,
    pub title: String,
}

/// TicketForCreate Struct
#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}
//endregion

//region: Model Controller

/// Model Controller Struct
#[derive(Debug, Clone)]
pub struct ModelController {
    // Tickets Store Mutex Arc Vec Option Ticket
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

/// Model Controller Implementation
impl ModelController {
    /// New Model Controller
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }
}

// CRUD Implementation
impl ModelController {
    /// Create Ticket
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        // Tickets Store locked by Mutex
        let mut tickets_store = self.tickets_store.lock().unwrap();
        // Set Ticket ID as the length of Tickets Store
        let id = tickets_store.len() as u64;
        // Ticket Struct
        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };
        // Push Ticket to Tickets Store
        tickets_store.push(Some(ticket.clone()));
        // Return Ticket
        Ok(ticket)
    }

    /// Get Tickets List
    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        // Tickets Store locked by Mutex
        let tickets_store = self.tickets_store.lock().unwrap();
        // Get Tickets from Tickets Store and filter out None values and collect them into a Vec
        let tickets = tickets_store.iter().filter_map(|t| t.clone()).collect();
        // Return Tickets
        Ok(tickets)
    }

    /// Delete Ticket by ID from Tickets Store and return the deleted Ticket
    /// or an Error if the ID is not found in the Tickets Store
    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        // Tickets Store locked by Mutex
        let mut tickets_store = self.tickets_store.lock().unwrap();
        // Get Ticket by ID from Tickets Store and take it out of the Option and return it or an Error if the ID is not found in the Tickets Store
        let ticket = tickets_store.get_mut(id as usize).and_then(|t| t.take());
        // Return Ticket or Error
        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}

//endregion
