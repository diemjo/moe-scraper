use crate::domain::amiami::models::availability::Availability;
use crate::domain::amiami::models::product::ProductData;
use crate::outbound::amiami_scraper::{PRODUCT_DETAILS_URL, PRODUCT_IMAGE_BASE_URL};
use serde_json::Value;
use thiserror::Error;

pub fn parse_product_list(category: &str, json: Value) -> Result<Vec<ProductData>, ParseError> {
    let items = json["items"].as_array()
        .ok_or_else(|| ParseError::ProductListNotFound)?;
    let products = items.into_iter()
        .map(|p| parse_item(category, p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(products)
}

fn parse_item(category: &str, json: &Value) -> Result<ProductData, ParseError> {
    let gcode = json["gcode"].as_str().ok_or(ParseError::ProductGcodeNotFound)?;
    let url = PRODUCT_DETAILS_URL.replace("{code}", &format!("gcode={}", gcode));
    let title = json["gname"].as_str().ok_or(ParseError::ProductTitleNotFound(gcode.to_owned()))?.to_owned();
    let rel_image_url = json["thumb_url"].as_str().ok_or(ParseError::ProductImageUrlNotFound(gcode.to_owned()))?.to_owned();
    let image_url = format!("{PRODUCT_IMAGE_BASE_URL}{rel_image_url}");
    let maker = json["maker_name"].as_str().ok_or(ParseError::ProductMakerNotFound(gcode.to_owned()))?.to_owned();
    let full_price = json["c_price_taxed"].as_number().and_then(|n| n.as_i64()).ok_or(ParseError::ProductFullPriceNotFound(gcode.to_owned()))?;
    let min_price = json["min_price"].as_number().and_then(|n| n.as_i64()).ok_or(ParseError::ProductMinPriceNotFound(gcode.to_owned()))?;
    let availability = parse_availability(gcode, json)?;
    Ok(ProductData::new(url, title, image_url, category.to_owned(), maker, full_price.try_into().unwrap_or(0), min_price.try_into().unwrap_or(0), availability))
}

fn parse_availability(gcode: &str, json: &Value) -> Result<Availability, ParseError> {
    let instock_flg = json["instock_flg"].as_number().and_then(|n| n.as_i64())
        .ok_or_else(|| ParseError::ProductAvailabilityNotFound(gcode.to_owned(), "instock_flg".to_owned()))?;
    let preorderitem = json["preorderitem"].as_number().and_then(|n| n.as_i64())
        .ok_or_else(|| ParseError::ProductAvailabilityNotFound(gcode.to_owned(), "preorderitem".to_owned()))?;
    match (instock_flg, preorderitem) {
        (1, _) => Ok(Availability::Available),
        (_, 1) => Ok(Availability::Preorder),
        (_, _) => Ok(Availability::NotAvailable),
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Could not find product list")]
    ProductListNotFound,
    #[error("Could not find product gcode")]
    ProductGcodeNotFound,
    #[error("Could not find product title for gcode {0}")]
    ProductTitleNotFound(String),
    #[error("Could not find product image url for gcode {0}")]
    ProductImageUrlNotFound(String),
    #[error("Could not find product maker for gcode {0}")]
    ProductMakerNotFound(String),
    #[error("Could not find product full price for gcode {0}")]
    ProductFullPriceNotFound(String),
    #[error("Could not find product min price for gcode {0}")]
    ProductMinPriceNotFound(String),
    #[error("Could not find product availability {0} for gcode {1}")]
    ProductAvailabilityNotFound(String, String),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::outbound::amiami_scraper::BISHOUJO_CATEGORY;

    #[test]
    fn test_parse_product_urls() {
        let document = get_list_json();
        let products_data = parse_product_list(BISHOUJO_CATEGORY, document).unwrap();
        println!("{:?}", products_data);
        assert_eq!(products_data.len(), 50)
    }

    fn get_list_json() -> Value {
        let str = get_test_list_json();
        let json = str.parse().unwrap();
        json
    }

    // https://api.amiami.com/api/v1.0/items?pagemax=20&lang=eng&mcode=&ransu=&age_confirm=1&s_cate2=459
    fn get_test_list_json() -> &'static str {
        include_str!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test-data/amiami/product-list.json"
            )
        )
    }
}