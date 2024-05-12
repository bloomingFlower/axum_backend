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
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut tickets_store = self.tickets_store.lock().unwrap();
        let id = tickets_store.len() as u64;
        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };
        tickets_store.push(Some(ticket.clone()));

        Ok(ticket)
    }

    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let tickets_store = self.tickets_store.lock().unwrap();
        let tickets = tickets_store.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets)
    }

    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        let mut tickets_store = self.tickets_store.lock().unwrap();
        let ticket = tickets_store.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}

//endregion
