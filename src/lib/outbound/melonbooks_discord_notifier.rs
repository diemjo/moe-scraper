use crate::config::DiscordSettings;
use crate::domain::melonbooks::models::product::Product;
use crate::domain::melonbooks::ports::MelonbooksNotifier;
use log::error;
use webhook::client::WebhookClient;

const DISCORD_URL: &str = "https://discord.com/api/webhooks/";

#[derive(Debug, Clone)]
pub struct MelonbooksDiscordNotifier {
    settings: Option<DiscordSettings>
}

impl MelonbooksDiscordNotifier {
    pub fn new(settings: Option<DiscordSettings>) -> Self {
        Self {
            settings
        }
    }
    
    async fn send_products_notifications<P: AsRef<Product>>(&self, content: String, products: &[P]) -> Result<(), anyhow::Error> {
        if self.settings.is_none() {
            return Ok(());
        }
        let settings = self.settings.as_ref().unwrap();
        let url = format!("{}{}", DISCORD_URL, &settings.api_key);
        let client = WebhookClient::new(&url);
        for product_chunk in products.chunks(settings.chunk_size as usize) {
            client.send(|mut message| {
                message = message
                    .content(content.as_str())
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
    match product.price() { 
        Some(price) => format!("{} [{}]\n{}", product.category(), product.flags().join(" "), price),
        None => format!("{} [{}]", product.category(), product.flags().join(" "))
    }
}

impl MelonbooksNotifier for MelonbooksDiscordNotifier {
    async fn new_products<P: AsRef<Product>>(&self, artist: &str, products: &[P]) {
        if let Err(e) = self.send_products_notifications(format!("{}: new products available", artist), products).await {
            error!("Unable to send new product notifications: {}", e);
        }
    }

    async fn restocked_products<P: AsRef<Product>>(&self, artist: &str, products: &[P]) {
        if let Err(e) = self.send_products_notifications(format!("{}: products available again", artist), products).await {
            error!("Unable to send restocked product notifications: {}", e);
        }
    }
}