use std::io::Cursor;
use std::path::Path;
use std::sync::{Mutex};

use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, Data};
use log::info;

struct AppState {
    current_image: Mutex<Vec<u8>>,
}

#[get("/")]
async fn mainpage() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[get("/get")]
async fn get_image(req: HttpRequest) -> impl Responder {
    let state = req.app_data::<Data<AppState>>().unwrap();
    let current_img = state.current_image.lock().unwrap();
    let cloned_img = current_img.clone();
    drop(current_img);

    info!("Sending bytes with length {}", cloned_img.len());

    Bytes::from(cloned_img)
}


#[post("/new")]
async fn set_image(req: HttpRequest, body: Bytes) -> impl Responder {
    let state = req.app_data::<Data<AppState>>().unwrap();
    let mut current_image = state.current_image.lock().unwrap();

    *current_image = body.to_vec();
    info!("Received bytes with length {}", current_image.len());

    HttpResponse::build(StatusCode::OK)
}


fn read_jpeg(p: &Path) -> Vec<u8> {
    let img = image::io::Reader::open(p).expect("image file to be readable");
    let mut bytes: Vec<u8> = Vec::new();
    img.decode().expect("decode to succeed").write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Jpeg(90)).expect("image writing to succeed");
    bytes
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let first_img = read_jpeg(Path::new("static/first_frame.jpeg"));
    let app_data = Data::new(AppState {
        current_image: Mutex::new(first_img),
    });

    HttpServer::new(move || {
        App::new()
            .service(mainpage)
            .service(web::scope("/thestream")
                .app_data(app_data.clone())
                .service(get_image)
                .service(set_image)
            )
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}