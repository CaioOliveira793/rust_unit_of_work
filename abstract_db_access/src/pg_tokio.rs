use async_trait::async_trait;
use tokio_postgres::{Client, GenericClient, Transaction};

use super::{
    DbAccess, DbUnit, RepositoryError, SavePoint, TransactionState, TransactionUnit, Transactor,
};

#[derive(Debug, Clone)]
pub struct PgClient<C: GenericClient> {
    client: C,
    state: TransactionState,
}

pub type PgUnit = PgClient<Client>;
pub type PgTrxUnit<'t> = PgClient<Transaction<'t>>;

pub enum OpenTransaction<'c, C: GenericClient> {
    Created(Transaction<'c>),
    Reused(&'c mut C),
}

impl<C: GenericClient> PgClient<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            state: TransactionState::new(),
        }
    }

    pub fn from_transaction(trx: C, depth: u32) -> Self {
        Self {
            client: trx,
            state: TransactionState::from_open_transaction(depth),
        }
    }

    pub async fn make_transaction<'s>(
        &'s mut self,
    ) -> Result<OpenTransaction<'s, C>, RepositoryError> {
        if self.state.open {
            Ok(OpenTransaction::Reused(&mut self.client))
        } else {
            let trx = self.client.transaction().await?;
            Ok(OpenTransaction::Created(trx))
        }
    }

    pub fn client(&self) -> &C {
        &self.client
    }

    pub fn transaction_state(&self) -> &TransactionState {
        &self.state
    }
}

impl<C: GenericClient> DbAccess for PgClient<C> {
    type Connection = C;
}

impl<C: GenericClient> Transactor for PgClient<C> {
    type Transaction<'t> = PgTrxUnit<'t>;
}

#[async_trait]
impl DbUnit for PgUnit {
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
        let trx = self.client.transaction().await?;
        Ok(Self::Transaction {
            client: trx,
            state: TransactionState::from_open_transaction(0),
        })
    }
}

#[async_trait]
impl<'t> TransactionUnit for PgTrxUnit<'t> {
    async fn commit(self) -> Result<(), RepositoryError> {
        self.client.commit().await?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), RepositoryError> {
        self.client.rollback().await?;
        Ok(())
    }
}

#[async_trait]
impl<'t> SavePoint for PgTrxUnit<'t> {
    async fn save_point<'s>(
        &'s mut self,
        name: &str,
    ) -> Result<Self::Transaction<'s>, RepositoryError> {
        let depth = self.depth() + 1;
        let point = self.client.savepoint(name).await?;
        Ok(Self::Transaction::from_transaction(point, depth))
    }

    fn depth(&self) -> u32 {
        self.state.depth
    }
}
