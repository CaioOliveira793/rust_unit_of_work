use abstract_db_access::{
    sqlx::{SqlxTrxUnit, SqlxUnit},
    DbAccess, DbUnit, RepositoryError, TransactionUnit,
};
use async_trait::async_trait;
use sqlx::Executor;
use utilities::connection;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
struct User {
    id: uuid::Uuid,
    name: String,
    email: String,
}

#[async_trait]
trait UserRepository: DbAccess {
    async fn insert(&mut self, user: User) -> Result<(), RepositoryError>;
    async fn find(&mut self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError>;
}

async fn insert_user<'e, E>(executor: E, user: User) -> Result<(), RepositoryError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("INSERT INTO public.user (id, name, email) VALUES ($1, $2, $3)")
        .bind(user.id)
        .bind(user.name)
        .bind(user.email)
        .execute(executor)
        .await
        .unwrap();
    Ok(())
}

async fn find_user<'e, E>(executor: E, id: uuid::Uuid) -> Result<Option<User>, RepositoryError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    if let Some(user) =
        sqlx::query_as::<_, User>("SELECT (id, name, email) FROM public.user WHERE user.id = $1")
            .bind(id)
            .fetch_optional(executor)
            .await
            .unwrap()
    {
        return Ok(Some(user));
    }

    Ok(None)
}

#[async_trait]
impl UserRepository for SqlxUnit<sqlx::Postgres> {
    async fn insert(&mut self, user: User) -> Result<(), RepositoryError> {
        insert_user(self, user).await
    }

    async fn find(&mut self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        find_user(self, id).await
    }
}

#[async_trait]
impl<'t> UserRepository for SqlxTrxUnit<'t, sqlx::Postgres> {
    async fn insert(&mut self, user: User) -> Result<(), RepositoryError> {
        insert_user(self, user).await
    }

    async fn find(&mut self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        find_user(self, id).await
    }
}

// async fn multi_repo_transaction(
//     mut unit: SqlxUnit<sqlx::Postgres>,
//     user: User,
// ) -> Result<(), RepositoryError> {
//     let mut trx = DbUnit::transaction(&mut unit).await.unwrap();

//     UserRepository::insert(&mut trx, user).await.unwrap();

//     trx.commit().await.unwrap();

//     Ok(())
// }

async fn multi_repo(mut unit: SqlxUnit<sqlx::Postgres>, user: User) -> Result<(), RepositoryError> {
    UserRepository::insert(&mut unit, user).await.unwrap();

    Ok(())
}

#[allow(dead_code)]
async fn generic_function<Unit, Trx>(mut unit: Unit, user: User) -> Result<(), RepositoryError>
where
    for<'t> Unit: DbUnit<Transaction<'t> = Trx>,
    Unit: UserRepository,
    Trx: TransactionUnit,
    Trx: UserRepository,
{
    let mut trx = unit.transaction().await.unwrap();

    UserRepository::insert(&mut trx, user.clone())
        .await
        .unwrap();

    trx.commit().await.unwrap();

    let restored_user = UserRepository::find(&mut unit, user.id).await.unwrap();

    assert_eq!(restored_user, Some(user));

    Ok(())
}

async fn setup_db(pool: &sqlx::PgPool) {
    let mut client = pool.acquire().await.unwrap();

    client
        .execute(concat!(
            "DROP SCHEMA IF EXISTS public CASCADE;\n",
            "CREATE SCHEMA IF NOT EXISTS public;\n",
            "SET search_path TO public;\n",
            include_str!("dbschema.sql")
        ))
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let mut users = (0..).map(|idx| User {
        id: uuid::Uuid::new_v4(),
        email: format!("rustac{idx}@email.com"),
        name: format!("Rustacean {idx}"),
    });

    let pool = connection::create_sqlx_pool().await;

    setup_db(&pool).await;

    let client = pool.acquire().await.unwrap();

    multi_repo(client, users.next().unwrap().clone())
        .await
        .unwrap();

    // let client = pool.acquire().await.unwrap();
    // multi_repo_transaction(client, users.next().unwrap().clone())
    //     .await
    //     .unwrap();

    // NOTE: HRTB issue
    // let client = pool.acquire().await.unwrap();
    // generic_function(client, user.clone()).await.unwrap();
}
