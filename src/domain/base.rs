use async_trait::async_trait;
use tokio_postgres::error::DbError;

#[async_trait]
pub trait UnitOfWork: RepositoryBuilder {
    /// Creates a new transaction.
    async fn transaction<Trx>(&self) -> Result<Trx, RepositoryError>
    where
        Trx: TransactionUnit<Connection = Self::Connection>;
}

#[async_trait]
pub trait TransactionUnit: RepositoryBuilder {
    async fn commit(self) -> Result<(), RepositoryError>;
    async fn rollback(self) -> Result<(), RepositoryError>;

    async fn save_point<SP>(&self, name: &str) -> Result<SP, RepositoryError>
    where
        SP: TransactionUnit<Connection = Self::Connection>;

    /// Returns the nested level
    fn depth() -> u32;
}

pub trait RepositoryBuilder {
    type Connection;

    fn repo<R>(&self) -> R
    where
        R: Repository<Connection = Self::Connection>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionInfo {
    /// Indicates if transaction is open
    pub open: bool,

    /// Determines the transaction depth level
    pub depth: u32,
}

pub trait Repository {
    type Connection;

    fn new(conn: Self::Connection, info: TransactionInfo) -> Self;
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    Connection(String),
    Db(DbError),
    Timeout,
    Unknown(String),
}
