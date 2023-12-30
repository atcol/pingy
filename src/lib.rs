pub mod app;
use chrono::prelude::*;
use log::{debug, error, info};
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, fs::File, io::Read};

#[derive(Debug, Clone, PartialEq, Hash, Eq, serde::Deserialize, serde::Serialize)]
pub struct UrlConfig {
    pub title: String,
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
    pub results: Arc<RwLock<HashMap<UrlConfig, Vec<MonitorResult>>>>,
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
    pub fn get_results(&self) -> HashMap<UrlConfig, Vec<MonitorResult>> {
        let lock = self.results.read().expect("Couldn't lock results");
        lock.clone()
    }

    pub fn record(&mut self, url: UrlConfig, result: MonitorResult) -> () {
        let mut map = self.results.write().expect("Couldn't unlock results");
        map.entry(url).or_insert(Vec::new()).push(result);
    }
}

/// An async task to request the given URL every `url.check_interval` ms
pub async fn monitor(
    url: UrlConfig,
    tx: tokio::sync::mpsc::Sender<(UrlConfig, MonitorResult)>,
) -> Result<(), String> {
    info!("Processor initialised for {}", url.title);
    loop {
        debug!("Checking {}", url.title);
        let ts = Utc::now();
        let res = reqwest::get(url.url.clone())
            .await
            .map_err(|e| format!("Error {}", e))?;

        match tx
            .send((
                url.clone(),
                MonitorResult {
                    timestamp: ts,
                    status_code: res.status().into(),
                    latency: (Utc::now().time() - ts.time()).num_milliseconds(),
                },
            ))
            .await
        {
            Err(e) => {
                error!("Couldn't send result for {}: {e}", url.url);
                break;
            }
            _ => (),
        }
        debug!(
            "{} monitor sleeping for {}ms",
            url.title, url.check_interval
        );
        let _ =
            tokio::time::sleep(std::time::Duration::from_millis(url.check_interval.into())).await;
    }
    Ok(())
}
