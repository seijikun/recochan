use serde_derive::Deserialize;
use config::{Config, File, ConfigError};


#[derive(Deserialize)]
pub struct RecoChanSettingsApi {
    pub bind: String,
    pub port: u16
}


#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum RecoChanSettingsDataProvider {
    SQL {
        connection_string: String,
        aid_name: String,
        uid_name: String,
        rating_name: String,
        table_name: String
    },
    CSVTest { path: String }
}


#[derive(Deserialize)]
pub struct RecoChanSettings {
    pub api: RecoChanSettingsApi,
    pub dataprovider: RecoChanSettingsDataProvider,
    pub retrain_every_sec: u64
}


impl RecoChanSettings {
    pub fn open(filename: &str) -> Result<Self, ConfigError> {
        let mut settings = Config::default();

        // Add defaults
        settings.set_default("api.bind", "127.0.0.1").unwrap();
        settings.set_default("api.port", 1337).unwrap();
        settings.set_default("retrain_every_sec", 24*60*60).unwrap();

        settings.merge(File::with_name(filename))?;
        return settings.try_into();
    }
}