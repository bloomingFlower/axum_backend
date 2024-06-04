use crate::model::store;
use crate::pwd;
use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From, Clone)]
pub enum Error {
    EntityNotFound {
        entity: &'static str,
        id: i64,
    },
    ListLimitOverMax {
        max: i64,
        actual: i64,
    },
    // -- Modules
    #[from]
    Pwd(pwd::Error),
    #[from]
    Store(store::Error),

    // -- Externals
    #[from]
    Sqlx(#[serde_as(as = "DisplayFromStr")] Arc<sqlx::Error>),
    #[from]
    SeaQuery(#[serde_as(as = "DisplayFromStr")] Arc<sea_query::error::Error>),
    #[from]
    ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] Arc<modql::filter::IntoSeaError>),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
