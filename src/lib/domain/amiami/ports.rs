use crate::domain::amiami::models::product::{CreateProductArgs, CreateProductError, GetCategoriesError, GetProductsError, Product, ProductData, ScrapeProductsError, UpdateProductArgs, UpdateProductError};
use std::future::Future;

pub trait AmiamiService: Clone + Send + Sync + 'static {
    fn get_products(&self) -> impl Future<Output = Result<Vec<Product>, GetProductsError>> + Send;
    fn scrape_available_products(&self) -> impl Future<Output = Result<(), ScrapeProductsError>> + Send;
}

pub trait AmiamiRepository: Clone + Send + Sync + 'static {
    fn create_amiami_product(&self, req: &CreateProductArgs) -> impl Future<Output = Result<Product, CreateProductError>> + Send;
    fn update_amiami_product(&self, req: &UpdateProductArgs) -> impl Future<Output = Result<Product, UpdateProductError>> + Send;
    fn get_amiami_products(&self) -> impl Future<Output = Result<Vec<Product>, GetProductsError>> + Send;
    fn get_following_amiami_categories(&self) -> impl Future<Output = Result<Vec<String>, GetCategoriesError>> + Send;
}

pub trait AmiamiNotifier: Clone + Send + Sync + 'static {
    fn new_products<P: AsRef<Product> + Sync>(&self, category: &str, products: &[P]) -> impl Future<Output = ()> + Send;
    fn restocked_products<P: AsRef<Product> + Sync>(&self, category: &str, products: &[P]) -> impl Future<Output = ()> + Send;
}

pub trait AmiamiScraper: Clone + Send + Sync + 'static {
    fn get_products(&self, category: &str) -> impl Future<Output = Result<Vec<ProductData>, ScrapeProductsError>> + Send;
}