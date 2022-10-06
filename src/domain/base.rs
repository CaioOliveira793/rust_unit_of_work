use async_trait::async_trait;
use tokio_postgres::error::DbError;

pub trait Repository {
    type Connection;
}

#[async_trait]
pub trait UnitOfWork: Repository {
    type Transaction<'t>: TransactionUnit
    where
        Self: 't;

    /// Creates a new transaction.
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError>;
}

#[async_trait]
pub trait TransactionUnit: Repository {
    async fn commit(self) -> Result<(), RepositoryError>;
    async fn rollback(self) -> Result<(), RepositoryError>;

    async fn save_point(&mut self, name: &str) -> Result<Self, RepositoryError>
    where
        Self: Sized;

    /// Returns the nested level
    fn depth(&self) -> u32;
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct TransactionState {
    /// Indicates if transaction is open
    pub open: bool,

    /// Determines the transaction depth level
    pub depth: u32,
}

impl TransactionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn created_transaction() -> Self {
        Self {
            open: true,
            depth: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    Connection(String),
    Db(DbError),
    Timeout,
    Unknown(String),
}
