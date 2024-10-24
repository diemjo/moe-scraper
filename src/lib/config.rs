use figment::providers::{Env, Format, Yaml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use strum_macros::{Display, EnumString};
use thiserror::Error;

#[derive(Debug)]
pub struct ServerConfiguration {
    pub db_path: PathBuf,
    pub site_settings: HashMap<Site, SiteSettings>,
    pub discord_settings: HashMap<Site, DiscordSettings>,
    pub openssl_config: Option<PathBuf>
}

#[derive(Debug, Clone)]
pub struct SiteSettings {
    pub schedule: String,
}

#[derive(Debug, Clone)]
pub struct DiscordSettings {
    pub api_key: String,
    pub image_url: Option<String>,
    pub username: String,
    pub chunk_size: u32
}

#[derive(Debug, Error)]
#[error("invalid config: {0}")]
pub struct ConfigurationError(figment::Error);

impl ServerConfiguration {
    pub fn load_config() -> Result<Self, ConfigurationError> {
        let config = Figment::from(Yaml::file("/config/moe-scraper.yaml"))
            .merge(Yaml::file("./config/moe-scraper.yaml"))
            .merge(Yaml::file("./moe-scraper.yaml"))
            .merge(Env::prefixed("MOE_").split('_'))
            .extract::<ServerConfigurationOptions>()
            .or_else(|e| Err(ConfigurationError(e)))?
            .into_actual();
        Ok(config)
    }
    
    pub fn melonbooks(&self) -> bool {
        self.site_settings.contains_key(&Site::Melonbooks)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigurationOptions {
    pub dbpath: Option<PathBuf>,
    pub sites: Option<HashMap<Site, SiteSettingsOptions>>,
    pub discord: Option<HashMap<Site, DiscordSettingsOptions>>,
    pub opensslconfig: Option<PathBuf>
}

#[derive(Debug, Serialize, Deserialize)]
struct SiteSettingsOptions {
    schedule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordSettingsOptions {
    pub apikey: String,
    pub imageurl: Option<String>,
    pub username: Option<String>,
    pub chunksize: Option<u32>
}

impl ServerConfigurationOptions {
    fn into_actual(self) -> ServerConfiguration {
        ServerConfiguration {
            db_path: self.dbpath.unwrap_or_else(|| PathBuf::from("/data/moe-scraper.sqlite")),
            site_settings: self.sites
                .map(|map| map.into_iter().map(|(k, v)| (k.to_owned(), v.into_actual())).collect())
                .unwrap_or_else(|| HashMap::new()),
            discord_settings: self.discord
                .map(|map| map.into_iter().map(|(k, v)| (k.to_owned(), v.into_actual(&k))).collect())
                .unwrap_or_else(|| HashMap::new()),
            openssl_config: self.opensslconfig,
        }
    }
}

impl SiteSettingsOptions {
    pub fn into_actual(self) -> SiteSettings {
        SiteSettings { schedule: self.schedule }
    }
}

impl DiscordSettingsOptions {
    fn into_actual(self, site: &Site) -> DiscordSettings {
        DiscordSettings {
            api_key: self.apikey,
            image_url: self.imageurl,
            username: self.username.unwrap_or_else(|| site.to_string() + "-Scraper"),
            chunk_size: self.chunksize.unwrap_or(10)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum Site {
    Melonbooks,
} 