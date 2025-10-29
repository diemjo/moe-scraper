use crate::domain::amiami::models::availability::Availability;
use crate::outbound::amiami_scraper::parser::ParseError;
use chrono::{DateTime, NaiveDate, Utc};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Product {
    id: i32,
    date_added: DateTime<Utc>,
    url: String,
    title: String,
    image_url: String,
    category: String,
    maker: String,
    full_price: i32,
    min_price: i32,
    release_date: NaiveDate,
    availability: Availability,
}

impl Product {
    pub fn new(id: i32, date_added: DateTime<Utc>, url: String, title: String, image_url: String, category: String, maker: String, full_price: i32, min_price: i32, release_date: NaiveDate, availability: Availability) -> Self {
        Self { id, date_added, url, title, image_url, category, maker, full_price, min_price, release_date, availability }
    }

    pub fn id(&self) -> i32 { self.id }
    pub fn date_added(&self) -> DateTime<Utc> { self.date_added.clone() }
    pub fn url(&self) -> &str { &self.url }
    pub fn title(&self) -> &str { &self.title }
    pub fn image_url(&self) -> &str { &self.image_url }
    pub fn category(&self) -> &str { &self.category }
    pub fn maker(&self) -> &str { &self.maker }
    pub fn full_price(&self) -> i32 { self.full_price }
    pub fn min_price(&self) -> i32 { self.min_price }
    pub fn release_date(&self) -> NaiveDate { self.release_date }
    pub fn availability(&self) -> Availability { self.availability.clone() }
}

impl AsRef<Product> for Product {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug)]
pub struct ProductData {
    url: String,
    title: String,
    image_url: String,
    category: String,
    maker: String,
    full_price: i32,
    min_price: i32,
    release_date: NaiveDate,
    availability: Availability,
}

impl ProductData {
    pub fn new(url: String, title: String, image_url: String, category: String, maker: String, full_price: i32, min_price: i32, release_date: NaiveDate, availability: Availability) -> Self {
        Self { url, title, image_url, category, maker, full_price, min_price, release_date, availability }
    }

    pub fn url(&self) -> &str { &self.url }
    pub fn title(&self) -> &str { &self.title }
    pub fn image_url(&self) -> &str { &self.image_url }
    pub fn category(&self) -> &str { &self.category }
    pub fn maker(&self) -> &str { &self.maker }
    pub fn full_price(&self) -> i32 { self.full_price }
    pub fn min_price(&self) -> i32 { self.min_price }
    pub fn release_date(&self) -> NaiveDate { self.release_date }
    pub fn availability(&self) -> Availability { self.availability.to_owned() }
}

#[derive(Debug)]
pub struct CreateProductArgs {
    url: String,
    title: String,
    image_url: String,
    category: String,
    maker: String,
    full_price: i32,
    min_price: i32,
    release_date: NaiveDate,
    availability: Availability,
}

impl CreateProductArgs {
    pub fn new(url: String, title: String, image_url: String, category: String, maker: String, full_price: i32, min_price: i32, release_date: NaiveDate, availability: Availability) -> Self {
        Self { url, title, image_url, category, maker, full_price, min_price, release_date, availability }
    }
    
    pub fn new_from_data(data: ProductData) -> Self {
        Self::new(data.url, data.title, data.image_url, data.category, data.maker, data.full_price, data.min_price, data.release_date, data.availability)
    }

    pub fn url(&self) -> &str { &self.url }
    pub fn title(&self) -> &str { &self.title }
    pub fn image_url(&self) -> &str { &self.image_url }
    pub fn category(&self) -> &str { &self.category }
    pub fn maker(&self) -> &str { &self.maker }
    pub fn full_price(&self) -> i32 { self.full_price }
    pub fn min_price(&self) -> i32 { self.min_price }
    pub fn release_date(&self) -> NaiveDate { self.release_date }
    pub fn availability(&self) -> Availability { self.availability.to_owned() }
}

impl AsRef<CreateProductArgs> for CreateProductArgs {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug)]
pub struct UpdateProductArgs {
    url: String,
    full_price: i32,
    min_price: i32,
    release_date: NaiveDate,
    availability: Availability,
}

impl UpdateProductArgs {
    pub fn new(url: String, full_price: i32, min_price: i32, release_date: NaiveDate, availability: Availability) -> Self {
        Self { url, full_price, min_price, release_date, availability }
    }

    pub fn url(&self) -> &str { &self.url }

    pub fn full_price(&self) -> i32 { self.full_price }

    pub fn min_price(&self) -> i32 { self.min_price }

    pub fn release_date(&self) -> NaiveDate { self.release_date }

    pub fn availability(&self) -> Availability { self.availability.clone() }
}

impl AsRef<UpdateProductArgs> for UpdateProductArgs {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug, Error)]
pub enum CreateProductError {
    #[error("Product '{title}' ({url}) already exists")]
    DuplicateProduct { url: String, title: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UpdateProductError {
    #[error("Product {url} does not exist")]
    ProductMissing { url: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetProductsError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetCategoriesError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ScrapeProductsError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    GetProductError(#[from] GetProductsError),
    #[error(transparent)]
    CreateProductError(#[from] CreateProductError),
    #[error(transparent)]
    UpdateProductError(#[from] UpdateProductError),
    #[error(transparent)]
    GetCategoriesError(#[from] GetCategoriesError),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}