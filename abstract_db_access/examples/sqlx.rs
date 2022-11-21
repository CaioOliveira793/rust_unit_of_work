use abstract_db_access::{
    sqlx::{SqlxTrxUnit, SqlxUnit},
    DbAccess, DbUnit, RepositoryError, TransactionUnit,
};
use async_trait::async_trait;
use utilities::connection;

#[derive(Debug, Clone, sqlx::FromRow)]
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
    sqlx::query("INSERT INTO user (id, name, email) VALUES ($1, $2, $3)")
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
        sqlx::query_as::<_, User>("SELECT (id, name, email) FROM user WHERE user.id = $1")
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

async fn multi_repo_transaction(
    mut unit: SqlxUnit<sqlx::Postgres>,
    user: User,
) -> Result<(), RepositoryError> {
    let mut trx = DbUnit::transaction(&mut unit).await.unwrap();

    UserRepository::insert(&mut trx, user.clone())
        .await
        .unwrap();

    trx.commit().await.unwrap();

    Ok(())
}

async fn multi_repo(mut unit: SqlxUnit<sqlx::Postgres>, user: User) -> Result<(), RepositoryError> {
    UserRepository::insert(&mut unit, user.clone())
        .await
        .unwrap();

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

    UserRepository::insert(&mut unit, user).await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() {
    let user = User {
        id: uuid::Uuid::new_v4(),
        email: "rustac@email.com".into(),
        name: "Rustacean".into(),
    };

    let pool = connection::create_sqlx_pool().await;

    let client = pool.acquire().await.unwrap();
    multi_repo(client, user.clone()).await.unwrap();

    let client = pool.acquire().await.unwrap();
    multi_repo_transaction(client, user.clone()).await.unwrap();

    // NOTE: HRTB issue
    // let client = pool.acquire().await.unwrap();
    // generic_function(client, user.clone()).await.unwrap();

    println!("{user:?}");
}
