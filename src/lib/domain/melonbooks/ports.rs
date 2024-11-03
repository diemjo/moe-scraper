use crate::domain::melonbooks::models::artist::{Artist, ArtistArgs, FollowArtistError, GetArtistsError, UnfollowArtistError};
use crate::domain::melonbooks::models::product::{AddSkippingUrlError, AddTitleSkipSequenceError, CreateProductArgs, CreateProductError, DeleteTitleSkipSequenceError, GetProductsError, GetSkippingUrlsError, GetTitleSkipSequencesError, Product, ProductData, ScrapeProductsError, UpdateProductArgs, UpdateProductError};
use std::future::Future;

pub trait MelonbooksService: Clone + Send + Sync + 'static {
    fn follow_artist(&self, req: &ArtistArgs) -> impl Future<Output = Result<(), FollowArtistError>> + Send;
    fn unfollow_artist(&self, artist_id: i32) -> impl Future<Output = Result<(), UnfollowArtistError>> + Send;
    fn get_artists(&self) -> impl Future<Output = Result<Vec<Artist>, GetArtistsError>> + Send;
    fn get_followed_artists(&self) -> impl Future<Output = Result<Vec<Artist>, GetArtistsError>> + Send;

    fn get_products(&self) -> impl Future<Output = Result<Vec<Product>, GetProductsError>> + Send;
    fn get_products_by_artist(&self, artist_id: i32) -> impl Future<Output = Result<Vec<Product>, GetProductsError>> + Send;

    fn add_title_skip_sequence(&self, sequence: &str) -> impl Future<Output = Result<(), AddTitleSkipSequenceError>> + Send;
    fn delete_title_skip_sequence(&self, sequence: &str) -> impl Future<Output = Result<(), DeleteTitleSkipSequenceError>> + Send;
    fn get_title_skip_sequences(&self) -> impl Future<Output = Result<Vec<String>, GetTitleSkipSequencesError>> + Send;
    
    fn scrape_available_products(&self) -> impl Future<Output = Result<(), ScrapeProductsError>> + Send;
}

pub trait MelonbooksRepository: Clone + Send + Sync + 'static {
    fn follow_melonbooks_artist(&self, req: &ArtistArgs) -> impl Future<Output = Result<(), FollowArtistError>> + Send;
    fn unfollow_melonbooks_artist(&self, artist_id: i32) -> impl Future<Output = Result<(), UnfollowArtistError>> + Send;
    fn get_melonbooks_artists(&self) -> impl Future<Output = Result<Vec<Artist>, GetArtistsError>> + Send;

    fn create_melonbooks_product(&self, req: &CreateProductArgs) -> impl Future<Output = Result<Product, CreateProductError>> + Send;
    fn update_melonbooks_product(&self, req: &UpdateProductArgs) -> impl Future<Output = Result<Product, UpdateProductError>> + Send;
    fn get_melonbooks_products(&self) -> impl Future<Output = Result<Vec<Product>, GetProductsError>> + Send;
    fn get_melonbooks_products_by_artist(&self, artist_id: i32) -> impl Future<Output = Result<Vec<Product>, GetProductsError>> + Send;

    fn add_melonbooks_skipping_url<S: AsRef<str> + Sync>(&self, url: &str, artists: &[S]) -> impl Future<Output = Result<(), AddSkippingUrlError>> + Send;
    fn get_melonbooks_skipping_urls(&self) -> impl Future<Output = Result<Vec<String>, GetSkippingUrlsError>> + Send;

    fn add_melonbooks_title_skip_sequence(&self, sequence: &str) -> impl Future<Output = Result<(), AddTitleSkipSequenceError>> + Send;
    fn delete_melonbooks_title_skip_sequence(&self, sequence: &str) -> impl Future<Output = Result<(), DeleteTitleSkipSequenceError>> + Send;
    fn get_melonbooks_title_skip_sequences(&self) -> impl Future<Output = Result<Vec<String>, GetTitleSkipSequencesError>> + Send;
}

pub trait MelonbooksNotifier: Clone + Send + Sync + 'static {
    fn new_products<P: AsRef<Product> + Sync>(&self, artist: &str, products: &[P]) -> impl Future<Output = ()> + Send;
    fn restocked_products<P: AsRef<Product> + Sync>(&self, artist: &str, products: &[P]) -> impl Future<Output = ()> + Send;
}

pub trait MelonbooksScraper: Clone + Send + Sync + 'static {
    fn get_potential_product_urls(&self, artist: &str) -> impl Future<Output = Result<Vec<String>, ScrapeProductsError>> + Send;
    fn get_product(&self, url: &str) -> impl Future<Output = Result<ProductData, ScrapeProductsError>> + Send;
}