use abstract_db_access::{
    pg_deadpool::{PgTrxUnit, PgUnit},
    DbAccess, DbUnit, RepositoryError, TransactionUnit,
};
use async_trait::async_trait;
use utilities::connection;

#[derive(Debug, Clone, PartialEq)]
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
            "INSERT INTO public.user (id, name, email) VALUES ($1, $2, $3)",
            &[&user.id, &user.name, &user.email],
        )
        .await?;
        Ok(())
    }

    async fn find(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        let row = self
            .query_opt(
                "SELECT (id, name, email) FROM public.user WHERE user.id = $1",
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
                "INSERT INTO public.user (id, name, email) VALUES ($1, $2, $3)",
                &[&user.id, &user.name, &user.email],
            )
            .await?;
        Ok(())
    }

    async fn find(&self, id: uuid::Uuid) -> Result<Option<User>, RepositoryError> {
        let row = self
            .client
            .query_opt(
                "SELECT (id, name, email) FROM public.user WHERE user.id = $1",
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

    let restored_user = UserRepository::find(&mut unit, user.id).await.unwrap();

    assert_eq!(restored_user, Some(user));

    Ok(())
}

async fn setup_db(pool: &deadpool_postgres::Pool) {
    let mut client = pool.get().await.unwrap();
    let trx = client.transaction().await.unwrap();
    trx.client
        .batch_execute(concat!(
            "DROP SCHEMA IF EXISTS public CASCADE;\n",
            "CREATE SCHEMA IF NOT EXISTS public;\n",
            "SET search_path TO public;\n",
            include_str!("dbschema.sql")
        ))
        .await
        .unwrap();
    trx.commit().await.unwrap();
}

#[tokio::main]
async fn main() {
    let mut users = (0..).map(|idx| User {
        id: uuid::Uuid::new_v4(),
        email: format!("rustac{idx}@email.com"),
        name: format!("Rustacean {idx}"),
    });

    let pool = connection::create_pg_deadpool();

    setup_db(&pool).await;

    let client = pool.get().await.unwrap();
    multi_repo(client, users.next().unwrap().clone())
        .await
        .unwrap();

    let client = pool.get().await.unwrap();
    multi_repo_transaction(client, users.next().unwrap().clone())
        .await
        .unwrap();

    // NOTE: HRTB issue
    // let client = pool.get().await.unwrap();
    // generic_function(client, user.clone()).await.unwrap();
}
