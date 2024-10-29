mod models;
use std::collections::HashMap;

use actix_cors::Cors;
use actix_files::Files;
use models::*;

use bytes::Bytes;

use actix_web::{
    get,
    http::header::{ContentLength, ACCEPT_RANGES},
    middleware::Logger,
    web::{scope, Data, Path},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tokio_util::io::ReaderStream;
// use reqwest::Client;

#[get("/movies")]
async fn get_movies(state: Data<State>) -> impl Responder {
    HttpResponse::Ok().json(&state.movies)
}

async fn stream_from_file(tx: mpsc::Sender<Result<Bytes, std::io::Error>>, media: &String) -> u64 {
    let file = tokio::fs::File::open(media).await.unwrap();
    let metadata = file.metadata().await.unwrap();
    let content_length = metadata.len();
    let reader = tokio::io::BufReader::new(file);
    actix_web::rt::spawn(async move {
        let mut stream = ReaderStream::new(reader);
        while let Some(item) = stream.next().await {
            match tx.send(item).await {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    content_length
}

async fn stream_from_url(tx: mpsc::Sender<Result<Bytes, std::io::Error>>, url: &String) -> u64 {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await.unwrap();
    let content_length = res.content_length().unwrap();

    actix_web::rt::spawn(async move {
        let stream = res.bytes_stream();
        let mut stream = stream;
        while let Some(item) = stream.next().await {
            match tx.send(Ok(item.unwrap())).await {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    content_length
}

#[get("/movies/{id}/stream")]
async fn stream_movie(state: Data<State>, id: Path<String>, req: HttpRequest) -> impl Responder {
    let movies = state.movies.clone();
    if let Some(movie) = movies.iter().cloned().find(|m| m.id == id.to_string()) {
        let (tx, rx) = mpsc::channel(1024);
        let content_length = match movie.source {
            Source::File => stream_from_file(tx, &movie.media).await,
            Source::Url => stream_from_url(tx, &movie.media).await,
        };

        log::info!(
            "Streaming movie: {} with size {}",
            movie.title,
            content_length
        );

        return HttpResponse::PartialContent()
            .content_type(movie.mime_type)
            .insert_header(ContentLength(content_length as usize))
            .insert_header((ACCEPT_RANGES, "bytes"))
            .streaming(ReceiverStream::new(rx));
    }
    HttpResponse::NotFound().finish()
}

#[get("/series/{serie_id}/seasons/{season_id}/episodes/{episode_id}/stream")]
async fn stream_serie_episode(
    state: Data<State>,
    path: Path<(String, String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (serie_id, season_id, episode_id) = path.into_inner();
    let series = state.series.clone();
    if let Some(serie) = series.iter().cloned().find(|s| s.id == serie_id) {
        if let Some(season) = serie.seasons.iter().cloned().find(|s| s.id == season_id) {
            if let Some(episode) = season.episodes.iter().cloned().find(|e| e.id == episode_id) {
                let (tx, rx) = mpsc::channel(1024);
                let content_length = match episode.source {
                    Source::File => stream_from_file(tx, &episode.media).await,
                    Source::Url => stream_from_url(tx, &episode.media).await,
                };

                return HttpResponse::PartialContent()
                    .content_type(episode.mime_type)
                    .insert_header(ContentLength(content_length as usize))
                    .insert_header((ACCEPT_RANGES, "bytes"))
                    .streaming(ReceiverStream::new(rx));
            }
        }
    }
    HttpResponse::NotFound().finish()
}

#[get("/series")]
async fn get_series(state: Data<State>) -> impl Responder {
    HttpResponse::Ok().json(&state.series)
}

#[derive(Debug, Clone)]
struct State {
    pub movies: Vec<Movie>,
    pub series: Vec<Serie>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let series = vec![
        Serie{
            id: "1".to_string(),
            title: "Peak Blinders".to_string(),
            thumbnail_url:"https://oldsiller.com.br/cdn/shop/articles/Estilo_masculino_peaky_blinders.jpg?v=1687810856".to_string(),
            description: "Peak Blinders é uma série de televisão britânica de drama histórico sobre uma família de gângsteres pós Primeira Guerra Mundial, que é chefiada por Tommy Shelby (Cillian Murphy).".to_string(),
            genre: "Ação".to_string(),
            seasons: vec![
                Season{
                    id: "1".to_string(),
                    serie_id: "1".to_string(),
                    number: 1,
                    serie: None,
                    episodes: vec![
                        Episode{
                            id: "1".to_string(),
                            season_id: "1".to_string(),
                            number: 1,
                            title: "Piloto".to_string(),
                            description: "Os Vingadores se reúnem para desfazer as ações de Thanos e restaurar o universo.".to_string(),
                            media: "https://videos.pexels.com/video-files/28828145/12487871_1440_2560_60fps.mp4".to_string(),
                            source: Source::Url,
                            thumbnail_url: "https://tm.ibxk.com.br/2022/03/07/07013050568001.jpg".to_string(),
                            duration: 180,
                            mime_type: "video/mp4".to_string(),
                            season: None,
                        }
                    ]
                }
            ]
        }
    ];

    let movies = vec![
        Movie{
            id: "1".to_string(),
            title: "Vingadores - Ultimato".to_string(),
            thumbnail_url:"https://ichef.bbci.co.uk/ace/ws/640/cpsprodpb/BF0D/production/_106090984_2e39b218-c369-452e-b5be-d2476f9d8728.jpg.webp".to_string(),
            description: "Os Vingadores se reúnem para desfazer as ações de Thanos e restaurar o universo.".to_string(),
            media: "db/movies/radio_pesadelo.mkv".to_string(),
            source: Source::File,
            duration: 180,
            mime_type: "video/mp4".to_string(),
            genre: "Ação".to_string(),
        }
    ];

    let state = State { movies, series };
    let mut labels = HashMap::new();
    labels.insert("service".to_string(), "primepobre".to_string());

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .wrap(Logger::new(
                "%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\"",
            ))
            .app_data(Data::new(state.clone()))
            .service(
                scope("/api/v1")
                    .service(get_movies)
                    .service(get_series)
                    .service(stream_movie)
                    .service(stream_serie_episode),
            )
            .service(Files::new("/", "public").show_files_listing())
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
