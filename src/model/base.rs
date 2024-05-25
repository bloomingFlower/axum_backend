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

pub async fn create<M, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    M: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(M::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
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

pub async fn list<M, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    M: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    let entities: Vec<E> = sqlb::select()
        .table(M::TABLE)
        .columns(E::field_names())
        .order_by("id")
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn update<M, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    M: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let count = sqlb::update()
        .table(M::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: M::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<M>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    M: DbBmc,
{
    let db = mm.db();

    let count = sqlb::delete()
        .table(M::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: M::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
