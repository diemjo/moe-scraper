use crate::domain::melonbooks::models::artist::{Artist, ArtistArgs, FollowArtistError, GetArtistsError, UnfollowArtistError};
use crate::domain::melonbooks::models::availability::Availability;
use crate::domain::melonbooks::models::product::{CreateProductArgs, GetProductsError, Product, ScrapeProductsError, UpdateProductArgs};
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

    async fn unfollow_artist(&self, artist_args: &ArtistArgs) -> Result<(), UnfollowArtistError> {
        info!("unfollow artist '{}'", artist_args.name());
        self.repo.unfollow_melonbooks_artist(artist_args).await
    }

    async fn get_artists(&self) -> Result<Vec<Artist>, GetArtistsError> {
        info!("get artists");
        self.repo.get_melonbooks_artists().await
    }

    async fn get_products(&self) -> Result<Vec<Product>, GetProductsError> {
        info!("get products");
        self.repo.get_melonbooks_products().await
    }

    async fn scrape_available_products(&self) -> Result<(), ScrapeProductsError> {
        info!("scrape available products");
        let artists = self.get_artists().await?;
        for artist in artists.iter().filter(|a| a.following()).collect::<Vec<&Artist>>() {
            info!("scrape available products for '{}'", artist.name());
            let products = self.repo.get_melonbooks_products_by_artist(artist.id()).await?;
            let (available_products, unavailable_products) = products.iter()
                .partition::<Vec<_>, _>(|p| p.availability() == Availability::Available);
            let available_urls = available_products.iter().map(|p| p.url()).collect::<BTreeSet<_>>();
            let unavailable_urls = unavailable_products.iter().map(|p| p.url()).collect::<BTreeSet<_>>();
            let skip_urls = self.repo.get_melonbooks_skipping_urls().await?.into_iter().collect::<BTreeSet<_>>();
            let urls = self.scraper.get_potential_product_urls(artist.name()).await?
                .into_iter().filter(|u| !skip_urls.contains(u)).collect::<Vec<_>>();
            let (new_urls, restocked_urls) = urls.into_iter()
                .filter(|u| !available_urls.contains(u.as_str()))
                .partition::<Vec<_>, _>(|u| !unavailable_urls.contains(u.as_str()));
            let mut restocked_products = Vec::<Product>::new();
            for restocked_url in restocked_urls.iter() {
                let product = self.repo.update_melonbooks_product(&UpdateProductArgs::new(restocked_url.to_owned(), Availability::Available)).await?;
                restocked_products.push(product);
            }
            self.notifier.restocked_products(artist.name(), &restocked_products).await;
            let mut new_products = Vec::<Product>::new();
            for new_url in new_urls {
                let product_data = self.scraper.get_product(&new_url).await?;
                if product_data.artists().iter().all(|n| n != artist.name()) {
                    self.repo.add_melonbooks_skipping_url(&new_url, product_data.artists()).await?;
                    continue;
                }
                let args = CreateProductArgs::new_from_data(new_url, product_data);
                let product = self.repo.create_melonbooks_product(&args).await?;
                new_products.push(product);
            }
            self.notifier.new_products(artist.name(), &new_products).await;
        }
        Ok(())
    }
}