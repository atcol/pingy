use std::collections::HashMap;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::{AppState, MonitorResult, UrlConfig};
use thaw::*;

#[component]
pub fn App(app_state: AppState) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/ssr_modes.css"/>
        <Title text="Welcome to Leptos"/>

        <Router>
            <main>
                <Routes>
                    <Route path="" view=move || view! {
                        <HomePage results=app_state
                            .get_results()
                        />
                    }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage(results: HashMap<UrlConfig, Vec<MonitorResult>>) -> impl IntoView {
    view! {
        <h1>"Pingy!"</h1>

        <Table>
            <thead>
                <tr>
                    <th>"Title"</th>
                    <th>"URL"</th>
                    <th>"Status Code"</th>
                    <th>"Latency (ms)"</th>
                </tr>
            </thead>
            <tbody>
                {results.into_iter()
                    .map(|n| view! {
                        <tr>
                            <td>{n.0.title.to_string()}</td>
                            <td>{n.0.url.to_string()}</td>
                            <td>{n.1.iter().last().unwrap().status_code}</td>
                            <td>{n.1.iter().last().unwrap().latency}</td>
                        </tr>
                    })
                    .collect::<Vec<_>>()}
            </tbody>
        </Table>
    }
}
