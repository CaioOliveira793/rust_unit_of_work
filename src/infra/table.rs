use sea_query::Iden;

#[derive(Iden)]
#[iden = "clientes"]
pub enum TCustomer {
    Table,
    #[iden = "id"]
    Id,
    #[iden = "nome"]
    Name,
    #[iden = "cpf"]
    Cpf,
    #[iden = "criado_em"]
    Created,
    #[iden = "atualizado_em"]
    Updated,
}

#[derive(Iden)]
#[iden = "telefones_cliente"]
pub enum TCustomerPhone {
    Table,
    #[iden = "id"]
    Id,
    #[iden = "numero"]
    Number,
    #[iden = "whatsapp"]
    Whatsapp,
    #[iden = "id_cliente"]
    CustomerId,
}
