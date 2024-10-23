use std::env;

use actix_web::web::Bytes;
use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use mime_guess::from_path;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
struct VideoFile {
    id: u64,
    quality: String,
    file_type: String,
    width: Option<u32>,
    height: Option<u32>,
    link: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoPicture {
    id: u64,
    picture: String,
    nr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Video {
    id: u64,
    width: u32,
    height: u32,
    url: String,
    image: String,
    duration: u32,
    user: User,
    video_files: Vec<VideoFile>,
    video_pictures: Vec<VideoPicture>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PexelsResponse {
    page: u32,
    per_page: u32,
    total_results: u32,
    url: String,
    videos: Vec<Video>,
}

async fn search_video(term: &String) -> Result<PexelsResponse, reqwest::Error> {
    let api_url = env::var("PEXELS_API_URL").expect("API_URL must be set");
    let api_key = env::var("PEXELS_API_KEY").expect("API_KEY must be set");

    let url = format!(
        "{}/videos/search?query={}&locale=pt-BR&size=10",
        api_url, term
    );

    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", api_key)
        .send()
        .await?
        .json::<PexelsResponse>()
        .await?;

    Ok(response)
}

async fn video_stream() -> HttpResponse {
    let terms = vec![
        "código",
        "natureza",
        "tecnologia",
        "academia",
        "esportes",
        "música",
    ];
    let mut rng = rand::thread_rng();
    let rand_index = rng.gen_range(0..terms.len());
    let term = terms[rand_index];
    let response = search_video(&term.to_string()).await.unwrap();
    let first_video = &response.videos[0];

    let link = &first_video.video_files[0].link;
    let mime_type = &first_video.video_files[0].file_type;
    let client = Client::new();
    let response = match client.get(link).send().await {
        Ok(response) => response,
        Err(_) => return HttpResponse::NotFound().body("Failed to fetch video"),
    };

    if !response.status().is_success() {
        return HttpResponse::NotFound().body("Failed to fetch video");
    }

    // Determina o tipo MIME do arquivo de vídeo

    // Cria um canal para enviar os dados de forma assíncrona
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(32);

    // Cria uma task assíncrona para ler e enviar o vídeo por pedaços (chunks)
    actix_web::rt::spawn(async move {
        let mut stream = response.bytes_stream();

        // Lê os dados em chunks e envia via o canal
        while let Some(bytes) = stream.next().await {
            match tx.send(Ok(bytes.unwrap())).await {
                Ok(_) => {
                    if tx.is_closed() {
                        break;
                    }
                }
                Err(_) => {
                    if tx.is_closed() {
                        break;
                    }

                    if tx
                        .send(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to send data",
                        )))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    // Retorna a resposta HTTP com o corpo de streaming
    // Retorna a resposta HTTP com o corpo de streaming
    HttpResponse::Ok()
        .content_type(mime_type.to_string())
        .streaming(ReceiverStream::new(rx))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Carrega as variáveis de ambiente do arquivo .env
    HttpServer::new(|| {
        App::new().route("/video", web::get().to(video_stream)) // Rota para o streaming de vídeo
    })
    .bind(("127.0.0.1", 8080))? // Porta do servidor
    .run()
    .await
}
