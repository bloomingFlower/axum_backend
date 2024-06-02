use crate::ctx::Ctx;
use crate::model::error::Error;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::HasFields;
use modql::SIden;
use sea_query::{Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

#[derive(Iden)]
pub enum CommonIden {
    Id,
}

pub trait DbBmc {
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

pub async fn create<M, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    M: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    // Extract the fields and values
    let fields = data.not_none_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    // Build the SQL query
    let mut query = Query::insert();
    query
        .into_table(M::table_ref())
        .columns(columns)
        .values(sea_values)?
        .returning(Query::returning().columns([CommonIden::Id]));

    // Execute the query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let (id,) = sqlx::query_as_with::<_, (i64,), _>(&sql, values)
        .fetch_one(db)
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

    // Build the SQL query
    let mut query = Query::select();
    query
        .from(M::table_ref())
        .columns(E::field_column_refs())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // Execute the query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
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

    // Build the SQL query
    let mut query = Query::select();
    query.from(M::table_ref()).columns(E::field_column_refs());

    // Execute the query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
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

    // Prepare the fields and values
    let fields = data.not_none_fields();
    let fields = fields.for_sea_update();

    // Build the SQL query
    let mut query = Query::update();
    query
        .table(M::table_ref())
        .values(fields)
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // Execute the query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // Check if the entity was updated
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

    // Build the SQL query
    let mut query = Query::delete();
    query
        .from_table(M::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // Execute the query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // Check if the entity was deleted
    if count == 0 {
        Err(Error::EntityNotFound {
            entity: M::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
