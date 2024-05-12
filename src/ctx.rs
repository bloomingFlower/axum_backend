/// Context for the request
#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: u64,
}

/// Implement the Ctx type with a new method that returns a new Ctx instance with the user_id
impl Ctx {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }
}

/// Implement the Ctx type with a user_id method that returns the user_id
impl Ctx {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}
