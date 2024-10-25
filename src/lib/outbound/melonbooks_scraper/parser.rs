use crate::domain::melonbooks::models::availability::Availability;
use crate::outbound::melonbooks_scraper::{ProductData, PRODUCT_URL};
use itertools::Itertools;
use regex::Regex;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Not, Predicate};
use thiserror::Error;

pub fn parse_product_list(document: Document) -> Result<Vec<String>, ParseError> {
    let product_list = document.find(Class("item-list")).next()
        .ok_or_else(|| ParseError::ProductListNotFound)?;
    let product_urls = product_list.find(Name("li").and(Not(Class("item-list__placeholder"))))
        .map(parse_product_url_from_grid_item)
        .collect::<Result<_, _>>()?;
    Ok(product_urls)
}

fn parse_product_url_from_grid_item(node: Node) -> Result<String, ParseError> {
    let a = node.find(Name("a").child(Class("product_title"))).next()
        .and_then(|n| n.parent())
        .ok_or_else(|| ParseError::ProductLinkNodeNotFound)?;
    let href = a.attr("href")
        .ok_or_else(|| ParseError::ProductUrlNotFound(a.text()))?;
    Ok(PRODUCT_URL.replace("{relative_url}", href))
}

pub fn parse_product_details(document: Document) -> Result<ProductData, ParseError> {
    let item_page = document.find(Class("item-page")).next()
        .ok_or_else(|| ParseError::ProductItemPageNotFound)?;
    let title = parse_product_title(item_page)?;
    let category = parse_product_category(item_page)?;
    let flags = parse_product_flags(item_page)?;
    let tags = parse_product_tags(item_page)?;
    let price = parse_product_price(item_page)?;
    let circle = parse_product_circle(item_page)?;
    let artists = parse_product_artists(item_page)?;
    let availability = parse_product_availability(item_page)?;
    let image_url = parse_product_image_url(item_page)?;
    Ok(ProductData::new(title, circle, artists, image_url, category, tags, flags, price, availability))
}

fn parse_product_title(item_page: Node) -> Result<String, ParseError> {
    let header = item_page.find(Class("item-header")).next()
        .ok_or_else(|| ParseError::ProductItemHeaderNotFound)?;
    let page_header = header.find(Class("page-header")).next()
        .ok_or_else(|| ParseError::ProductPageHeaderNotFound)?;
    let title = page_header.text().trim().to_owned();
    Ok(title)
}

fn parse_product_category(item_page: Node) -> Result<String, ParseError> {
    let header = item_page.find(Class("item-header")).next()
        .ok_or_else(|| ParseError::ProductItemHeaderNotFound)?;
    let category_span = header.find(Class("notes-analog")).next()
        .ok_or_else(|| ParseError::ProductCategoryNotFound)?;
    let category = category_span.text().trim().to_owned();
    Ok(category)
}

fn parse_product_flags(item_page: Node) -> Result<Vec<String>, ParseError> {
    let header = item_page.find(Class("item-header")).next()
        .ok_or_else(|| ParseError::ProductItemHeaderNotFound)?;
    let tags = header.find(Class("notes-red"))
        .map(|n| n.text().trim().to_owned())
        .unique()
        .collect::<Vec<_>>();
    Ok(tags)
}

fn parse_product_tags(item_page: Node) -> Result<Vec<String>, ParseError> {
    let tag_list = item_page.find(Class("item-detail2").child(Class("mt6"))).next()
        .ok_or_else(|| ParseError::ProductTagListNotFound)?;
    let tags = tag_list.find(Name("a"))
        .map(|n| n.text().strip_prefix('#').map(|t| t.trim().to_owned()).unwrap_or_else(|| n.text().trim().to_owned()))
        .unique()
        .collect::<Vec<_>>();
    Ok(tags)
}

fn parse_product_price(item_page: Node) -> Result<Option<String>, ParseError> {
    let item_meta = item_page.find(Class("item-metas-wrap")).next()
        .ok_or_else(|| ParseError::ProductItemMetaNotFound)?;
    let price = item_meta.find(Class("price").child(Class("yen"))).next()
        .map(|p| p.text().trim().to_owned());
    Ok(price)
}

// optional
// https://www.melonbooks.co.jp/detail/detail.php?product_id=2587862
fn parse_product_circle(item_page: Node) -> Result<Option<String>, ParseError> {
    let row = item_page.find(Class("item-detail").descendant(Class("table-wrapper")).descendant(Name("tr")))
        .filter(|tr|
            tr.find(Name("th"))
                .filter(|th| vec!["サークル名"].contains(&th.text().as_str()))
                .next().is_some()
        )
        .next();
    let row = match row { 
        Some(row) => row,
        None => return Ok(None),
    };
    let regex = Regex::new(r"^(.*)(\(.*\))$").unwrap();
    let circle = row.find(Name("a"))
        .filter(|a| a.attr("href").unwrap_or("#") != "#")
        .map(|a| a.text())
        .next()
        .map(|c| {
            let captures = regex.captures(&c);
            match captures { 
                None => c,
                Some(captures) => captures.get(1).unwrap().as_str().trim().to_owned(),
            }
        })
        .ok_or_else(|| ParseError::ProductCircleNotFound)?;
    Ok(Some(circle))
}

