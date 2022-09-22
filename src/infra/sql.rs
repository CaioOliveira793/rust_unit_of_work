pub mod customer {
    use sea_query::{Alias, Expr, Query, SelectStatement};
    use uuid::Uuid;

    use crate::infra::table::{TCustomer, TCustomerPhone};

    pub struct SelectCustomerSttm {
        pub customer: SelectStatement,
        pub phone: SelectStatement,
    }

    pub fn select_customer_by_import_id(id: &Uuid) -> SelectCustomerSttm {
        let mut customer = Query::select();
        customer
            .from(TCustomer::Table)
            .expr_as(Expr::tbl(TCustomer::Table, TCustomer::Id), Alias::new("id"))
            .expr_as(Expr::col(TCustomer::Name), Alias::new("name"))
            .expr_as(Expr::col(TCustomer::Cpf), Alias::new("cpf"))
            .expr_as(
                Expr::col(TCustomer::CustomerImportId),
                Alias::new("import_id"),
            )
            .expr_as(
                Expr::col(TCustomer::CustomerImportDocumentId),
                Alias::new("import_document_id"),
            )
            .expr_as(Expr::col(TCustomer::Kind), Alias::new("kind"))
            .expr_as(Expr::col(TCustomer::Relation), Alias::new("relation"))
            .expr_as(Expr::col(TCustomer::Created), Alias::new("created"))
            .expr_as(Expr::col(TCustomer::Updated), Alias::new("updated"))
            .and_where(Expr::col(TCustomer::CustomerImportId).eq(*id));

        let mut phone = Query::select();
        phone
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
            .inner_join(
                TCustomer::Table,
                Expr::tbl(TCustomerPhone::Table, TCustomerPhone::CustomerId)
                    .equals(TCustomer::Table, TCustomer::Id),
            )
            .and_where(Expr::col(TCustomer::CustomerImportId).eq(*id));

        SelectCustomerSttm { customer, phone }
    }

    pub fn select_customer_by_id(id: &Uuid) -> SelectCustomerSttm {
        let mut customer_sttm = Query::select();
        customer_sttm
            .from(TCustomer::Table)
            .expr_as(Expr::tbl(TCustomer::Table, TCustomer::Id), Alias::new("id"))
            .expr_as(Expr::col(TCustomer::Name), Alias::new("name"))
            .expr_as(Expr::col(TCustomer::Cpf), Alias::new("cpf"))
            .expr_as(
                Expr::col(TCustomer::CustomerImportId),
                Alias::new("import_id"),
            )
            .expr_as(
                Expr::col(TCustomer::CustomerImportDocumentId),
                Alias::new("import_document_id"),
            )
            .expr_as(Expr::col(TCustomer::Kind), Alias::new("kind"))
            .expr_as(Expr::col(TCustomer::Relation), Alias::new("relation"))
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
}
