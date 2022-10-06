pub mod customer {
    use crate::{
        domain::{
            base::{TransactionUnit, UnitOfWork},
            entities::customer::{CreateCustomerData, Customer},
            repositories::CustomerRepository,
        },
        infra::repositories::PgClient,
    };

    pub struct CreateCustomerRequest {
        pub data: CreateCustomerData,
    }

    async fn create_unit_somehow<DB>() -> DB
    where
        DB: UnitOfWork,
    {
        todo!()
    }

    async fn validate_customer_data<CR>(_data: &CreateCustomerData, _repo: &CR)
    where
        CR: CustomerRepository,
    {
        todo!()
    }

    // pub async fn create_customer<'t, DB>(dto: CreateCustomerRequest) -> Result<Customer, ()>
    // where
    //     DB: UnitOfWork,
    //     DB: TransactionUnit,
    //     DB: CustomerRepository,
    // {
    //     let mut unit = create_unit_somehow::<DB>().await;

    //     validate_customer_data::<DB>(&dto.data, &unit).await;
    //     let customer = Customer::try_from(dto.data)?;

    //     let mut trx = unit.transaction().await.unwrap();

    //     trx.insert([customer.clone()]).await.unwrap();
    //     trx.insert([customer.clone()]).await.unwrap();

    //     trx.commit().await.unwrap();

    //     unit.insert([customer.clone()]).await.unwrap();

    //     CustomerRepository::insert(&mut unit, [customer])
    //         .await
    //         .unwrap();

    //     Err(())
    // }

    pub async fn concrete_create_customer(dto: CreateCustomerRequest) -> Result<Customer, ()> {
        let mut unit = create_unit_somehow::<PgClient>().await;

        validate_customer_data(&dto.data, &unit).await;
        let customer = Customer::try_from(dto.data)?;

        let mut trx = unit.transaction().await.unwrap();

        trx.insert([customer.clone()]).await.unwrap();
        trx.insert([customer.clone()]).await.unwrap();

        trx.commit().await.unwrap();

        unit.insert([customer.clone()]).await.unwrap();

        CustomerRepository::insert(&mut unit, [customer])
            .await
            .unwrap();

        Err(())
    }
}
