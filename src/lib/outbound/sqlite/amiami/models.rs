use crate::domain::amiami::models::availability::Availability;
use crate::outbound::sqlite::schema;
use chrono::NaiveDateTime;
use diesel::{AsChangeset, ExpressionMethods, Identifiable, Insertable, Queryable, Selectable};

#[derive(Debug, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::amiami_product)]
#[diesel(treat_none_as_null = true)]
pub struct ProductRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub url: String,
    pub title: String,
    pub image_url: String,
    pub category_id: i32,
    pub maker: String,
    pub full_price: i32,
    pub min_price: i32,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub availability: Availability,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::amiami_product)]
#[diesel(treat_none_as_null = true)]
pub struct ProductRowInsert<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub image_url: &'a str,
    pub category_id: i32,
    pub maker: &'a str,
    pub full_price: i32,
    pub min_price: i32,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub availability: Availability,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::amiami_category)]
#[diesel(treat_none_as_null = true)]
pub struct CategoryRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub category: String,
    pub following: bool
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::amiami_category)]
#[diesel(treat_none_as_null = true)]
pub struct CategoryRowInsert<'a> {
    pub category: &'a str,
    pub following: bool
}
