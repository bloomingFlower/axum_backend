// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

// endregion: --- Modules

#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: i64,
}

// Constructor.
impl Ctx {
    pub fn root_ctx() -> Self {
        // Root Context is created with user_id = 0 (System User)
        Ctx { user_id: 0 }
    }

    pub fn new(user_id: i64) -> Result<Self> {
        // New cannot be created with user_id = 0.
        if user_id == 0 {
            Err(Error::CtxCannotNewRootCtx)
        } else {
            Ok(Self { user_id })
        }
    }
}

// Property Accessors.
impl Ctx {
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
}
