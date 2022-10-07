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

pub struct PgClient<C: GenericClient> {
    client: C,
    transaction: TransactionState,
}

enum OpenTransaction<'c, C: GenericClient> {
    Created(Transaction<'c>),
    Reused(&'c mut C),
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

    pub(self) fn from_transaction_result<'t>(
        result: OpenTransaction<'t, C>,
        depth: u32,
    ) -> PgClient<Transaction<'t>> {
        match result {
            // TODO: what to do with reused open transactions?
            OpenTransaction::Reused(..) => todo!("cannot re open transaction"),
            OpenTransaction::Created(trx) => PgClient {
                client: trx,
                transaction: TransactionState::from_open_transaction(depth),
            },
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
    type Transaction<'t> = PgClient<Transaction<'t>>;
}

#[async_trait]
impl UnitOfWork for PgClient<Client> {
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
        let res = self.make_transaction().await?;
        Ok(Self::from_transaction_result(res, 0))
    }
}

#[async_trait]
impl<'t> TransactionUnit for PgClient<Transaction<'t>> {
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

        let c_rows = self.client.query(&c_sttm.0, &c_sttm.1.as_params()).await?;
        let p_rows = self.client.query(&p_sttm.0, &p_sttm.1.as_params()).await?;

        let customer = customer_from_row(c_rows, p_rows).into_iter().next();
        Ok(customer)
    }

    async fn insert<I: IntoIterator<Item = customer::Customer> + Send>(
        &mut self,
        customers: I,
    ) -> Result<(), RepositoryError> {
        let mut trx = self.make_transaction().await?;

        async fn inner_insert<C, I>(trx: &mut C, customers: I) -> Result<(), RepositoryError>
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

        match trx {
            OpenTransaction::Created(ref mut trx) => inner_insert(trx, customers).await?,
            OpenTransaction::Reused(trx) => inner_insert(trx, customers).await?,
        }

        Ok(())
    }
}
