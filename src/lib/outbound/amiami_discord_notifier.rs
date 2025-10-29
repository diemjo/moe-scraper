use crate::config::DiscordSettings;
use crate::domain::amiami::models::product::Product;
use crate::domain::amiami::ports::AmiamiNotifier;
use log::error;
use std::fmt::format;
use webhook::client::WebhookClient;

const DISCORD_URL: &str = "https://discord.com/api/webhooks/";

#[derive(Debug, Clone)]
pub struct AmiamiDiscordNotifier {
    settings: Option<DiscordSettings>
}

impl AmiamiDiscordNotifier {
    pub fn new(settings: Option<DiscordSettings>) -> Self {
        Self {
            settings
        }
    }
    
    async fn send_products_notifications<P: AsRef<Product>>(&self, content: &str, products: &[P]) -> Result<(), anyhow::Error> {
        if self.settings.is_none() {
            return Ok(());
        }
        let settings = self.settings.as_ref().unwrap();
        let url = format!("{}{}", DISCORD_URL, &settings.api_key);
        let client = WebhookClient::new(&url);
        for product_chunk in products.chunks(settings.chunk_size as usize) {
            client.send(|mut message| {
                message = message
                    .content(content)
                    .username(settings.username.as_str());
                if let Some(image_url) = &settings.image_url {
                    message = message.avatar_url(image_url);
                }
                for product in product_chunk {
                    let product = product.as_ref();
                    message = message
                        .embed(|embed| embed
                            .title(product.title())
                            .url(product.url())
                            .description(&product_description(&product))
                            .thumbnail(product.image_url())
                        );
                }
                message
            }).await.map_err(|e| anyhow::anyhow!("{:?}", e))?;
            tokio::time::sleep(core::time::Duration::from_secs(1)).await;
        }
        Ok(())
    }
}

fn product_description(product: &Product) -> String {
    format!(
        "{} — {}\n¥{}",
        product.release_date().format("%Y %B"),
        product.maker(),
        product.min_price()
    )
}

impl AmiamiNotifier for AmiamiDiscordNotifier {
    async fn new_products<P: AsRef<Product>>(&self, category: &str, products: &[P]) {
        if let Err(e) = self.send_products_notifications(&format!("Category {}: new products available", category), products).await {
            error!("Unable to send new product notifications: {}", e);
        }
    }

    async fn restocked_products<P: AsRef<Product>>(&self, category: &str, products: &[P]) {
        if let Err(e) = self.send_products_notifications(&format!("Category {}: products available again", category), products).await {
            error!("Unable to send restocked product notifications: {}", e);
        }
    }
}