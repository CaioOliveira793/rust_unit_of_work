use async_trait::async_trait;

pub trait DbAccess {
    type Connection;
}

pub trait Transactor {
    type Transaction<'t>: TransactionUnit;
}

#[async_trait]
pub trait DbUnit: DbAccess + Transactor {
    /// Creates a new transaction.
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError>;
}

#[async_trait]
pub trait TransactionUnit: DbAccess + Transactor {
    async fn commit(self) -> Result<(), RepositoryError>;
    async fn rollback(self) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait SavePoint: TransactionUnit + Sized {
    async fn save_point<'s>(
        &'s mut self,
        name: &str,
    ) -> Result<Self::Transaction<'s>, RepositoryError>;

    /// Returns the nested level
    fn depth(&self) -> u32;
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct TransactionState {
    /// Indicates if transaction is open
    open: bool,
    /// Determines the transaction depth level
    depth: u32,
}

impl TransactionState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_open_transaction(depth: u32) -> Self {
        debug_assert!(
            depth > 0,
            "a open transaction must have at least one level of depth"
        );
        Self { open: true, depth }
    }

    /// Indicates if transaction is open
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Transaction depth level
    pub fn depth(&self) -> u32 {
        self.depth
    }
}

pub type UnknownError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[cfg(any(feature = "pg_tokio", feature = "pg_deadpool"))]
use tokio_postgres::error::DbError;

#[derive(Debug)]
pub enum RepositoryError {
    #[cfg(any(feature = "pg_tokio", feature = "pg_deadpool"))]
    TokioPostgres(DbError),
    Unknown(UnknownError),
}

#[cfg(any(feature = "pg_tokio", feature = "pg_deadpool"))]
impl From<tokio_postgres::Error> for RepositoryError {
    fn from(err: tokio_postgres::Error) -> Self {
        if let Some(db_err) = err.as_db_error() {
            return RepositoryError::TokioPostgres(db_err.clone());
        }

        RepositoryError::Unknown(err.into())
    }
}

#[cfg(feature = "pg_tokio")]
pub mod pg_tokio;

#[cfg(feature = "pg_deadpool")]
pub mod pg_deadpool;

#[cfg(feature = "sqlx")]
pub mod sqlx;
