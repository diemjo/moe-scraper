use crate::domain::melonbooks::ports::MelonbooksService;
use anyhow::Context;
use axum::response::Redirect;
use axum::routing::{get, post};
use handlers::melonbooks_routes;
use log::info;
use std::sync::Arc;
use tokio::net;

mod handlers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig {
    pub port: u16,
}

#[derive(Debug, Clone)]
struct AppState<MS: MelonbooksService> {
    melonbooks_service: Arc<MS>
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new<MS: MelonbooksService>( config: HttpServerConfig, melonbooks_service: Arc<MS>) -> Result<Self, anyhow::Error> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );
        let state = AppState { melonbooks_service };
        let router = axum::Router::new()
            .route("/", get(|| async { Redirect::temporary("/melonbooks") }))
            .nest("/melonbooks", melonbooks_routes())
            .nest("/api", api_routes())
            .layer(trace_layer)
            .with_state(state);
        
        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
        Ok(Self {
            listener,
            router
        })
    }
    
    pub async fn run(self) -> Result<(), anyhow::Error> {
        info!("starting inbound http server  {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router).await.context("server error")?;
        Ok(())
    }
}

fn melonbooks_routes<MS: MelonbooksService>() -> axum::Router<AppState<MS>> {
    axum::Router::new()
        .route("/", get(melonbooks_routes::get_overview::<MS>))
        .route("/artist", post(melonbooks_routes::post_artist::<MS>))
        .route("/artist/delete/id/:artist_id", post(melonbooks_routes::delete_artist::<MS>))
}

fn api_routes<MS: MelonbooksService>() -> axum::Router<AppState<MS>> {
    axum::Router::new().route("/artists", get(melonbooks_routes::get_artists::<MS>))
}