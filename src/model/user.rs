use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::{ModelManager, Result};
use crate::pwd::{self, ContentToHash};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, Iden, PostgresQueryBuilder, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

/// User model
/// (FromRow trait is used to convert the result from the database to the struct)
/// (Fields trait is used to get the fields of the struct. It is used in the orm libraries)
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub password_clear: String,
}

#[derive(Fields)]
pub struct UserForInsert {
    username: String,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,
    pub password: Option<String>, // hashed
    pub password_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,
    pub token_salt: Uuid,
}

/// UserBy trait is used to get the user by different fields
/// (For example, get user by id, get user by username)
/// (HasFields trait is used to get the fields of the struct. It is used in the orm libraries)
/// (Unpin trait is used to make the struct usable in async functions. This trait enables the struct to be moved across threads(memory))
/// (Send trait is used to make the struct usable in async functions)
pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

// Iden trait is used to convert the struct name to string
#[derive(Iden)]
enum UserIden {
    Id,
    Username,
    Password,
}

pub struct UserBmc;

impl DbBmc for UserBmc {
    // The table name should be usable in the program
    const TABLE: &'static str = "user";
}

impl UserBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    /// Get user by username
    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();

        // Build the SQL query
        let mut query = sea_query::Query::select();
        query
            .from(Self::table_ref())
            .columns(E::field_idens())
            .and_where(Expr::col(UserIden::Username).eq(username));

        // Execute the query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let user = sqlx::query_as_with::<_, E, _>(&sql, values)
            .fetch_optional(db)
            .await?;

        Ok(user)
    }

    pub async fn update_pwd(
        ctx: &Ctx, 
        mm: &ModelManager, 
        id: i64, 
        pwd_clear: &str
    ) -> Result<()> {
        let db = mm.db();

        let user: UserForLogin = Self::get(ctx, mm, id).await?;
        let pwd_enc = pwd::hash_pwd(&ContentToHash {
            content: pwd_clear.to_string(),
            salt: user.password_salt,
        })?;

        // Build the SQL query
        let mut query = sea_query::Query::update();
        query
            .table(Self::table_ref())
            .value(UserIden::Password, SimpleExpr::from(pwd_enc))
            .and_where(Expr::col(UserIden::Id).eq(id));

        // Execute the query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let _count = sqlx::query_with(&sql, values)
            .execute(db)
            .await?
            .rows_affected();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_username = "demo1";

        let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .context("Should have user demo1")?;

        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
