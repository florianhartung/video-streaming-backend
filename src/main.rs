use std::io::Cursor;
use std::path::Path;
use std::sync::Mutex;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use actix_web::http::StatusCode;
use actix_web::web::Bytes;

struct AppState {
    current_image: Mutex<Vec<u8>>,
}

#[get("/")]
async fn mainpage() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[get("/thestream")]
async fn get_image(data: web::Data<AppState>) -> impl Responder {
    let cloned_img = data.current_image.lock().unwrap().clone();

    Bytes::from(cloned_img)
}


#[post("/thestream")]
async fn set_image(req_body: String, data: web::Data<AppState>) -> impl Responder {
    let mut current_image = data.current_image.lock().unwrap();

    *current_image = req_body.as_bytes().to_vec();

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
    let first_img = read_jpeg(Path::new("../static/first_frame.jpeg"));

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                current_image: Mutex::new(first_img),
            }))
            .service(mainpage)
            .service(get_image)
            .service(set_image)
        // .service(echo)
        // .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}