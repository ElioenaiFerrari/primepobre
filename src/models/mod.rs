use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Source {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "url")]
    Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Movie {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source: Source,
    pub stream: String,
    pub thumbnail_url: String,
    pub duration: i32,
    pub mime_type: String,
    pub genre: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Serie {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub genre: String,
    pub seasons: Vec<Season>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    pub id: String,
    pub serie_id: String,
    pub serie: Option<Serie>,
    pub episodes: Vec<Episode>,
    pub number: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Episode {
    pub id: String,
    pub season_id: String,
    pub season: Option<Season>,
    pub number: i32,
    pub title: String,
    pub description: String,
    pub source: Source,
    pub stream: String,
    pub thumbnail_url: String,
    pub duration: i32,
    pub mime_type: String,
}
