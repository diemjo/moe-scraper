use crate::domain::melonbooks::models::artist::{Artist, GetArtistsError};
use crate::domain::melonbooks::models::availability::Availability;
use crate::outbound::melonbooks_scraper::ParseError;
use chrono::{DateTime, Utc};
use derivative::Derivative;
use thiserror::Error;

#[derive(Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct Product {
    id: i32,
    date_added: DateTime<Utc>,
    url: String,
    title: String,
    circle: Option<String>,
    artists: Vec<Artist>,
    image_url: String,
    category: String,
    tags: Vec<String>,
    flags: Vec<String>,
    price: Option<String>,
    availability: Availability,
}

impl Product {
    pub fn new(id: i32, date_added: DateTime<Utc>, url: String, title: String, circle: Option<String>, artists: Vec<Artist>, image_url: String, category: String, tags: Vec<String>, flags: Vec<String>, price: Option<String>, availability: Availability) -> Self {
        Self { id, date_added, url, title, circle, artists, image_url, category, tags, flags, price, availability }
    }

    pub fn id(&self) -> i32 { self.id }
    pub fn date_added(&self) -> DateTime<Utc> { self.date_added.clone() }
    pub fn url(&self) -> &str { &self.url }
    pub fn title(&self) -> &str { &self.title }
    pub fn circle(&self) -> Option<&str> { self.circle.as_deref() }
    pub fn artists(&self) -> &[Artist] { &self.artists }
    pub fn image_url(&self) -> &str { &self.image_url }
    pub fn category(&self) -> &str { &self.category }
    pub fn tags(&self) -> &[String] { &self.tags }
    pub fn flags(&self) -> &[String] { &self.flags }
    pub fn price(&self) -> Option<&str> { self.price.as_deref() }
    pub fn availability(&self) -> Availability { self.availability.clone() }
}

impl AsRef<Product> for Product {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug)]
pub struct ProductData {
    title: String,
    circle: Option<String>,
    artists: Vec<String>,
    image_url: String,
    category: String,
    tags: Vec<String>,
    flags: Vec<String>,
    price: Option<String>,
    availability: Availability,
}

impl ProductData {
    pub fn new(title: String, circle: Option<String>, artists: Vec<String>, image_url: String, category: String, tags: Vec<String>, flags: Vec<String>, price: Option<String>, availability: Availability) -> Self {
        Self { title, circle, artists, image_url, category, tags, flags, price, availability }
    }

    pub fn title(&self) -> &str { &self.title }
    pub fn circle(&self) -> Option<&String> { self.circle.as_ref() }
    pub fn artists(&self) -> &[String] { &self.artists }
    pub fn image_url(&self) -> &str { &self.image_url }
    pub fn category(&self) -> &str { &self.category }
    pub fn tags(&self) -> &[String] { &self.tags }
    pub fn flags(&self) -> &[String] { &self.flags }
    pub fn price(&self) -> Option<&String> { self.price.as_ref() }
    pub fn availability(&self) -> &Availability { &self.availability }
}

#[derive(Debug)]
pub struct CreateProductArgs {
    url: String,
    title: String,
    circle: Option<String>,
    artists: Vec<String>,
    image_url: String,
    category: String,
    tags: Vec<String>,
    flags: Vec<String>,
    price: Option<String>,
    availability: Availability,
}

impl CreateProductArgs {
    pub fn new(url: String, title: String, circle: Option<String>, artists: Vec<String>, image_url: String, category: String, tags: Vec<String>, flags: Vec<String>, price: Option<String>, availability: Availability) -> Self {
        Self { url, title, circle, artists, image_url, category, tags, flags, price, availability }
    }
    
    pub fn new_from_data(url: String, data: ProductData) -> Self {
        Self::new(url, data.title, data.circle, data.artists, data.image_url, data.category, data.tags, data.flags, data.price, data.availability)
    }
    
    pub fn url(&self) -> &str { &self.url }
    pub fn title(&self) -> &str { &self.title }
    pub fn circle(&self) -> Option<&str> { self.circle.as_deref() }
    pub fn artists(&self) -> &[String] { &self.artists }
    pub fn image_url(&self) -> &str { &self.image_url }
    pub fn category(&self) -> &str { &self.category }
    pub fn tags(&self) -> &[String] { &self.tags }
    pub fn flags(&self) -> &[String] { &self.flags }
    pub fn price(&self) -> Option<&str> { self.price.as_deref() }
    pub fn availability(&self) -> Availability { self.availability.clone() }
}

impl AsRef<CreateProductArgs> for CreateProductArgs {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug)]
pub struct UpdateProductArgs {
    url: String,
    availability: Availability,
}

impl UpdateProductArgs {
    pub fn new(url: String, availability: Availability) -> Self {
        Self { url, availability }
    }

    pub fn url(&self) -> &str { &self.url }

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
pub enum GetSkippingUrlsError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum AddSkippingUrlError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum AddTitleSkippSequenceError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum DeleteTitleSkippSequenceError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetTitleSkippSequencesError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ScrapeProductsError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    GetArtistsError(#[from] GetArtistsError),
    #[error(transparent)]
    GetProductError(#[from] GetProductsError),
    #[error(transparent)]
    CreateProductError(#[from] CreateProductError),
    #[error(transparent)]
    GetSkippingUrlsError(#[from] GetSkippingUrlsError),
    #[error(transparent)]
    UpdateProductError(#[from] UpdateProductError),
    #[error(transparent)]
    AddSkippingUrlError(#[from] AddSkippingUrlError),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}