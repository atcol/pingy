use chrono::prelude::*;
use log::{debug, error, info};
use std::collections::HashMap;
use url::Url;

use actix_web::{
    get,
    web::{self, Data, ServiceConfig},
    Responder,
};
use pingy::*;
use reqwest;
use shuttle_actix_web::ShuttleActixWeb;
use std::sync::RwLock;

/// Return the latest results
///
#[get("/")]
async fn get_results(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let res = state.get_results().await;

    Ok(web::Json(res))
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = pingy::Config::new("./config.toml");

    let results = RwLock::new(HashMap::new());

    let app_state = pingy::AppState {
        monitors: config.monitors,
        results: results.into(),
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(Url, MonitorResult)>(1);

    for conf in app_state.monitors.clone() {
        let t = tx.clone();
        tokio::spawn(async move {
            let c = conf.clone();
            debug!("Starting {} monitor for every {}", c.url, c.check_interval);
            process(c.url, c.check_interval, t).await
        });
    }

    let mut state = app_state.clone();
    tokio::spawn(async move {
        loop {
            if let Some((url, result)) = rx.recv().await {
                debug!("Received result {:?}", result);
                state.record(url, result);
            } else {
                info!("None response for receiver. Exiting");
                break;
            }
        }
    });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(get_results);
        cfg.app_data(Data::new(app_state.clone()));
    };

    info!("Starting actix");
    Ok(config.into())
}

async fn process(
    url: Url,
    timeout: u64,
    tx: tokio::sync::mpsc::Sender<(Url, MonitorResult)>,
) -> Result<(), String> {
    info!("Processor initialised for {}", url);
    loop {
        debug!("Checking {}", url);
        let ts = Utc::now();
        let res = reqwest::get(url.clone())
            .await
            .map_err(|e| format!("Error {}", e))?;

        match tx
            .send((
                url.clone(),
                MonitorResult {
                    timestamp: ts,
                    status_code: res.status().into(),
                    latency: (ts.time() - Utc::now().time()).num_milliseconds(),
                },
            ))
            .await
        {
            Err(e) => {
                error!("Couldn't send result for {}: {e}", url);
                break;
            }
            _ => (),
        }
        info!("Sleeping for {}ms", timeout);
        let _ = tokio::time::sleep(std::time::Duration::from_millis(timeout.into())).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use actix_web::{http::header::ContentType, test, App};

    use super::*;

    #[actix_web::test]
    async fn test_get_results() {
        let app = test::init_service(App::new().service(get_results).app_data(AppState {
            monitors: vec![],
            results: Arc::new(HashMap::new().into()),
        }))
        .await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::json())
            .to_request();
        let resp: HashMap<url::Url, u16> = test::try_call_and_read_body_json(&app, req)
            .await
            .expect("Failed request");
        assert_eq!(0, resp.values().len());
    }
}
