use crate::domain::amiami::models::product::{CreateProductArgs, GetProductsError, Product, ScrapeProductsError, UpdateProductArgs};
use crate::domain::amiami::ports::{AmiamiNotifier, AmiamiRepository, AmiamiScraper, AmiamiService};
use log::info;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct AmiamiServiceImpl<R, N, S>
where
    R: AmiamiRepository,
    N: AmiamiNotifier,
    S: AmiamiScraper
{
    repo: R,
    notifier: N,
    scraper: S,
}

impl<R, N, S> AmiamiServiceImpl<R, N, S>
where
    R: AmiamiRepository,
    N: AmiamiNotifier,
    S: AmiamiScraper
{
    pub fn new(repo: R, notifier: N, scraper: S) -> Self {
        Self { repo, notifier, scraper }
    }
}

impl<R, N, S> AmiamiService for AmiamiServiceImpl<R, N, S>
where
    R: AmiamiRepository,
    N: AmiamiNotifier,
    S: AmiamiScraper
{
    async fn get_products(&self) -> Result<Vec<Product>, GetProductsError> {
        info!("get products");
        self.repo.get_amiami_products().await
    }

    async fn scrape_available_products(&self) -> Result<(), ScrapeProductsError> {
        info!("scrape available products");

        let products = self.repo.get_amiami_products().await?;
        let categories = self.repo.get_following_amiami_categories().await?;
        for category in categories.iter() {
            let (available_products, unavailable_products) = products.iter()
                .filter(|p| p.category() == category)
                .partition::<Vec<_>, _>(|p| p.availability().is_available());
            let available_urls = available_products.iter().map(|p| p.url()).collect::<BTreeSet<_>>();
            let unavailable_urls = unavailable_products.iter().map(|p| p.url()).collect::<BTreeSet<_>>();

            let current_product_data_list = self.scraper.get_products(category).await?;

            let (new_product_data_list, restocked_product_data_list) = current_product_data_list.into_iter()
                .filter(|p| !available_urls.contains(p.url()))
                .partition::<Vec<_>, _>(|p| !unavailable_urls.contains(p.url()));

            let mut restocked_products = Vec::<Product>::new();
            for restocked_product_data in restocked_product_data_list.into_iter() {
                let product = self.repo.update_amiami_product(&UpdateProductArgs::new(
                    restocked_product_data.url().to_owned(),
                    restocked_product_data.full_price(),
                    restocked_product_data.min_price(),
                    restocked_product_data.release_date(),
                    restocked_product_data.availability()
                )).await?;
                restocked_products.push(product);
            }
            info!("found '{}' restocked products for category '{}'", restocked_products.len(), category);
            self.notifier.restocked_products(category, &restocked_products).await;

            let mut new_products = Vec::<Product>::new();
            for product_data in new_product_data_list.into_iter() {
                let args = CreateProductArgs::new_from_data(product_data);
                let product = self.repo.create_amiami_product(&args).await?;
                new_products.push(product);
            }
            info!("found '{}' new products for category '{}'", new_products.len(), category);
            self.notifier.new_products(category, &new_products).await;
        }

        Ok(())
    }
}
