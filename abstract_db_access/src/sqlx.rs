use async_trait::async_trait;

use super::{DbAccess, DbUnit, RepositoryError, TransactionUnit, Transactor};

pub type SqlxUnit<DB> = sqlx_core::pool::PoolConnection<DB>;

pub type SqlxTrxUnit<'t, DB> = sqlx_core::transaction::Transaction<'t, DB>;

impl<DB: sqlx_core::database::Database> DbAccess for SqlxUnit<DB> {
    type Connection = Self;
    // NOTE: sqlx_core::acquire::Acquire is a more correct type to be this connection:
    // type Connection = <Self as Acquire>::Connection
    // but Acquire is only implemented for the concrete db connection types, PoolConnection
    // and Transaction with the DB generic specified.

    // looking at the types inside sqlx_core, seams that the `sqlx_core::acquire::Acquire` *could*
    // be implemented as
    // impl<DB: sqlx_core::database::Database> Acquire PoolConnection<DB> ...
    // impl<DB: sqlx_core::database::Database> Acquire Transaction<DB> ...
    // but maybe due lifetimes in some associated type it was not possible.

    // However, since a [big refactor](https://github.com/launchbadge/sqlx/issues/1163) is in the way,
    // maybe its worth changing some internals to bring more flexibility.
}

impl<DB: sqlx_core::database::Database> Transactor for SqlxUnit<DB> {
    type Transaction<'t> = SqlxTrxUnit<'t, DB>;
}

#[async_trait]
impl<DB: sqlx_core::database::Database> DbUnit for SqlxUnit<DB> {
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
        let trx = self.transaction().await.unwrap();
        Ok(trx)
    }
}

impl<'t, DB: sqlx_core::database::Database> DbAccess for SqlxTrxUnit<'t, DB> {
    type Connection = SqlxUnit<DB>;
    // NOTE: Same note from above goes to here:
    // type Connection = <Self as Acquire>::Connection
    // `sqlx::connection::Connection` trait is complete, havin methods to execute and create transactions
}

impl<'t, DB: sqlx_core::database::Database> Transactor for SqlxTrxUnit<'t, DB> {
    type Transaction<'trx> = SqlxTrxUnit<'trx, DB>;
}

#[async_trait]
impl<'t, DB: sqlx_core::database::Database> TransactionUnit for SqlxTrxUnit<'t, DB> {
    async fn commit(self) -> Result<(), RepositoryError> {
        self.commit().await.unwrap();
        Ok(())
    }

    async fn rollback(self) -> Result<(), RepositoryError> {
        self.rollback().await.unwrap();
        Ok(())
    }
}
