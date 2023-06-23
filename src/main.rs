pub mod almost_eq;
pub mod core;
pub mod dataset;
pub mod error;
pub mod search;

use crate::{core::Core, dataset::Dataset};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub dataset: Arc<Dataset>,
    pub core: Arc<Core>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let dataset = Arc::new(Dataset::load().await?);
    let core = Arc::new(Core::new(&dataset)?);

    let router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/search", post(search::handler))
        .with_state(AppState { dataset, core })
        .fallback_service(ServeDir::new("public"));

    tracing::info!("监听 http://localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
