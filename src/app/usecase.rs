pub mod customer {
    use tokio_postgres::Client;

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

    async fn validate_customer_data<Cr>(_data: &CreateCustomerData, _repo: &Cr)
    where
        Cr: CustomerRepository,
    {
        println!("validating...");
    }

    // pub async fn create_customer<'trx, Db>(dto: CreateCustomerRequest) -> Result<Customer, ()>
    // where
    //     Db: UnitOfWork<Transaction<'trx> = Db>,
    //     Db: TransactionUnit,
    //     Db: CustomerRepository,
    //     Db: 'trx,
    // {
    //     let mut unit = create_unit_somehow::<Db>().await;

    //     validate_customer_data::<Db>(&dto.data, &unit).await;
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

    pub async fn concrete_create_customer(
        unit: &mut PgClient<Client>,
        dto: CreateCustomerRequest,
    ) -> Result<Customer, ()> {
        validate_customer_data(&dto.data, unit).await;
        let customer = Customer::try_from(dto.data)?;

        let mut trx = unit.transaction().await.unwrap();

        trx.insert([customer.clone()]).await.unwrap();
        trx.insert([customer.clone()]).await.unwrap();

        trx.commit().await.unwrap();

        unit.insert([customer.clone()]).await.unwrap();

        CustomerRepository::insert(unit, [customer]).await.unwrap();

        Err(())
    }
}
