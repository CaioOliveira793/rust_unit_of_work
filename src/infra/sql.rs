use sea_query::{Alias, Expr, InsertStatement, Query, SelectStatement};
use uuid::Uuid;

use crate::domain::entity::User;
use table::*;

mod table {
    use sea_query::Iden;

    #[derive(Iden)]
    #[iden = "user"]
    pub enum TUser {
        Table,
        #[iden = "id"]
        Id,
        #[iden = "name"]
        Name,
        #[iden = "email"]
        Email,
        #[iden = "created"]
        Created,
        #[iden = "updated"]
        Updated,
    }

    #[derive(Iden)]
    #[iden = "user_phone"]
    pub enum TUserPhone {
        Table,
        #[iden = "id"]
        Id,
        #[iden = "number"]
        Number,
        #[iden = "verified"]
        Verified,
        #[iden = "user_id"]
        UserId,
    }
}

pub struct SelectUserSttm {
    pub user: SelectStatement,
    pub phone: SelectStatement,
}

pub struct InsertUserSttm {
    pub user: InsertStatement,
    pub phone: InsertStatement,
}

pub fn select_user_by_id(id: &Uuid) -> SelectUserSttm {
    let mut user = Query::select();
    user.from(TUser::Table)
        .expr_as(Expr::tbl(TUser::Table, TUser::Id), Alias::new("id"))
        .expr_as(Expr::col(TUser::Name), Alias::new("name"))
        .expr_as(Expr::col(TUser::Email), Alias::new("email"))
        .expr_as(Expr::col(TUser::Created), Alias::new("created"))
        .expr_as(Expr::col(TUser::Updated), Alias::new("updated"))
        .and_where(Expr::col(TUser::Id).eq(*id));

    let mut phone = Query::select();
    phone
        .from(TUserPhone::Table)
        .expr_as(
            Expr::tbl(TUserPhone::Table, TUserPhone::Id),
            Alias::new("id"),
        )
        .expr_as(Expr::col(TUserPhone::UserId), Alias::new("user_id"))
        .expr_as(Expr::col(TUserPhone::Number), Alias::new("number"))
        .expr_as(Expr::col(TUserPhone::Verified), Alias::new("verified"))
        .and_where(Expr::col(TUserPhone::UserId).eq(*id));

    SelectUserSttm { user, phone }
}

pub fn insert_user<I>(users: I) -> InsertUserSttm
where
    I: IntoIterator<Item = User>,
{
    let mut user = Query::insert();
    user.into_table(TUser::Table);
    user.columns([
        TUser::Id,
        TUser::Email,
        TUser::Name,
        TUser::Created,
        TUser::Updated,
    ]);

    let mut phone = Query::insert();
    phone.into_table(TUserPhone::Table);
    phone.columns([
        TUserPhone::Id,
        TUserPhone::Number,
        TUserPhone::Verified,
        TUserPhone::UserId,
    ]);

    for us in users {
        user.values_panic([
            us.id.into(),
            us.email.into(),
            us.name.into(),
            us.created.into(),
            us.updated.into(),
        ]);

        for p in us.phones {
            phone.values_panic([
                Uuid::new_v4().into(),
                p.number.into(),
                p.verified.into(),
                us.id.into(),
            ]);
        }
    }

    InsertUserSttm { user, phone }
}
