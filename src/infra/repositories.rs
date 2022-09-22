use crate::domain::base::{Repository, RepositoryBuilder, RepositoryError, TransactionInfo};
use crate::domain::entities::*;
use crate::domain::repositories::*;
use crate::infra::{database::conversion::customer_from_row, sql};
use async_trait::async_trait;
use deadpool_postgres::Client;
use sea_query::{PostgresDriver, PostgresQueryBuilder};

pub struct PgClient<'a> {
    client: &'a Client,
    transaction: TransactionInfo,
}

impl<'a> RepositoryBuilder for PgClient<'a> {
    type Connection = &'a Client;

    fn repo<R>(&self) -> R
    where
        R: Repository<Connection = Self::Connection>,
    {
        R::new(&self.client, self.transaction)
    }
}

impl<'a> Repository for PgClient<'a> {
    type Connection = &'a Client;

    fn new(conn: Self::Connection, info: TransactionInfo) -> Self {
        Self {
            client: conn,
            transaction: info,
        }
    }
}

#[async_trait]
impl<'a> CustomerRepository for PgClient<'a> {
    async fn find(&self, id: &uuid::Uuid) -> Result<Option<customer::Customer>, RepositoryError> {
        let (customer_sttm, phone_sttm) = {
            let sql::customer::SelectCustomerSttm { customer, phone } =
                sql::customer::select_customer_by_id(id);

            let customer_sttm = customer.build(PostgresQueryBuilder);
            let phone_sttm = phone.build(PostgresQueryBuilder);
            (customer_sttm, phone_sttm)
        };

        let customer_rows = self
            .client
            .query(&customer_sttm.0, &customer_sttm.1.as_params())
            .await?;
        let phone_rows = self
            .client
            .query(&phone_sttm.0, &phone_sttm.1.as_params())
            .await?;

        let customer = customer_from_row(customer_rows, phone_rows)
            .into_iter()
            .next();
        Ok(customer)
    }

    async fn insert<I: IntoIterator<Item = customer::Customer> + Send>(
        &self,
        customers: I,
    ) -> Result<(), RepositoryError> {
        drop(customers);
        Ok(())
    }
}
