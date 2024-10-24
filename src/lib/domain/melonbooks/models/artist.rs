use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Artist {
    id: i32,
    date_added: DateTime<Utc>,
    name: String,
    following: bool,
    date_followed: Option<DateTime<Utc>>,
}

impl Artist {
    pub fn new(id: i32, date_added: DateTime<Utc>, name: String, following: bool, date_followed: Option<DateTime<Utc>>) -> Self {
        Artist { id, date_added, name, following, date_followed }
    }
   
    pub fn id(&self) -> i32 { self.id }
    pub fn date_added(&self) -> DateTime<Utc> { self.date_added.clone() }
    pub fn name(&self) -> &str { &self.name }
    pub fn following(&self) -> bool { self.following }
    pub fn date_followed(&self) -> Option<DateTime<Utc>> { self.date_followed.clone() }
}

#[derive(Debug)]
pub struct ArtistArgs {
    name: String,
}

impl ArtistArgs {
    pub fn new(name: String) -> Self {
        ArtistArgs { name }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Error)]
pub enum FollowArtistError {
    #[error("Artist already followed since {0}")]
    AlreadyFollowedError(DateTime<Utc>),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UnfollowArtistError {
    #[error("unknown artist '{name}'")]
    UnknownArtist{ name: String },
    #[error("artist '{name}' not followed")]
    ArtistNotFollowed{ name: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetArtistsError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}