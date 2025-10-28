use crate::domain::amiami::models::availability::Availability;
use crate::domain::amiami::models::product::{GetProductsError, Product};
use crate::domain::amiami::ports::AmiamiService;
use crate::domain::melonbooks::ports::MelonbooksService;
use crate::inbound::http::AppState;
use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use strum::IntoEnumIterator;

#[derive(Template)]
#[template(path = "amiami.html")]
struct AmiamiTemplate {
    products: Vec<Product>,
    availabilities: Vec<Availability>,
    selected_availability: Option<Availability>,
}

impl AmiamiTemplate {
    fn format_date(date: DateTime<Utc>) -> String {
        date.format("%Y-%m-%d %H:%M").to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct OverviewParams {
    pub selected_availability: Option<Availability>,
}

pub async fn get_overview<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>, Query(params): Query<OverviewParams>) -> Response {
    get_overview_response(state.amiami_service, params.selected_availability).await
}

pub async fn get_overview_response<AS: AmiamiService>(service: Arc<AS>, selected_availability: Option<Availability>) -> Response {
    let products = match service.get_products().await {
        Ok(a) => a,
        Err(e) => return e.into_response()
    };
    let filtered_products = match &selected_availability {
        Some(availability) => products.into_iter().filter(|p| &p.availability() == availability).collect(),
        None => products
    };
    let template = AmiamiTemplate {
        products: filtered_products,
        availabilities: Availability::iter().collect(),
        selected_availability
    };
    template.into_response()
}

impl IntoResponse for GetProductsError {
    fn into_response(self) -> Response {
        match self {
            GetProductsError::Unknown(cause) => (StatusCode::INTERNAL_SERVER_ERROR, cause.to_string()).into_response(),
        }
    }
}