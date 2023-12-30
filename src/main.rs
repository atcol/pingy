pub mod app;

use log::{debug, info};
use std::collections::HashMap;

use actix_web::{
    get,
    web::{self, Data, ServiceConfig},
    Responder,
};
use leptos::*;
use leptos_actix::{generate_route_list, LeptosRoutes};
use pingy::app::App;
use pingy::monitor;
use pingy::*;

use shuttle_actix_web::ShuttleActixWeb;
use std::sync::RwLock;

/// Return the latest results
#[get("/api/monitors/results")]
async fn get_results(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let res = state.get_results();

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

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(UrlConfig, MonitorResult)>(1);

    for conf in app_state.monitors.clone() {
        let t = tx.clone();
        tokio::spawn(async move {
            let c = conf.clone();
            debug!("Starting {} monitor for every {}", c.url, c.check_interval);
            monitor(c.clone(), t).await
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

    let leptos_config = get_configuration(Some("Cargo.toml")).await.unwrap();
    let site_root = leptos_config.leptos_options.site_root.clone();
    use actix_files::Files;
    let as1 = app_state.clone();
    let as2 = app_state.clone();
    let config = move |cfg: &mut ServiceConfig| {
        let routes = generate_route_list(move || {
            view! {
                <App
                    app_state=as1.clone()
                />
            }
        });
        cfg.service(get_results);
        cfg.app_data(Data::new(app_state.clone()));
        cfg.route("/api/{tail:.*}", leptos_actix::handle_server_fns());
        cfg.leptos_routes(
            leptos_config.leptos_options.to_owned(),
            routes.to_owned(),
            move || {
                view! {
                    <App
                        app_state=as2.clone()
                    />
                }
            },
        );
        cfg.service(Files::new("/", site_root));
    };

    info!("Starting actix");
    Ok(config.into())
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
