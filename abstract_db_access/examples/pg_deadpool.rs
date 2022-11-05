use abstract_db_access::{
    pg_deadpool::{PgTrxUnit, PgUnit},
    DbAccess, DbUnit, RepositoryError, TransactionUnit,
};
use async_trait::async_trait;
use utilities::connection;

#[derive(Debug, Clone)]
struct User {
    id: uuid::Uuid,
    name: String,
    email: String,
}

impl From<tokio_postgres::Row> for User {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        }
    }
}

#[async_trait]
trait UserRepository: DbAccess {
    async fn insert(&mut self, user: User) -> Result<(), RepositoryError>;
    async fn find(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError>;
}

#[async_trait]
impl UserRepository for PgUnit {
    async fn insert(&mut self, user: User) -> Result<(), RepositoryError> {
        self.query(
            "INSERT INTO user (id, name, email) VALUES ($1, $2, $3)",
            &[&user.id, &user.name, &user.email],
        )
        .await?;
        Ok(())
    }

    async fn find(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        let row = self
            .query_opt(
                "SELECT (id, name, email) FROM user WHERE user.id = $1",
                &[&id],
            )
            .await?;

        Ok(row.map(User::from))
    }
}

#[async_trait]
impl<'t> UserRepository for PgTrxUnit<'t> {
    async fn insert(&mut self, user: User) -> Result<(), RepositoryError> {
        self.client
            .query(
                "INSERT INTO user (id, name, email) VALUES ($1, $2, $3)",
                &[&user.id, &user.name, &user.email],
            )
            .await?;
        Ok(())
    }

    async fn find(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        let row = self
            .client
            .query_opt(
                "SELECT (id, name, email) FROM user WHERE user.id = $1",
                &[&id],
            )
            .await?;

        Ok(row.map(User::from))
    }
}

async fn multi_repo_transaction(mut unit: PgUnit, user: User) -> Result<(), RepositoryError> {
    let mut trx = DbUnit::transaction(&mut unit).await.unwrap();

    UserRepository::insert(&mut trx, user.clone())
        .await
        .unwrap();

    trx.commit().await.unwrap();

    Ok(())
}

async fn multi_repo(mut unit: PgUnit, user: User) -> Result<(), RepositoryError> {
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

    let pool = connection::create_pg_deadpool();

    let client = pool.get().await.unwrap();
    multi_repo(client, user.clone()).await.unwrap();

    let client = pool.get().await.unwrap();
    multi_repo_transaction(client, user.clone()).await.unwrap();

    // NOTE: HRTB issue
    // let client = pool.get().await.unwrap();
    // generic_function(client, user.clone()).await.unwrap();
}
