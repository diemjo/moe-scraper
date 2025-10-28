use crate::domain::amiami::ports::AmiamiService;
use crate::domain::melonbooks::models::artist::{Artist, ArtistArgs, FollowArtistError, GetArtistsError, UnfollowArtistError};
use crate::domain::melonbooks::models::product::{AddTitleSkipSequenceError, DeleteTitleSkipSequenceError, GetProductsError, GetTitleSkipSequencesError, Product};
use crate::domain::melonbooks::ports::MelonbooksService;
use crate::inbound::http::AppState;
use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Form, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct GetArtistsResponseBody {
    artists: Vec<ArtistResponse>
}

#[derive(Debug, Serialize)]
pub struct ArtistResponse {
    id: i32,
    date_added: DateTime<Utc>,
    name: String,
    following: bool,
}

impl From<Artist> for ArtistResponse {
    fn from(a: Artist) -> Self {
        Self {
            id: a.id(),
            date_added: a.date_added(),
            name: a.name().to_owned(),
            following: a.following(),
        }
    }
}

pub async fn get_artists<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>) -> Json<GetArtistsResponseBody> {
    let artists = state.melonbooks_service.get_artists().await
        .unwrap()
        .into_iter()
        .filter(|a| a.following())
        .map(|a| a.into())
        .collect::<Vec<ArtistResponse>>();
    Json(GetArtistsResponseBody { artists } )
}

#[derive(Template)]
#[template(path = "melonbooks.html")]
struct MelonbooksTemplate {
    products: Vec<Product>,
    artists: Vec<Artist>,
    selected_artist: Option<Artist>,
    skip_sequences: Vec<String>,
}

impl MelonbooksTemplate {
    fn format_date(date: DateTime<Utc>) -> String {
        date.format("%Y-%m-%d %H:%M").to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct OverviewParams {
    pub selected_artist: Option<i32>,
}

pub async fn get_overview<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>, Query(params): Query<OverviewParams>) -> Response {
    get_overview_response(state.melonbooks_service, params.selected_artist).await
}

#[derive(Debug, Deserialize)]
pub struct PostArtistForm {
    name: String
}

pub async fn post_artist<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>, Form(input): Form<PostArtistForm>) -> Response {
    if let Err(e) = state.melonbooks_service.follow_artist(&ArtistArgs::new(input.name)).await {
        return e.into_response();
    }
    get_overview_response(state.melonbooks_service, None).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeleteArtistForm {
    selected_artist_id: i32
}

pub async fn delete_artist<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>, Form(input): Form<DeleteArtistForm>) -> Response {
    if let Err(e) = state.melonbooks_service.unfollow_artist(input.selected_artist_id).await {
        return e.into_response();
    }
    get_overview_response(state.melonbooks_service, None).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddTitleSkipSequenceForm {
    title_skip_sequence: String
}

pub async fn post_title_skip_sequence<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>, Form(input): Form<AddTitleSkipSequenceForm>) -> Response {
    if let Err(e) = state.melonbooks_service.add_title_skip_sequence(&input.title_skip_sequence).await {
        return e.into_response();
    }
    get_overview_response(state.melonbooks_service, None).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeleteTitleSkipSequenceForm {
    title_skip_sequence: String
}

pub async fn delete_title_skip_sequence<MS: MelonbooksService, AS: AmiamiService>(State(state): State<AppState<MS, AS>>, Form(input): Form<DeleteTitleSkipSequenceForm>) -> Response {
    if let Err(e) = state.melonbooks_service.delete_title_skip_sequence(&input.title_skip_sequence).await {
        return e.into_response();
    }
    get_overview_response(state.melonbooks_service, None).await
}

pub async fn get_overview_response<MS: MelonbooksService>(service: Arc<MS>, selected_artist_id: Option<i32>) -> Response {
    let artists = match service.get_followed_artists().await {
        Ok(a) => a,
        Err(e) => return e.into_response()
    };
    let selected_artist = match selected_artist_id {
        Some(id) => artists.iter().find(|a| a.id() == id).cloned(),
        None => None
    };
    let products = match selected_artist.as_ref() {
        Some(artist) => match service.get_products_by_artist(artist.id()).await {
            Ok(p) => p,
            Err(e) => return e.into_response()
        }
        None => match service.get_products().await {
            Ok(p) => p,
            Err(e) => return e.into_response()
        }
    };
    let skip_sequences = match service.get_title_skip_sequences().await {
        Ok(s) => s,
        Err(e) => return e.into_response()
    };
    let template = MelonbooksTemplate {
        products,
        artists,
        selected_artist,
        skip_sequences
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

impl IntoResponse for GetArtistsError {
    fn into_response(self) -> Response {
        match self {
            GetArtistsError::Unknown(cause) => (StatusCode::INTERNAL_SERVER_ERROR, cause.to_string()).into_response(),
        }
    }
}

impl IntoResponse for FollowArtistError {
    fn into_response(self) -> Response {
        match self {
            e @ FollowArtistError::AlreadyFollowedError { .. } => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            FollowArtistError::Unknown(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

impl IntoResponse for UnfollowArtistError {
    fn into_response(self) -> Response {
        match self {
            e @ UnfollowArtistError::UnknownArtist { .. } => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
            e @ UnfollowArtistError::ArtistNotFollowed { .. } => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            UnfollowArtistError::Unknown(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

impl IntoResponse for GetTitleSkipSequencesError {
    fn into_response(self) -> Response {
        match self { 
            GetTitleSkipSequencesError::Unknown(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

impl IntoResponse for AddTitleSkipSequenceError {
    fn into_response(self) -> Response {
        match self {
            AddTitleSkipSequenceError::Unknown(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

impl IntoResponse for DeleteTitleSkipSequenceError {
    fn into_response(self) -> Response {
        match self {
            DeleteTitleSkipSequenceError::Unknown(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}