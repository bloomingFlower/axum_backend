use crate::ctx::Ctx;
use crate::model::error::Error;
use crate::model::ModelManager;
use crate::model::Result;
use sqlb::HasFields;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub trait DbBmc {
    const TABLE: &'static str;
}

pub async fn get<M, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    M: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    //let sql = format!("SELECT * FROM {} WHERE id = $1", M::TABLE); // for sqlx
    let entity: E = sqlb::select()
        .table(M::TABLE)
        .columns(E::field_names())
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: M::TABLE,
            id,
        })?;

    Ok(entity)
}
