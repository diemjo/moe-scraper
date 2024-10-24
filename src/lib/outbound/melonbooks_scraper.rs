use crate::domain::melonbooks::models::product::{ProductData, ScrapeProductsError};
use crate::domain::melonbooks::ports::MelonbooksScraper;
use crate::outbound::melonbooks_scraper::parser::{parse_product_details, parse_product_list};
use anyhow::Context;
use log::info;
pub use parser::ParseError;
use reqwest::cookie::Jar;
use reqwest::{Client, Url};
use select::document::Document;
use std::sync::Arc;

mod parser;

const BASE_URL: &str = "https://www.melonbooks.co.jp";
const ARTIST_URL: &str = "https://www.melonbooks.co.jp/search/search.php?name={artist}&text_type=author&pageno={page}";
const PRODUCT_URL: &str = "https://www.melonbooks.co.jp{relative_url}";

#[derive(Debug, Clone)]
pub struct MelonbooksScraperImpl {
    client: Client,
}

impl MelonbooksScraperImpl {
    pub fn new() -> Result<Self, anyhow::Error> {
        let jar = Jar::default();
        jar.add_cookie_str("AUTH_ADULT=1", &BASE_URL.parse::<Url>()?);
        let client = Client::builder()
            //.use_rustls_tls()
            .cookie_provider(Arc::new(jar))
            .pool_max_idle_per_host(0)
            .build()
            .context("Failed to build MelonbooksScraper client")?;
        Ok(MelonbooksScraperImpl { client })
    }

    async fn get_product_list_page(&self, artist: &str, page_no: u32) -> Result<Document, reqwest::Error> {
        let url = ARTIST_URL
            .replace("{artist}", artist)
            .replace("{page}", page_no.to_string().as_str());
        let response = self.client.get(&url).send().await?;
        info!("request GET '{}' returned with status {}", url, response.status());
        let body = response.text().await?;
        let document = Document::from(body.as_str());
        Ok(document)
    }

    async fn get_product_list_urls(&self, artist: &str, page_no: u32) -> Result<Vec<String>, ScrapeProductsError> {
        let document = self.get_product_list_page(artist, page_no).await
            .with_context(|| format!("Error getting product urls for artist '{}'", artist))?;
        let urls = parse_product_list(document)?;
        info!("Found {} products on page {} for artist '{}'", urls.len(), page_no, artist);
        Ok(urls)
    }

    async fn get_product_urls(&self, artist: &str) -> Result<Vec<String>, ScrapeProductsError> {
        let mut page_no = 1_u32;
        let mut urls = Vec::<String>::new();
        loop {
            let page_urls = self.get_product_list_urls(artist, page_no).await?;
            let page_url_count = page_urls.len();
            urls.extend(page_urls);
            if page_url_count < 100 {
                break;
            }
            page_no += 1;
        }
        info!("Found {} total products for artist '{}'", urls.len(), artist);
        Ok(urls)
    }

    async fn get_product_page(&self, url: &str) -> Result<Document, reqwest::Error> {
        let response = self.client.get(url).send().await?;
        info!("request GET '{}' returned with status {}", url, response.status());
        let body = response.text().await?;
        let document = Document::from(body.as_str());
        Ok(document)
    }

    async fn get_product(&self, url: &str) -> Result<ProductData, ScrapeProductsError> {
        let document = self.get_product_page(&url).await
            .with_context(|| format!("Error getting product details for url '{}'", url))?;
        let product = parse_product_details(document)?;
        info!("Parsed product '{}' ({})", product.title(), url);
        Ok(product)
    }
}

impl MelonbooksScraper for MelonbooksScraperImpl {
    async fn get_potential_product_urls(&self, artist: &str) -> Result<Vec<String>, ScrapeProductsError> {
        self.get_product_urls(artist).await
    }

    async fn get_product(&self, url: &str) -> Result<ProductData, ScrapeProductsError>{
        self.get_product(&url).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_get_potential_product_urls() {
        env::set_var("OPENSSL_CONF", "./seclevel_1_openssl.conf");
        let scraper = MelonbooksScraperImpl::new().unwrap();
        let urls = scraper.get_potential_product_urls("まふゆ").await.unwrap();
        println!("{urls:?}");
    }

    #[tokio::test]
    async fn test_get_product() {
        env::set_var("OPENSSL_CONF", "./seclevel_1_openssl.conf");
        let scraper = MelonbooksScraperImpl::new().unwrap();
        let product = scraper.get_product("https://www.melonbooks.co.jp/detail/detail.php?product_id=2508959").await.unwrap();
        println!("{product:?}");
    }
}