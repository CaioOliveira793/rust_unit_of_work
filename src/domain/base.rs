use async_trait::async_trait;
use tokio_postgres::error::DbError;

pub trait Repository {
    type Connection;
}

pub trait Transactor {
    type Transaction<'t>: TransactionUnit;
}

#[async_trait]
pub trait UnitOfWork: Repository + Transactor {
    /// Creates a new transaction.
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError>;
}

#[async_trait]
pub trait TransactionUnit: Repository + Transactor {
    async fn commit(self) -> Result<(), RepositoryError>;
    async fn rollback(self) -> Result<(), RepositoryError>;

    async fn save_point<'s>(
        &'s mut self,
        name: &str,
    ) -> Result<Self::Transaction<'s>, RepositoryError>
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
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_open_transaction(depth: u32) -> Self {
        Self { open: true, depth }
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    Db(DbError),
    Unknown(anyhow::Error),
}
