pub mod transform {
    use std::collections::HashMap;

    use chrono::{DateTime, NaiveDateTime, Utc};
    use tokio_postgres::Error;
    use uuid::Uuid;

    use crate::domain::{
        base::RepositoryError,
        entity::{Phone, User},
    };

    impl From<Error> for RepositoryError {
        fn from(err: Error) -> Self {
            if let Some(db_err) = err.as_db_error() {
                return RepositoryError::Db(db_err.clone());
            }

            RepositoryError::Unknown(err.into())
        }
    }

    impl From<tokio_postgres::Row> for Phone {
        fn from(row: tokio_postgres::Row) -> Self {
            Self {
                verified: row.get("verified"),
                number: row.get("number"),
            }
        }
    }

    pub fn map_row_by<T, Rows>(id_idx: &str, rows: Rows) -> HashMap<Uuid, Vec<T>>
    where
        T: From<tokio_postgres::Row>,
        Rows: IntoIterator<Item = tokio_postgres::Row>,
    {
        let mut map: HashMap<Uuid, Vec<T>> = HashMap::new();
        for row in rows {
            let id: Uuid = row.get(id_idx);
            let value: T = row.into();
            if let Some(values) = map.get_mut(&id) {
                values.push(value);
            } else {
                map.insert(id, vec![value]);
            }
        }
        map
    }

    pub fn user_from_row(
        user_rows: Vec<tokio_postgres::Row>,
        phone_rows: Vec<tokio_postgres::Row>,
    ) -> Vec<User> {
        let mut phones_map: HashMap<Uuid, Vec<Phone>> = map_row_by("user_id", phone_rows);

        user_rows
            .into_iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let updated: Option<NaiveDateTime> = row.get("updated");
                User {
                    id,
                    email: row.get("email"),
                    name: row.get("name"),
                    phones: phones_map.remove(&id).unwrap_or_default(),
                    created: DateTime::from_utc(row.get("created"), Utc),
                    updated: updated.map(|naive| DateTime::from_utc(naive, Utc)),
                }
            })
            .collect()
    }
}

pub mod deadpool {
    use async_trait::async_trait;

    use crate::{
        config::connection::PgDeadpoolConn,
        domain::base::{
            Repository, RepositoryError, TransactionState, TransactionUnit, Transactor, UnitOfWork,
        },
    };

    pub type PgDeadpoolUnit = PgDeadpoolConn;
    pub struct PgDeadpoolTrxUnit<'t> {
        // NOTE: not possible to disambiguate `<deadpool_postgres::Client as Deref>::transaction`
        // so the transaction client type is not wrapped
        pub client: tokio_postgres::Transaction<'t>,
        pub state: TransactionState,
    }

    impl Repository for PgDeadpoolUnit {
        type Connection = PgDeadpoolConn;
    }

    impl Transactor for PgDeadpoolUnit {
        type Transaction<'t> = PgDeadpoolTrxUnit<'t>;
    }

    #[async_trait]
    impl<'t> UnitOfWork for PgDeadpoolUnit {
        async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
            let client = tokio_postgres::Client::transaction(self).await?;
            let state = TransactionState::from_open_transaction(0);
            Ok(Self::Transaction { client, state })
        }
    }

    impl<'t> Repository for PgDeadpoolTrxUnit<'t> {
        type Connection = PgDeadpoolConn;
    }

    impl<'t> Transactor for PgDeadpoolTrxUnit<'t> {
        type Transaction<'trx> = PgDeadpoolTrxUnit<'trx>;
    }

    #[async_trait]
    impl<'t> TransactionUnit for PgDeadpoolTrxUnit<'t> {
        async fn commit(self) -> Result<(), RepositoryError> {
            self.commit().await?;
            Ok(())
        }

        async fn rollback(self) -> Result<(), RepositoryError> {
            self.rollback().await?;
            Ok(())
        }

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
}

pub mod tokio {
    use async_trait::async_trait;
    use tokio_postgres::{Client, GenericClient, Transaction};

    use crate::domain::base::{
        Repository, RepositoryError, TransactionState, TransactionUnit, Transactor, UnitOfWork,
    };

    #[derive(Debug, Clone)]
    pub struct PgClient<C: GenericClient> {
        pub(super) client: C,
        pub(super) transaction: TransactionState,
    }

    pub type PgTokioUnit = PgClient<Client>;
    pub type PgTokioTrxUnit<'t> = PgClient<Transaction<'t>>;

    pub(super) enum OpenTransaction<'c, C: GenericClient> {
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

