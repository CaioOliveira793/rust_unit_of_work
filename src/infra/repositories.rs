use crate::domain::base::{
    Repository, RepositoryError, TransactionState, TransactionUnit, Transactor, UnitOfWork,
};
use crate::domain::entities::*;
use crate::domain::repositories::*;
use crate::infra::{database::conversion::customer_from_row, sql};
use async_trait::async_trait;
use sea_query::{PostgresDriver, PostgresQueryBuilder};
use tokio_postgres::{Client, GenericClient, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PgClient<C: GenericClient> {
    client: C,
    transaction: TransactionState,
}

pub type PgUnit = PgClient<Client>;
pub type PgTrxUnit<'t> = PgClient<Transaction<'t>>;

enum OpenTransaction<'c, C: GenericClient> {
    Created(Transaction<'c>),
    Reused(&'c mut C),
}

macro_rules! open_trx_conn {
    ($trx:ident, $func:expr) => {
        match $trx {
            OpenTransaction::Created(ref mut trx) => $func(trx).await,
            OpenTransaction::Reused(trx) => $func(trx).await,
        }
    };
}

impl<C: GenericClient> PgClient<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            transaction: TransactionState::new(),
        }
    }

    pub fn from_transaction(trx: C, depth: u32) -> Self {
        Self {
            client: trx,
            transaction: TransactionState::from_open_transaction(depth),
        }
    }

    pub(self) async fn make_transaction<'s>(
        &'s mut self,
    ) -> Result<OpenTransaction<'s, C>, RepositoryError> {
        if self.transaction.open {
            Ok(OpenTransaction::Reused(&mut self.client))
        } else {
            let trx = self.client.transaction().await?;
            Ok(OpenTransaction::Created(trx))
        }
    }
}

impl<C: GenericClient> Repository for PgClient<C> {
    type Connection = C;
}

impl<C: GenericClient> Transactor for PgClient<C> {
    type Transaction<'t> = PgTrxUnit<'t>;
}

#[async_trait]
impl UnitOfWork for PgUnit {
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
        let trx = self.client.transaction().await?;
        Ok(Self::Transaction {
            client: trx,
            transaction: TransactionState::from_open_transaction(0),
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

    async fn save_point<'s>(
        &'s mut self,
        name: &str,
    ) -> Result<Self::Transaction<'s>, RepositoryError>
    where
        Self: Sized,
    {
        let depth = self.depth() + 1;
        let point = self.client.savepoint(name).await?;
        Ok(Self::Transaction::from_transaction(point, depth))
    }

    fn depth(&self) -> u32 {
        self.transaction.depth
    }
}

#[async_trait]
impl<C: GenericClient + Send + Sync> CustomerRepository for PgClient<C> {
    async fn find(&self, id: &Uuid) -> Result<Option<customer::Customer>, RepositoryError> {
        let (c_sttm, p_sttm) = {
            let sql::customer::SelectCustomerSttm { customer, phone } =
                sql::customer::select_by_id(id);
            (
                customer.build(PostgresQueryBuilder),
                phone.build(PostgresQueryBuilder),
            )
        };

        let c_rows = self.client.query(&c_sttm.0, &c_sttm.1.as_params()).await;
        let p_rows = self.client.query(&p_sttm.0, &p_sttm.1.as_params()).await;

        let customer = customer_from_row(c_rows?, p_rows?).into_iter().next();
        Ok(customer)
    }

    async fn insert<I: IntoIterator<Item = customer::Customer> + Send>(
        &mut self,
        customers: I,
    ) -> Result<(), RepositoryError> {
        let mut trx = self.make_transaction().await?;

        open_trx_conn!(trx, async move |trx| insert_customer(trx, customers).await)?;

        Ok(())
    }
}

async fn insert_customer<C, I>(trx: &mut C, customers: I) -> Result<(), RepositoryError>
where
    C: GenericClient,
    I: IntoIterator<Item = customer::Customer> + Send,
{
    let (c_sttm, p_sttm) = {
        let sql::customer::InsertCustomerSttm { customer, phone } =
            sql::customer::insert(customers);
        (
            customer.build(PostgresQueryBuilder),
            phone.build(PostgresQueryBuilder),
        )
    };
    trx.query(&c_sttm.0, &c_sttm.1.as_params()).await?;
    trx.query(&p_sttm.0, &p_sttm.1.as_params()).await?;
    Ok(())
}
