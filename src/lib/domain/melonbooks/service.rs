use crate::domain::melonbooks::models::artist::{Artist, ArtistArgs, FollowArtistError, GetArtistsError, UnfollowArtistError};
use crate::domain::melonbooks::models::availability::Availability;
use crate::domain::melonbooks::models::product::{AddTitleSkipSequenceError, CreateProductArgs, DeleteTitleSkipSequenceError, GetProductsError, GetTitleSkipSequencesError, Product, ScrapeProductsError, UpdateProductArgs};
use crate::domain::melonbooks::ports::{MelonbooksNotifier, MelonbooksRepository, MelonbooksScraper, MelonbooksService};
use log::info;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct MelonbooksServiceImpl<R, N, S>
where
    R: MelonbooksRepository,
    N: MelonbooksNotifier,
    S: MelonbooksScraper
{
    repo: R,
    notifier: N,
    scraper: S,
}

impl<R, N, S> MelonbooksServiceImpl<R, N, S>
where
    R: MelonbooksRepository,
    N: MelonbooksNotifier,
    S: MelonbooksScraper
{
    pub fn new(repo: R, notifier: N, scraper: S) -> Self {
        Self { repo, notifier, scraper }
    }
}

impl<R, N, S> MelonbooksService for MelonbooksServiceImpl<R, N, S>
where
    R: MelonbooksRepository,
    N: MelonbooksNotifier,
    S: MelonbooksScraper
{
    async fn follow_artist(&self, artist_args: &ArtistArgs) -> Result<(), FollowArtistError> {
        info!("follow artist '{}'", artist_args.name());
        self.repo.follow_melonbooks_artist(artist_args).await
    }

    async fn unfollow_artist(&self, artist_id: i32) -> Result<(), UnfollowArtistError> {
        info!("unfollow artist with id '{}'", artist_id);
        self.repo.unfollow_melonbooks_artist(artist_id).await
    }

    async fn get_artists(&self) -> Result<Vec<Artist>, GetArtistsError> {
        info!("get artists");
        self.repo.get_melonbooks_artists().await
    }

    async fn get_followed_artists(&self) -> Result<Vec<Artist>, GetArtistsError> {
        info!("get followed artists");
        let artists = self.repo.get_melonbooks_artists().await?;
        Ok(
            artists.into_iter()
                .filter(|a| a.following())
                .collect()
        )
    }

    async fn get_products(&self) -> Result<Vec<Product>, GetProductsError> {
        info!("get products");
        self.repo.get_melonbooks_products().await
    }

    async fn get_products_by_artist(&self, artist_id: i32) -> Result<Vec<Product>, GetProductsError> {
        info!("get products by artist with id '{}'", artist_id);
        self.repo.get_melonbooks_products_by_artist(artist_id).await
    }

    async fn get_title_skip_sequences(&self) -> Result<Vec<String>, GetTitleSkipSequencesError> {
        info!("get title skip sequences");
        self.repo.get_melonbooks_title_skip_sequences().await
    }

    async fn add_title_skip_sequence(&self, sequence: &str) -> Result<(), AddTitleSkipSequenceError> {
        info!("add title skip sequences");
        self.repo.add_melonbooks_title_skip_sequence(sequence).await
    }

    async fn delete_title_skip_sequence(&self, sequence: &str) -> Result<(), DeleteTitleSkipSequenceError> {
        info!("delete title skip sequences");
        self.repo.delete_melonbooks_title_skip_sequence(sequence).await
    }

    async fn scrape_available_products(&self) -> Result<(), ScrapeProductsError> {
        info!("scrape available products");
        let artists = self.get_artists().await?;
        for artist in artists.iter().filter(|a| a.following()).collect::<Vec<&Artist>>() {
            info!("scrape available products for '{}'", artist.name());
            let products = self.repo.get_melonbooks_products_by_artist(artist.id()).await?;
            let (available_products, unavailable_products) = products.iter()
                .partition::<Vec<_>, _>(|p| p.availability().is_available());
            let available_urls = available_products.iter().map(|p| p.url()).collect::<BTreeSet<_>>();
            let unavailable_urls = unavailable_products.iter().map(|p| p.url()).collect::<BTreeSet<_>>();
            let skip_urls = self.repo.get_melonbooks_skipping_urls().await?.into_iter().collect::<BTreeSet<_>>();
            let urls = self.scraper.get_potential_product_urls(artist.name()).await?
                .into_iter().filter(|u| !skip_urls.contains(u)).collect::<Vec<_>>();
            let (new_urls, restocked_urls) = urls.iter()
                .filter(|u| !available_urls.contains(u.as_str()))
                .partition::<Vec<_>, _>(|u| !unavailable_urls.contains(u.as_str()));
            let title_skip_sequences = self.repo.get_melonbooks_title_skip_sequences().await?;
            let mut restocked_products = Vec::<Product>::new();
            for restocked_url in restocked_urls.into_iter() {
                let product = self.repo.update_melonbooks_product(&UpdateProductArgs::new(restocked_url.to_owned(), Availability::Available)).await?;
                if title_skip_sequences.iter().all(|s| !product.title().contains(s)) {
                    restocked_products.push(product);
                }
            }
            self.notifier.restocked_products(artist.name(), &restocked_products).await;
            let mut new_products = Vec::<Product>::new();
            for new_url in new_urls.into_iter() {
                let product_data = self.scraper.get_product(new_url).await?;
                if product_data.artists().iter().all(|n| n != artist.name()) {
                    self.repo.add_melonbooks_skipping_url(new_url, product_data.artists()).await?;
                    continue;
                }
                if title_skip_sequences.iter().all(|s| !product_data.title().contains(s)) {
                    let args = CreateProductArgs::new_from_data(new_url.to_owned(), product_data);
                    let product = self.repo.create_melonbooks_product(&args).await?;
                    new_products.push(product);
                }
            }
            self.notifier.new_products(artist.name(), &new_products).await;
            let newly_unavailable_products = available_products.iter()
                .filter(|p| !urls.iter().any(|u| u.eq(p.url())))
                .collect::<Vec<_>>();
            for newly_unavailable in newly_unavailable_products.into_iter() {
                self.repo.update_melonbooks_product(&UpdateProductArgs::new(newly_unavailable.url().to_owned(), Availability::NotAvailable)).await?;
            }
        }
        Ok(())
    }
}