        pub(super) async fn make_transaction<'s>(
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
        type Transaction<'t> = PgTokioTrxUnit<'t>;
    }

    #[async_trait]
    impl UnitOfWork for PgTokioUnit {
        async fn transaction<'s>(&'s mut self) -> Result<Self::Transaction<'s>, RepositoryError> {
            let trx = self.client.transaction().await?;
            Ok(Self::Transaction {
                client: trx,
                transaction: TransactionState::from_open_transaction(0),
            })
        }
    }

    #[async_trait]
    impl<'t> TransactionUnit for PgTokioTrxUnit<'t> {
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
        ) -> Result<Self::Transaction<'s>, RepositoryError> {
            let depth = self.depth() + 1;
            let point = self.client.savepoint(name).await?;
            Ok(Self::Transaction::from_transaction(point, depth))
        }

        fn depth(&self) -> u32 {
            self.transaction.depth
        }
    }
}

pub mod repository {
    use async_trait::async_trait;
    use sea_query::{PostgresDriver, PostgresQueryBuilder};
    use tokio_postgres::GenericClient;
    use uuid::Uuid;

    use crate::domain::{base::RepositoryError, entity::User, repository::UserRepository};
    use crate::infra::{database::transform::user_from_row, sql};

    async fn insert_user<C, I>(trx: &mut C, users: I) -> Result<(), RepositoryError>
    where
        C: GenericClient,
        I: IntoIterator<Item = User> + Send,
    {
        let (c_sttm, p_sttm) = {
            let sql::InsertUserSttm { user, phone } = sql::insert_user(users);
            (
                user.build(PostgresQueryBuilder),
                phone.build(PostgresQueryBuilder),
            )
        };
        trx.query(&c_sttm.0, &c_sttm.1.as_params()).await?;
        trx.query(&p_sttm.0, &p_sttm.1.as_params()).await?;
        Ok(())
    }

    pub mod pgtokio_repo {
        use super::*;
        use crate::infra::database::tokio::{OpenTransaction, PgClient};

        #[async_trait]
        impl<C: GenericClient + Send + Sync> UserRepository for PgClient<C> {
            async fn find(&self, id: &Uuid) -> Result<Option<User>, RepositoryError> {
                let (u_sttm, p_sttm) = {
                    let sql::SelectUserSttm { user, phone } = sql::select_user_by_id(id);
                    (
                        user.build(PostgresQueryBuilder),
                        phone.build(PostgresQueryBuilder),
                    )
                };

                let u_rows = self.client.query(&u_sttm.0, &u_sttm.1.as_params()).await?;
                let p_rows = self.client.query(&p_sttm.0, &p_sttm.1.as_params()).await?;

                let user = user_from_row(u_rows, p_rows).into_iter().next();
                Ok(user)
            }

            async fn insert<I>(&mut self, users: I) -> Result<(), RepositoryError>
            where
                I: IntoIterator<Item = User> + Send,
            {
                let mut trx = self.make_transaction().await?;

                match trx {
                    OpenTransaction::Created(ref mut trx) => insert_user(trx, users).await?,
                    OpenTransaction::Reused(trx) => insert_user(trx, users).await?,
                }

                Ok(())
            }
        }
    }

    pub mod deadpool_repo {
        use super::*;
        use crate::infra::database::deadpool::{PgDeadpoolTrxUnit, PgDeadpoolUnit};

        #[async_trait]
        impl UserRepository for PgDeadpoolUnit {
            async fn find(&self, id: &Uuid) -> Result<Option<User>, RepositoryError> {
                let (u_sttm, p_sttm) = {
                    let sql::SelectUserSttm { user, phone } = sql::select_user_by_id(id);
                    (
                        user.build(PostgresQueryBuilder),
                        phone.build(PostgresQueryBuilder),
                    )
                };

                let u_rows = self.query(&u_sttm.0, &u_sttm.1.as_params()).await?;
                let p_rows = self.query(&p_sttm.0, &p_sttm.1.as_params()).await?;

                let user = user_from_row(u_rows, p_rows).into_iter().next();
                Ok(user)
            }

            async fn insert<I>(&mut self, users: I) -> Result<(), RepositoryError>
            where
                I: IntoIterator<Item = User> + Send,
            {
                let mut trx = tokio_postgres::Client::transaction(self).await?;
                insert_user(&mut trx, users).await?;
                Ok(())
            }
        }

        #[async_trait]
        impl<'t> UserRepository for PgDeadpoolTrxUnit<'t> {
            async fn find(&self, id: &Uuid) -> Result<Option<User>, RepositoryError> {
                let (u_sttm, p_sttm) = {
                    let sql::SelectUserSttm { user, phone } = sql::select_user_by_id(id);
                    (
                        user.build(PostgresQueryBuilder),
                        phone.build(PostgresQueryBuilder),
                    )
                };

                let u_rows = self.client.query(&u_sttm.0, &u_sttm.1.as_params()).await?;
                let p_rows = self.client.query(&p_sttm.0, &p_sttm.1.as_params()).await?;

                let user = user_from_row(u_rows, p_rows).into_iter().next();
                Ok(user)
            }

            async fn insert<I>(&mut self, users: I) -> Result<(), RepositoryError>
            where
                I: IntoIterator<Item = User> + Send,
            {
                insert_user(&mut self.client, users).await?;
                Ok(())
            }
        }
    }
}
