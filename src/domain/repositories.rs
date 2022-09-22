use super::{
    base::{Repository, RepositoryError},
    entities::customer::Customer,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CustomerRepository: Repository {
    // async fn some_phone_numbers_exists<I: IntoIterator<Item = String>>(
    //     numbers: I,
    // ) -> Result<Vec<String>, RepositoryError>;

    // async fn some_cpfs_exists<I: IntoIterator<Item = String>>(
    //     cpfs: I,
    // ) -> Result<Vec<String>, RepositoryError>;

    async fn insert<I: IntoIterator<Item = Customer> + Send>(
        &self,
        customers: I,
    ) -> Result<(), RepositoryError>;
    async fn find(&self, id: &Uuid) -> Result<Option<Customer>, RepositoryError>;
}
