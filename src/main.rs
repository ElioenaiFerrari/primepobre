mod models;
use actix_cors::Cors;
use actix_files::Files;
use models::*;

use actix_web::{
    get, http::header::{HeaderValue, RANGE}, middleware::Logger, web::{scope, Data, Path}, App, HttpRequest, HttpResponse, HttpServer, Responder
};
use reqwest::Client;
use tokio_stream::StreamExt;

#[get("/movies")]
async fn get_movies(state: Data<State>) -> impl Responder {
    HttpResponse::Ok().json(&state.movies)
}
#[get("/movies/{id}")]
async fn get_movie(state: Data<State>, path: Path<String>, req: HttpRequest) -> impl Responder {
    let id = path.into_inner();
    let movie = state.movies.iter().find(|m| m.id == id.to_string());
    match movie {
        Some(m) => {
            let video_url = &m.video_url;
            let mime_type = &m.mime_type;

            // Create a reqwest client
            let client = Client::new();

            // Get the range header if present
            let range_header = req.headers().get(RANGE);
            let range = if let Some(range_header) = range_header {
                range_header.to_str().ok()
            } else {
                None
            };

            // Make the request to the remote video URL with the range header
            let response = client
                .get(video_url)
                .header("Range", range.unwrap_or("bytes=0-"))
                .send()
                .await
                .map_err(|_| HttpResponse::InternalServerError().finish())
                .unwrap();

            if !response.status().is_success() {
                return HttpResponse::NotFound().finish();
            }

            // Get the content range from the response
            let content_range = response.headers().get("Content-Range").cloned();

            // Create a stream of the response body
            let stream = response.bytes_stream();

            // Build the response
            let mut http_response = HttpResponse::PartialContent()
                .content_type(mime_type.clone())
                .streaming(stream);

            if let Some(content_range) = content_range {
                let actix_header_value =
                    HeaderValue::from_str(&content_range.to_str().unwrap()).unwrap();
                http_response
                    .headers_mut()
                    .insert("Content-Range".parse().unwrap(), actix_header_value);
            }

            http_response
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/movies/{id}/stream")]
async fn stream_movie(state: Data<State>, path: Path<String>) -> impl Responder {
    let id = path.into_inner();
    let movie = state.movies.iter().find(|m| m.id == id.to_string());
    match movie {
        Some(m) => HttpResponse::Ok()
            .content_type(m.mime_type.clone())
            .body(m.video_url.clone()),
        None => HttpResponse::NotFound().finish(),
    }
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
                            video_url: "https://videos.pexels.com/video-files/28851690/12495824_360_640_30fps.mp4".to_string(),
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
            video_url: "https://videos.pexels.com/video-files/28769620/12472889_360_640_24fps.mp4".to_string(),
            duration: 180,
            mime_type: "video/mp4".to_string(),
            genre: "Ação".to_string(),
        }
    ];

    let state = State { movies, series };
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\""))
            .app_data(Data::new(state.clone()))
            .service(
                scope("/api/v1")
                    .service(get_movies)
                    .service(get_series)
                    .service(get_movie)
                    .service(stream_movie),
            )
            .service(Files::new("/", "public").show_files_listing())
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
