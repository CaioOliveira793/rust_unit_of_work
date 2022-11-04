pub mod customer {
    use sea_query::{Alias, Expr, InsertStatement, Query, SelectStatement};
    use uuid::Uuid;

    use crate::{
        domain::entities::customer::Customer,
        infra::table::{TCustomer, TCustomerPhone},
    };

    pub struct SelectCustomerSttm {
        pub customer: SelectStatement,
        pub phone: SelectStatement,
    }

    pub struct InsertCustomerSttm {
        pub customer: InsertStatement,
        pub phone: InsertStatement,
    }

    pub fn select_by_id(id: &Uuid) -> SelectCustomerSttm {
        let mut customer_sttm = Query::select();
        customer_sttm
            .from(TCustomer::Table)
            .expr_as(Expr::tbl(TCustomer::Table, TCustomer::Id), Alias::new("id"))
            .expr_as(Expr::col(TCustomer::Name), Alias::new("name"))
            .expr_as(Expr::col(TCustomer::Cpf), Alias::new("cpf"))
            .expr_as(Expr::col(TCustomer::Created), Alias::new("created"))
            .expr_as(Expr::col(TCustomer::Updated), Alias::new("updated"))
            .and_where(Expr::col(TCustomer::Id).eq(*id));

        let mut phone_sttm = Query::select();
        phone_sttm
            .from(TCustomerPhone::Table)
            .expr_as(
                Expr::tbl(TCustomerPhone::Table, TCustomerPhone::Id),
                Alias::new("id"),
            )
            .expr_as(
                Expr::col(TCustomerPhone::CustomerId),
                Alias::new("customer_id"),
            )
            .expr_as(Expr::col(TCustomerPhone::Number), Alias::new("number"))
            .expr_as(Expr::col(TCustomerPhone::Whatsapp), Alias::new("whatsapp"))
            .and_where(Expr::col(TCustomerPhone::CustomerId).eq(*id));

        SelectCustomerSttm {
            customer: customer_sttm,
            phone: phone_sttm,
        }
    }

    pub fn insert<I>(customers: I) -> InsertCustomerSttm
    where
        I: IntoIterator<Item = Customer>,
    {
        let mut customer = Query::insert();
        customer.into_table(TCustomer::Table);
        customer.columns([
            TCustomer::Id,
            TCustomer::Cpf,
            TCustomer::Name,
            TCustomer::Created,
            TCustomer::Updated,
        ]);

        let mut phone = Query::insert();
        phone.into_table(TCustomerPhone::Table);
        phone.columns([
            TCustomerPhone::Id,
            TCustomerPhone::Number,
            TCustomerPhone::Whatsapp,
            TCustomerPhone::CustomerId,
        ]);

        for cust in customers {
            customer.values_panic([
                cust.id.into(),
                cust.cpf.into(),
                cust.name.into(),
                cust.created.into(),
                cust.updated.into(),
            ]);

            for p in cust.phones {
                phone.values_panic([
                    Uuid::new_v4().into(),
                    p.number.into(),
                    p.verified.into(),
                    cust.id.into(),
                ]);
            }
        }

        InsertCustomerSttm { customer, phone }
    }
}
