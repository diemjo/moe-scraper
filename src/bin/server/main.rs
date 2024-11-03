use chrono::Local;
use log::{error, info};
use moe_scraper::config::ServerConfiguration;
use moe_scraper::domain::melonbooks::ports::{MelonbooksRepository, MelonbooksService};
use moe_scraper::domain::melonbooks::service::MelonbooksServiceImpl;
use moe_scraper::inbound::http::{HttpServer, HttpServerConfig};
use moe_scraper::outbound::melonbooks_discord_notifier::MelonbooksDiscordNotifier;
use moe_scraper::outbound::melonbooks_scraper::MelonbooksScraperImpl;
use moe_scraper::outbound::sqlite::Sqlite;
use std::env;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

const OPENSSL_CONFIG_ENV_VAR: &str = "OPENSSL_CONF";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfiguration::load_config()?;
    tracing_subscriber::fmt::fmt()
        .with_max_level(config.log_level)
        .init();
    if let Some(openssl_config_file) = &config.openssl_config {
        env::set_var(OPENSSL_CONFIG_ENV_VAR, openssl_config_file)
    }
    info!("using config:\n{:#?}", config);
    let db = Sqlite::new(config.db_path.as_path().to_str().unwrap())?;
    db.setup()?;
    let scheduler = JobScheduler::new().await?;
    let melonbooks_service = init_melonbooks(&config, db.clone(), &scheduler).await?;
    scheduler.start().await?;
    let http_config = HttpServerConfig { port: config.http_settings.port };
    let http_server = HttpServer::new(http_config, melonbooks_service).await?;
    http_server.run().await?;
    Ok(())
}

async fn init_melonbooks(config: &ServerConfiguration, repo: impl MelonbooksRepository, scheduler: &JobScheduler) -> Result<Arc<impl MelonbooksService>, anyhow::Error> {
    let melonbooks_settings = &config.melonbooks;
    let discord_settings = &melonbooks_settings.discord_settings;
    let schedule = &melonbooks_settings.schedule;
    let notifier = MelonbooksDiscordNotifier::new(discord_settings.to_owned());
    let scraper = MelonbooksScraperImpl::new()?;
    let service = Arc::new(MelonbooksServiceImpl::new(repo, notifier, scraper));
    if let Some(schedule) = schedule {
        schedule_melonbooks(&scheduler, &schedule, service.clone()).await?;
    }
    Ok(service)
}

async fn schedule_melonbooks<S: MelonbooksService>(scheduler: &JobScheduler, schedule: &str, service: Arc<S>) -> Result<(), anyhow::Error> {
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