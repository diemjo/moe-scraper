use debug_ignore::DebugIgnore;
use figment::providers::{Env, Format, Yaml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::path::PathBuf;
use strum_macros::{Display, EnumString};
use thiserror::Error;
use tracing::level_filters::LevelFilter;

#[derive(Debug)]
pub struct ServerConfiguration {
    pub db_path: PathBuf,
    pub log_level: LevelFilter,
    pub melonbooks: SiteSettings,
    pub amiami: SiteSettings,
    pub openssl_config: Option<PathBuf>,
    pub http_settings: HttpSettings,
}

#[derive(Debug, Clone)]
pub struct SiteSettings {
    pub schedule: Option<String>,
    pub discord_settings: Option<DiscordSettings>,
}

#[derive(Debug, Clone)]
pub struct DiscordSettings {
    pub api_key: DebugIgnore<String>,
    pub image_url: Option<String>,
    pub username: String,
    pub chunk_size: u32
}

#[derive(Debug, Clone)]
pub struct HttpSettings {
    pub port: u16,
    pub assets_dir: Option<PathBuf>,
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self { 
            port: 80,
            assets_dir: None
        }
    }
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
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigurationOptions {
    pub dbpath: Option<PathBuf>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_log_level")]
    pub loglevel: LevelFilter,
    pub melonbooks: SiteSettingsOptions,
    pub amiami: SiteSettingsOptions,
    pub opensslconfig: Option<PathBuf>,
    pub http: Option<HttpSettingsOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteSettingsOptions {
    schedule: Option<String>,
    discord: Option<DiscordSettingsOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordSettingsOptions {
    pub apikey: String,
    pub imageurl: Option<String>,
    pub username: Option<String>,
    pub chunksize: Option<u32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpSettingsOptions {
    pub port: Option<u16>,
    pub assetsdir: Option<PathBuf>,
}

impl ServerConfigurationOptions {
    fn into_actual(self) -> ServerConfiguration {
        ServerConfiguration {
            db_path: self.dbpath.unwrap_or_else(|| PathBuf::from("/data/moe-scraper.sqlite")),
            log_level: self.loglevel,
            melonbooks: self.melonbooks.into_actual(&Site::Melonbooks),
            amiami: self.amiami.into_actual(&Site::Amiami),
            openssl_config: self.opensslconfig,
            http_settings: self.http.map(|h| h.into_actual()).unwrap_or_else(|| HttpSettings::default()),
        }
    }
}

fn default_log_level() -> LevelFilter {
    LevelFilter::INFO
}

impl SiteSettingsOptions {
    pub fn into_actual(self, site: &Site) -> SiteSettings {
        SiteSettings {
            schedule: self.schedule,
            discord_settings: self.discord.map(|ds| ds.into_actual(site))
        }
    }
}

impl DiscordSettingsOptions {
    fn into_actual(self, site: &Site) -> DiscordSettings {
        DiscordSettings {
            api_key: self.apikey.into(),
            image_url: self.imageurl,
            username: self.username.unwrap_or_else(|| site.to_string() + "-Scraper"),
            chunk_size: self.chunksize.unwrap_or(10)
        }
    }
}

impl HttpSettingsOptions {
    fn into_actual(self) -> HttpSettings {
        HttpSettings {
            port: self.port.unwrap_or(80),
            assets_dir: self.assetsdir
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum Site {
    Melonbooks,
    Amiami,
} 