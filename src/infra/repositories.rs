use crate::domain::base::{
    Repository, RepositoryError, TransactionState, TransactionUnit, UnitOfWork,
};
use crate::domain::entities::*;
use crate::domain::repositories::*;
use crate::infra::{database::conversion::customer_from_row, sql};
use async_trait::async_trait;
use deadpool_postgres::{Client, Transaction};
use sea_query::PostgresQueryBuilder;
use uuid::Uuid;

pub enum PgConnection<'a> {
    Simple(&'a mut Client),
    Transaction {
        trx: Transaction<'a>,
        state: TransactionState,
    },
}

enum MakeTransactionResult<'a> {
    Reused,
    Open(Transaction<'a>),
}

pub struct PgClient<'a> {
    client: PgConnection<'a>,
    transaction: TransactionState,
}

impl<'a> PgClient<'a> {
    pub(self) async fn make_transaction<'s>(&'s mut self) -> MakeTransactionResult<'s> {
        match self.client {
            PgConnection::Simple(ref mut conn) => {
                let trx = conn.transaction().await.unwrap();
                MakeTransactionResult::Open(trx)
            }
            PgConnection::Transaction { .. } => MakeTransactionResult::Reused,
        }
    }

    pub(self) fn from_transaction_result<'t>(
        result: MakeTransactionResult<'t>,
        _depth: u32,
    ) -> PgClient<'t> {
        match result {
            MakeTransactionResult::Reused => panic!("handle reopened transactions"),
            MakeTransactionResult::Open(trx) => PgClient {
                client: PgConnection::Transaction {
                    trx,
                    state: TransactionState::created_transaction(),
                },
                transaction: TransactionState::created_transaction(),
            },
        }
    }
}

impl<'a> Repository for PgClient<'a> {
    type Connection = PgConnection<'a>;
}

#[async_trait]
impl<'a> UnitOfWork for PgClient<'a> {
    type Transaction<'t> = PgClient<'t> where Self: 't;

    /// Creates a new transaction.
    async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
        let res = self.make_transaction().await;
        Ok(Self::from_transaction_result(res, 0))
    }
}

#[async_trait]
impl<'a> TransactionUnit for PgClient<'a> {
    async fn commit(self) -> Result<(), RepositoryError> {
        todo!()
    }

    async fn rollback(self) -> Result<(), RepositoryError> {
        todo!()
    }

    async fn save_point(&mut self, _name: &str) -> Result<Self, RepositoryError>
    where
        Self: Sized,
    {
        todo!()
    }

    /// Returns the nested level
    fn depth(&self) -> u32 {
        0
    }
}

#[async_trait]
impl<'a> CustomerRepository for PgClient<'a> {
    async fn find(&self, id: &Uuid) -> Result<Option<customer::Customer>, RepositoryError> {
        let (_customer_sttm, _phone_sttm) = {
            let sql::customer::SelectCustomerSttm { customer, phone } =
                sql::customer::select_customer_by_id(id);

            let customer_sttm = customer.build(PostgresQueryBuilder);
            let phone_sttm = phone.build(PostgresQueryBuilder);
            (customer_sttm, phone_sttm)
        };

        // let conn = match &self.client {
        //     PgConnection::Simple(conn) => conn.transaction().await.unwrap(),
        //     PgConnection::Transaction { trx, .. } => trx,
        // };

        // let customer_rows = conn
        //     .query(&customer_sttm.0, &customer_sttm.1.as_params())
        //     .await?;
        // let phone_rows = conn
        //     .query(&phone_sttm.0, &phone_sttm.1.as_params())
        //     .await?;

        let customer_rows = vec![];
        let phone_rows = vec![];

        let customer = customer_from_row(customer_rows, phone_rows)
            .into_iter()
            .next();
        Ok(customer)
    }

    async fn insert<I: IntoIterator<Item = customer::Customer> + Send>(
        &mut self,
        customers: I,
    ) -> Result<(), RepositoryError> {
        // let trx = match &self.client {
        //     PgConnection::Simple(conn) => conn.transaction().await.unwrap(),
        //     PgConnection::Transaction { trx, .. } => trx,
        // };
        drop(customers);
        Ok(())
    }
}
