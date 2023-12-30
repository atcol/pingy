use chrono::prelude::*;
use log::info;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, fs::File, io::Read};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UrlConfig {
    pub url: url::Url,
    pub method: Option<String>,
    pub check_interval: u64,
}

/// The result of a monitor attempt
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MonitorResult {
    /// The time the request was attempted
    pub timestamp: DateTime<Utc>,
    /// The HTTP status code
    pub status_code: u16,
    /// The time taken to send & receive the headers
    pub latency: i64,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub monitors: Vec<UrlConfig>,
    pub results: Arc<RwLock<HashMap<url::Url, Vec<MonitorResult>>>>,
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub monitors: Vec<UrlConfig>,
}

impl Config {
    pub fn new(path: &str) -> Config {
        let mut file = File::open(path).expect("Could not open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read config file");
        toml::from_str(&contents).expect("Could not parse config file")
    }
}

impl AppState {
    pub async fn get_results(&self) -> HashMap<url::Url, Vec<MonitorResult>> {
        let lock = self.results.read().expect("Couldn't lock results");
        info!("Got results {:?}", lock);
        lock.clone()
    }

    pub fn record(&mut self, url: url::Url, result: MonitorResult) -> () {
        let mut map = self.results.write().expect("Couldn't unlock results");
        map.entry(url).or_insert(Vec::new()).push(result);
    }
}
