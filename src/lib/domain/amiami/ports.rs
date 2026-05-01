use crate::domain::amiami::models::product::{CreateProductArgs, CreateProductError, GetCategoriesError, GetProductsError, Product, ProductData, ScrapeProductsError, UpdateProductArgs, UpdateProductError};
use async_trait::async_trait;

#[async_trait]
pub trait AmiamiService: Send + Sync + 'static {
    async fn get_products(&self) -> Result<Vec<Product>, GetProductsError>;
    async fn scrape_available_products(&self) -> Result<(), ScrapeProductsError>;
}

#[async_trait]
pub trait AmiamiRepository: Clone + Send + Sync + 'static {
    async fn create_amiami_product(&self, req: &CreateProductArgs) -> Result<Product, CreateProductError>;
    async fn update_amiami_product(&self, req: &UpdateProductArgs, ) -> Result<Product, UpdateProductError>;
    async fn get_amiami_products(&self) -> Result<Vec<Product>, GetProductsError>;
    async fn get_following_amiami_categories(&self) -> Result<Vec<String>, GetCategoriesError>;
}

#[async_trait]
pub trait AmiamiNotifier: Clone + Send + Sync + 'static {
    async fn new_products<P: AsRef<Product> + Sync>(&self, category: &str, products: &[P]);
    async fn restocked_products<P: AsRef<Product> + Sync>(&self, category: &str, products: &[P]);
}

#[async_trait]
pub trait AmiamiScraper: Clone + Send + Sync + 'static {
    async fn get_products(&self, category: &str) -> Result<Vec<ProductData>, ScrapeProductsError>;
}