fn parse_product_artists(item_page: Node) -> Result<Vec<String>, ParseError> {
    let row = item_page.find(Class("item-detail").descendant(Class("table-wrapper")).descendant(Name("tr")))
        .filter(|tr|
            tr.find(Name("th"))
                .filter(|th| vec!["作家名", "アーティスト"].contains(&th.text().as_str()))
                .next().is_some()
        )
        .next()
        .ok_or_else(|| ParseError::ProductArtistRowNotFound)?;
    let artists = row.find(Name("a"))
        .filter(|a| a.attr("href").unwrap_or("#") != "#")
        .map(|a| a.text().trim().to_owned())
        .unique()
        .collect::<Vec<_>>();
    Ok(artists)
}

fn parse_product_availability(item_page: Node) -> Result<Availability, ParseError> {
    let item_meta = item_page.find(Class("item-metas-wrap")).next()
        .ok_or_else(|| ParseError::ProductItemMetaNotFound)?;
    let availability_text = item_meta.find(Class("state-instock")).next()
        .map(|n| n.text())
        .ok_or_else(|| ParseError::ProductAvailabilityNotFound)?;
    let availability = match availability_text.as_str() {
        "-" => Availability::NotAvailable,
        "好評受付中" => Availability::Preorder,
        "残りわずか" => Availability::Available,
        "在庫あり" => Availability::Available,
        "発売中" => Availability::Available,
        other => Err(ParseError::ProductAvailabilityUnknown("availability_type".to_string() + other))?
    };
    Ok(availability)
}

fn parse_product_image_url(item_page: Node) -> Result<String, ParseError> {
    let img_url = item_page.find(Class("item-img").descendant(Name("img")))
        .next()
        .and_then(|i| i.attr("src"))
        .map(|src| src.replace("//", "https://").to_owned())
        .ok_or_else(|| ParseError::ProductImageUrlNotFound)?;
    Ok(img_url)
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Could not find product list")]
    ProductListNotFound,
    #[error("Could not find product link node from grid item")]
    ProductLinkNodeNotFound,
    #[error("Could not find product url in link node: {0}")]
    ProductUrlNotFound(String),
    #[error("Could not find product item page part")]
    ProductItemPageNotFound,
    #[error("Could not find product item header")]
    ProductItemHeaderNotFound,
    #[error("Could not find product page header")]
    ProductPageHeaderNotFound,
    #[error("Could not find product category")]
    ProductCategoryNotFound,
    #[error("Could not find product item meta")]
    ProductItemMetaNotFound,
    #[error("Could not find product table")]
    ProductTableNotFound,
    #[error("Could not find product circle row")]
    ProductCircleRowNotFound,
    #[error("Could not find product circle")]
    ProductCircleNotFound,
    #[error("Could not find product artist row")]
    ProductArtistRowNotFound,
    #[error("Could not find product tag list")]
    ProductTagListNotFound,
    #[error("Could not find product availability")]
    ProductAvailabilityNotFound,
    #[error("Could not find product image url")]
    ProductImageUrlNotFound,
    #[error("Unknown product availability: {0}")]
    ProductAvailabilityUnknown(String),
}

#[cfg(test)]
mod test {
    use crate::outbound::melonbooks_scraper::parser::{parse_product_details, parse_product_list};
    use select::document::Document;

    #[test]
    fn test_parse_product_urls() {
        let document = get_list_document();
        let urls = parse_product_list(document).unwrap();
        println!("{:?}", urls);
        assert_eq!(urls.len(), 30)
    }

    #[test]
    fn test_parse_product_details() {
        let document = get_details_document();
        let details = parse_product_details(document).unwrap();
        println!("{:?}", details);
    }
    
    #[test]
    fn test_parse_product_with_no_artist() {
        let document = get_music_document();
        let details = parse_product_details(document).unwrap();
        println!("{:?}", details);
    }

    fn get_list_document() -> Document {
        let path = get_test_list_html_path();
        let document = Document::from(path);
        document
    }

    // https://www.melonbooks.co.jp/search/search.php?name=%E3%81%BE%E3%81%B5%E3%82%86&text_type=author&pageno=1
    fn get_test_list_html_path() -> &'static str {
        include_str!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test-data/product-list.html"
            )
        )
    }

    fn get_details_document() -> Document {
        let path = get_test_details_html_path();
        let document = Document::from(path);
        document
    }

    // https://www.melonbooks.co.jp/detail/detail.php?product_id=2508959
    fn get_test_details_html_path() -> &'static str {
        include_str!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test-data/product-details.html"
            )
        )
    }
    
    fn get_music_document() -> Document {
        let path = get_test_music_html_path();
        let document = Document::from(path);
        document
    }

    // https://www.melonbooks.co.jp/detail/detail.php?product_id=2395046
    fn get_test_music_html_path() -> &'static str {
        include_str!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test-data/product-music.html"
            )
        )
    }
}