pub mod customer {
    use crate::{
        domain::{
            base::{TransactionUnit, Transactor, UnitOfWork},
            entities::customer::{CreateCustomerData, Customer},
            repositories::CustomerRepository,
        },
        infra::repositories::PgUnit,
    };

    async fn validate_customer_data<Cr>(_data: &CreateCustomerData, _repo: &Cr)
    where
        Cr: CustomerRepository,
    {
        println!("validating...");
    }

    pub async fn create_customer<Unit>(
        mut unit: Unit,
        data: CreateCustomerData,
    ) -> Result<Customer, ()>
    where
        Unit: UnitOfWork,
        Unit: CustomerRepository,
        for<'t> <Unit as Transactor>::Transaction<'t>: CustomerRepository,
    {
        validate_customer_data(&data, &unit).await;
        let customer = Customer::try_from(data)?;

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

    pub async fn concrete_create_customer(
        mut unit: PgUnit,
        data: CreateCustomerData,
    ) -> Result<Customer, ()> {
        validate_customer_data(&data, &unit).await;
        let customer = Customer::try_from(data)?;

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
