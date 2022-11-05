use async_trait::async_trait;

use super::{
    DbAccess, DbUnit, RepositoryError, SavePoint, TransactionState, TransactionUnit, Transactor,
};

pub type PgUnit = deadpool_postgres::Client;
pub struct PgTrxUnit<'t> {
    // NOTE: not possible to disambiguate `<deadpool_postgres::Client as Deref>::transaction`
    // so the transaction client type is not wrapped
    pub client: tokio_postgres::Transaction<'t>,
    pub state: TransactionState,
}

impl DbAccess for PgUnit {
    type Connection = deadpool_postgres::Client;
}

impl Transactor for PgUnit {
    type Transaction<'t> = PgTrxUnit<'t>;
}

#[async_trait]
impl<'t> DbUnit for PgUnit {
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
        let client = tokio_postgres::Client::transaction(self).await?;
        let state = TransactionState::from_open_transaction(0);
        Ok(Self::Transaction { client, state })
    }
}

impl<'t> DbAccess for PgTrxUnit<'t> {
    type Connection = deadpool_postgres::Client;
}

impl<'t> Transactor for PgTrxUnit<'t> {
    type Transaction<'trx> = PgTrxUnit<'trx>;
}

#[async_trait]
impl<'t> TransactionUnit for PgTrxUnit<'t> {
    async fn commit(self) -> Result<(), RepositoryError> {
        self.commit().await?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), RepositoryError> {
        self.rollback().await?;
        Ok(())
    }
}

#[async_trait]
impl<'t> SavePoint for PgTrxUnit<'t> {
    async fn save_point<'s>(
        &'s mut self,
        name: &str,
    ) -> Result<Self::Transaction<'s>, RepositoryError> {
        let state = TransactionState::from_open_transaction(self.depth() + 1);
        let client = self.client.savepoint(name).await?;
        Ok(Self::Transaction { client, state })
    }

    fn depth(&self) -> u32 {
        self.state.depth
    }
}
