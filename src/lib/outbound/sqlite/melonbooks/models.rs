use crate::domain::melonbooks::models::artist::Artist;
use crate::domain::melonbooks::models::availability::Availability;
use crate::domain::melonbooks::models::product::Product;
use crate::outbound::sqlite::schema;
use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};

#[derive(Debug, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::melonbooks_product)]
#[diesel(treat_none_as_null = true)]
pub struct ProductRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub url: String,
    pub title: String,
    pub circle: Option<String>,
    pub image_url: String,
    pub category_id: i32,
    pub price: Option<String>,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub availability: Availability,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_product)]
#[diesel(treat_none_as_null = true)]
pub struct ProductRowInsert<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub circle: Option<&'a str>,
    pub image_url: &'a str,
    pub category_id: i32,
    pub price: Option<&'a str>,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub availability: Availability,
}

#[derive(Debug, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::melonbooks_artist)]
#[diesel(treat_none_as_null = true)]
pub struct ArtistRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub name: String,
    pub following: bool,
    pub date_followed: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_artist)]
#[diesel(treat_none_as_null = true)]
pub struct ArtistRowInsert<'a> {
    pub name: &'a str,
    pub following: bool,
    pub date_followed: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::melonbooks_category)]
#[diesel(treat_none_as_null = true)]
pub struct CategoryRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub category: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_category)]
#[diesel(treat_none_as_null = true)]
pub struct CategoryRowInsert<'a> {
    pub category: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::melonbooks_tag)]
#[diesel(treat_none_as_null = true)]
pub struct TagRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub tag: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_tag)]
#[diesel(treat_none_as_null = true)]
pub struct TagRowInsert<'a> {
    pub tag: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::melonbooks_flag)]
#[diesel(treat_none_as_null = true)]
pub struct FlagRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub flag: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_flag)]
#[diesel(treat_none_as_null = true)]
pub struct FlagRowInsert<'a> {
    pub flag: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::melonbooks_skip_product)]
#[diesel(treat_none_as_null = true)]
pub struct SkipProductRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub url: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_skip_product)]
#[diesel(treat_none_as_null = true)]
pub struct SkipProductRowInsert<'a> {
    pub url: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::melonbooks_skip_product_artist)]
#[diesel(treat_none_as_null = true)]
pub struct SkipProductArtistRow {
    pub skip_product_id: i32,
    pub artist_name: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_skip_product_artist)]
#[diesel(treat_none_as_null = true)]
pub struct SkipProductArtistRowInsert<'a> {
    pub skip_product_id: i32,
    pub artist_name: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::melonbooks_title_skip_sequence)]
#[diesel(treat_none_as_null = true)]
pub struct TitleSkipSequenceRow {
    pub id: i32,
    pub date_added: NaiveDateTime,
    pub sequence: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::melonbooks_title_skip_sequence)]
#[diesel(treat_none_as_null = true)]
pub struct TitleSkipSequenceRowInsert<'a> {
    pub sequence: &'a str,
}

impl ArtistRow {
    pub fn into_domain(self) -> Artist {
        Artist::new(self.id, self.date_added.and_utc(), self.name, self.following, self.date_followed.map(|d| d.and_utc()))
    }
}

impl ProductRow {
    pub fn into_domain(self, artists: Vec<Artist>, category: String, tags: Vec<String>, flags: Vec<String>) -> Product {
        Product::new(self.id, self.date_added.and_utc(), self.url, self.title, self.circle, artists, self.image_url, category, tags, flags, self.price, self.availability)
    }
}

impl TagRow {
    pub fn into_domain(self) -> String {
        self.tag
    }
}

impl FlagRow {
    pub fn into_domain(self) -> String {
        self.flag
    }
}