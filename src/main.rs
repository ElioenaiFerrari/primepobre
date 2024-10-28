mod models;
use actix_cors::Cors;
use actix_files::Files;
use models::*;

use actix_web::{
    get,
    web::{scope, Data, Path},
    App, HttpResponse, HttpServer, Responder,
};

#[get("/movies")]
async fn get_movies(state: Data<State>) -> impl Responder {
    HttpResponse::Ok().json(&state.movies)
}
#[get("/movies/{id}")]
async fn get_movie(state: Data<State>, id: Path<String>) -> impl Responder {
    let movie = state.movies.iter().find(|m| m.id == id.to_string());
    match movie {
        Some(m) => HttpResponse::Ok().json(m),
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
                            video_url: "https://www.youtube.com/watch?v=5TbW4Zp8Z2A".to_string(),
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
            video_url: "https://www.youtube.com/watch?v=5TbW4Zp8Z2A".to_string(),
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
            .app_data(Data::new(state.clone()))
            .service(scope("/api/v1").service(get_movies).service(get_series))
            .service(Files::new("/", "public").show_files_listing())
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
