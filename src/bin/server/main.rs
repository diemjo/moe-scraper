use log::{error, info, LevelFilter};
use moe_scraper::config::{ServerConfiguration, Site};
use moe_scraper::domain::melonbooks::ports::MelonbooksService;
use moe_scraper::domain::melonbooks::service::MelonbooksServiceImpl;
use moe_scraper::outbound::melonbooks_discord_notifier::MelonbooksDiscordNotifier;
use moe_scraper::outbound::melonbooks_scraper::MelonbooksScraperImpl;
use moe_scraper::outbound::sqlite::Sqlite;
use std::env;
use std::sync::Arc;
use chrono::Local;
use tokio_cron_scheduler::{Job, JobScheduler};

const OPENSSL_CONFIG_ENV_VAR: &str = "OPENSSL_CONF";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();
    let config = ServerConfiguration::load_config()?;
    if let Some(openssl_config_file) = &config.openssl_config {
        env::set_var(OPENSSL_CONFIG_ENV_VAR, openssl_config_file)
    }
    info!("using config:\n{:#?}", config);
    let db = Sqlite::new(config.db_path.as_path().to_str().unwrap())?;
    db.setup()?;
    let scheduler = JobScheduler::new().await?;
    if let Some(melonbooks_settings) = config.site_settings.get(&Site::Melonbooks) {
        let discord_settings = melonbooks_settings.discord_settings.clone();
        let notifier = MelonbooksDiscordNotifier::new(discord_settings);
        let scraper = MelonbooksScraperImpl::new()?;
        let service = Arc::new(MelonbooksServiceImpl::new(db, notifier, scraper));
        if let Some(schedule) = melonbooks_settings.schedule.as_ref() {
            schedule_melonbooks(&scheduler, schedule, service.clone()).await?;
        }
    }
    scheduler.start().await?;
    loop {}
}

async fn schedule_melonbooks<S: MelonbooksService>(scheduler: &JobScheduler, schedule: &str, service: Arc<S>) -> Result<(), Box<dyn std::error::Error>> {
    scheduler.add(
        Job::new_async_tz(schedule, Local, move |_uuid, _l| {
            Box::pin({
                let service = service.clone();
                async move {
                    match service.scrape_available_products().await {
                        Ok(_) => info!("Successfully scraped melonbooks"),
                        Err(e) => error!("{:?}", e),
                    };
                }
            })
        })?
    ).await?;
    Ok(())
}