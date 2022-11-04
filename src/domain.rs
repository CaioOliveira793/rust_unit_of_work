pub mod base {
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
    pub trait TransactionUnit: Repository + Transactor + Sized {
        async fn commit(self) -> Result<(), RepositoryError>;
        async fn rollback(self) -> Result<(), RepositoryError>;

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
}

pub mod entity {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct User {
        pub id: Uuid,
        pub name: String,
        pub email: String,
        pub phones: Vec<Phone>,
        pub created: DateTime<Utc>,
        pub updated: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Phone {
        pub number: String,
        pub verified: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CreateUserDto {
        pub id: Uuid,
        pub name: String,
        pub email: String,
        pub phones: Vec<Phone>,
    }

    impl TryFrom<CreateUserDto> for User {
        type Error = ();

        fn try_from(data: CreateUserDto) -> Result<Self, Self::Error> {
            if data.phones.is_empty() {
                return Err(());
            }

            Ok(Self {
                id: data.id,
                email: data.email,
                name: data.name,
                phones: data.phones,
                created: Utc::now(),
                updated: None,
            })
        }
    }
}

pub mod repository {
    use super::{
        base::{Repository, RepositoryError},
        entity::User,
    };
    use async_trait::async_trait;
    use uuid::Uuid;

    #[async_trait]
    pub trait UserRepository: Repository {
        async fn insert<I>(&mut self, users: I) -> Result<(), RepositoryError>
        where
            I: IntoIterator<Item = User> + Send;

        async fn find(&self, id: &Uuid) -> Result<Option<User>, RepositoryError>;
    }
}
