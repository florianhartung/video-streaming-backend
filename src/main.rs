use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use actix_web::http::StatusCode;
use actix_web::rt::{spawn, time};
use actix_web::web::{Bytes, Data};
use log::info;

struct AppState {
    current_image: Mutex<Vec<u8>>,
    bytes_sent: AtomicUsize,
    bytes_received: AtomicUsize,
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

    state.bytes_sent.fetch_add(cloned_img.len(), Ordering::Acquire);

    Bytes::from(cloned_img)
}


#[post("/new")]
async fn set_image(req: HttpRequest, body: Bytes) -> impl Responder {
    let state = req.app_data::<Data<AppState>>().unwrap();
    let mut current_image = state.current_image.lock().unwrap();

    *current_image = body.to_vec();

    state.bytes_received.fetch_add(current_image.len(), Ordering::Acquire);

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
    env_logger::builder().filter_level(log::LevelFilter::Info).build();

    let first_img = read_jpeg(Path::new("static/first_frame.jpeg"));
    let app_data = Data::new(AppState {
        current_image: Mutex::new(first_img),
        bytes_sent: AtomicUsize::new(0),
        bytes_received: AtomicUsize::new(0),
    });

    let a = app_data.clone();
    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let r = a.bytes_received.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| Some(0)).unwrap();
            let s = a.bytes_sent.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |a| Some(0)).unwrap();
            info!("Received: {r}B/s Sent: {s}B/s")
        }
    });

    info!("Starting server");
    HttpServer::new(move || {
        App::new()
            .service(mainpage)
            .service(web::scope("/thestream")
                .app_data(app_data.clone())
                .service(get_image)
                .service(set_image)
            )
    })
        .bind(("0.0.0.0", 80))?
        .run()
        .await
}