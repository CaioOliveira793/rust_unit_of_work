pub mod customer {
    use crate::domain::{
        base::{TransactionUnit, UnitOfWork},
        entities::customer::{CreateCustomerData, Customer},
        repositories::CustomerRepository,
    };

    pub struct CreateCustomerRequest {
        pub data: CreateCustomerData,
    }

    async fn create_unit_somehow<UoW, Conn>() -> UoW
    where
        UoW: UnitOfWork<Connection = Conn>,
    {
        todo!()
    }

    async fn validate_customer_data<CR>(data: &CreateCustomerData, repo: &CR)
    where
        CR: CustomerRepository,
    {
        todo!()
    }

    pub async fn create_customer<UoW, Trx, CR, Conn>(
        dto: CreateCustomerRequest,
    ) -> Result<Customer, ()>
    where
        UoW: UnitOfWork<Connection = Conn>,
        Trx: TransactionUnit<Connection = Conn>,
        CR: CustomerRepository<Connection = Conn>,
    {
        let unit = create_unit_somehow::<UoW, Conn>().await;

        validate_customer_data::<CR>(&dto.data, &unit.repo::<CR>()).await;
        let customer = Customer::try_from(dto.data)?;

        let trx = unit.transaction::<Trx>().await.unwrap();

        trx.repo::<CR>().insert([customer.clone()]).await.unwrap();
        trx.repo::<CR>().insert([customer]).await.unwrap();

        trx.commit().await.unwrap();
        Err(())
    }
}
