use crate::domain::melonbooks::models::artist::{Artist, ArtistArgs, FollowArtistError, GetArtistsError, UnfollowArtistError};
use crate::domain::melonbooks::models::product::{AddSkippingUrlError, AddTitleSkipSequenceError, CreateProductArgs, CreateProductError, DeleteTitleSkipSequenceError, GetProductsError, GetSkippingUrlsError, GetTitleSkipSequencesError, Product, ProductData, ScrapeProductsError, UpdateProductArgs, UpdateProductError};
use async_trait::async_trait;

#[async_trait]
pub trait MelonbooksService: Send + Sync + 'static {
    async fn follow_artist(&self, req: &ArtistArgs) -> Result<(), FollowArtistError>;
    async fn unfollow_artist(&self, artist_id: i32) -> Result<(), UnfollowArtistError>;
    async fn get_artists(&self) -> Result<Vec<Artist>, GetArtistsError>;
    async fn get_followed_artists(&self) -> Result<Vec<Artist>, GetArtistsError>;

    async fn get_products(&self) -> Result<Vec<Product>, GetProductsError>;
    async fn get_products_by_artist(&self, artist_id: i32) -> Result<Vec<Product>, GetProductsError>;

    async fn add_title_skip_sequence(&self, sequence: &str) -> Result<(), AddTitleSkipSequenceError>;
    async fn delete_title_skip_sequence(&self, sequence: &str) -> Result<(), DeleteTitleSkipSequenceError>;
    async fn get_title_skip_sequences(&self) -> Result<Vec<String>, GetTitleSkipSequencesError>;

    async fn scrape_available_products(&self) -> Result<(), ScrapeProductsError>;
}

#[async_trait]
pub trait MelonbooksRepository: Clone + Send + Sync + 'static {
    async fn follow_melonbooks_artist(&self, req: &ArtistArgs) -> Result<(), FollowArtistError>;
    async fn unfollow_melonbooks_artist(&self, artist_id: i32) -> Result<(), UnfollowArtistError>;
    async fn get_melonbooks_artists(&self) -> Result<Vec<Artist>, GetArtistsError>;

    async fn create_melonbooks_product(&self, req: &CreateProductArgs) -> Result<Product, CreateProductError>;
    async fn update_melonbooks_product(&self, req: &UpdateProductArgs) -> Result<Product, UpdateProductError>;
    async fn get_melonbooks_products(&self) -> Result<Vec<Product>, GetProductsError>;
    async fn get_melonbooks_products_by_artist(&self, artist_id: i32) -> Result<Vec<Product>, GetProductsError>;

    async fn add_melonbooks_skipping_url<S: AsRef<str> + Sync>(&self, url: &str, artists: &[S]) -> Result<(), AddSkippingUrlError>;
    async fn get_melonbooks_skipping_urls(&self) -> Result<Vec<String>, GetSkippingUrlsError>;

    async fn add_melonbooks_title_skip_sequence(&self, sequence: &str) -> Result<(), AddTitleSkipSequenceError>;
    async fn delete_melonbooks_title_skip_sequence(&self, sequence: &str) -> Result<(), DeleteTitleSkipSequenceError>;
    async fn get_melonbooks_title_skip_sequences(&self) -> Result<Vec<String>, GetTitleSkipSequencesError>;
}

#[async_trait]
pub trait MelonbooksNotifier: Clone + Send + Sync + 'static {
    async fn new_products<P: AsRef<Product> + Sync>(&self, artist: &str, products: &[P]) -> ();
    async fn restocked_products<P: AsRef<Product> + Sync>(&self, artist: &str, products: &[P],) -> ();
}

#[async_trait]
pub trait MelonbooksScraper: Clone + Send + Sync + 'static {
    async fn get_potential_product_urls(&self, artist: &str) -> Result<Vec<String>, ScrapeProductsError>;
    async fn get_product(&self, url: &str) -> Result<ProductData, ScrapeProductsError>;
}
