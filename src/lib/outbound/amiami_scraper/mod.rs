use crate::domain::amiami::models::product::{ProductData, ScrapeProductsError};
use crate::domain::amiami::ports::AmiamiScraper;
use crate::outbound::amiami_scraper::parser::parse_product_list;
use anyhow::Context;
use log::info;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde_json::Value;
use std::future::Future;

const USER_KEY: &str = "X-User-Key";
const USER_KEY_VALUE: &str = "amiami_dev";
const USER_AGENT: &str = "User-Agent";
const USER_AGENT_VALUE: &str = "Mozilla/5.0";
const BISHOUJO_CATEGORY: &str = "459";
const MATURE_CATEGORY: &str = "9708";
const PRODUCT_LIST_URL: &str = "https://api.amiami.com/api/v1.0/items?pagemax=50&pagecnt={page}&lang=eng&age_confirm=1&s_cate2={category}&s_st_list_preorder_available=1&s_st_list_backorder_available=1&s_st_list_newitem_available=1&s_st_condition_flg=1";
const PRODUCT_DETAILS_URL: &str = "https://www.amiami.com/eng/detail/?{code}";
const PRODUCT_IMAGE_BASE_URL: &str = "https://img.amiami.com";
const MAX_NEW_PAGES: u32 = 3;

pub mod parser;

#[derive(Debug, Clone)]
pub struct AmiamiScraperImpl {
    client: Client,
}

impl AmiamiScraperImpl {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut header_map = HeaderMap::new();
        header_map.insert(USER_KEY, USER_KEY_VALUE.parse()?);
        header_map.insert(USER_AGENT, USER_AGENT_VALUE.parse()?);
        let client = Client::builder()
            .default_headers(header_map)
            .build()
            .context("Failed to build AmiamiScraperImpl client")?;
        Ok(AmiamiScraperImpl { client })
    }

    async fn get_product_list_page_json(&self, category: &str, page_no: u32) -> Result<Value, reqwest::Error> {
        let url = PRODUCT_LIST_URL.replace("{page}", page_no.to_string().as_str()).replace("{category}", category);
        let response = self.client.get(&url).send().await?;
        info!("request GET '{}' returned with status {}", url, response.status());
        let json = response.json().await?;
        Ok(json)
    }

    async fn get_product_list_page_items(&self, category: &str, page_no: u32) -> Result<Vec<ProductData>, ScrapeProductsError> {
        let json = self.get_product_list_page_json(category, page_no).await
            .with_context(|| format!("Error getting product urls for category '{}'", category))?;
        let urls = parse_product_list(category, json)?;
        println!("Found {} products on page {}", urls.len(), page_no);
        Ok(urls)
    }

    async fn get_products(&self, category: &str) -> Result<Vec<ProductData>, ScrapeProductsError> {
        let mut page_no = 1_u32;
        let mut products = Vec::<ProductData>::new();
        loop {
            let page_products = self.get_product_list_page_items(category, page_no).await?;
            let page_url_count = page_products.len();
            products.extend(page_products);
            if page_url_count < 20 || page_no >= MAX_NEW_PAGES {
                break;
            }
            page_no += 1;
        }
        products.reverse();
        println!("Found {} total products for category '{}'", products.len(), category);
        Ok(products)
    }
}

impl AmiamiScraper for AmiamiScraperImpl {
    fn get_products(&self, category: &str) -> impl Future<Output=Result<Vec<ProductData>, ScrapeProductsError>> + Send {
        self.get_products(category)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_bishoujo_product_data_list() {
        let scraper = AmiamiScraperImpl::new().unwrap();
        let product_data_list = scraper.get_products(BISHOUJO_CATEGORY).await;
        println!("{product_data_list:?}");
    }

    #[tokio::test]
    async fn test_get_mature_product_data_list() {
        let scraper = AmiamiScraperImpl::new().unwrap();
        let product_data_list = scraper.get_products(MATURE_CATEGORY).await;
        println!("{product_data_list:?}");
    }
}