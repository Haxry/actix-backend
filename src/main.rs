use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};
use anyhow::Result;
use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
#[get("/")]
async fn hello()-> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder{
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

//getting the path parameters
#[get("/users/{user_id}/{friend}")] // <- define path parameters
async fn index(path: web::Path<(u32, String)>,data: web::Data<AppState>) -> impl Responder {
    let (user_id, friend) = path.into_inner();
    // Increment the global count
    data.global_count.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().body(format!("Welcome {}, user_id {}!", friend, user_id))
    //Ok(format!("Welcome {}, user_id {}!", friend, user_id))
}


//getting the query parameters
//dont forget to add serde to your Cargo.toml
//dont forget to add the derive feature to serde
#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[get("/search")]
async fn search(info: web::Query<SearchQuery>) -> String{
    format!("Searching for: {}", info.query)
}

//handling json data
#[derive(Deserialize,Serialize)]
struct Info {
    username: String,
    email: String,
}
#[post("/submit")]
async fn submit(info: web::Json<Info>) -> impl Responder {
    HttpResponse::Ok().json(info) 
}
//adding global and local state
#[derive(Clone)]
struct AppState {
    local_count: Cell<usize>,
    global_count: Arc<AtomicUsize>,
}
#[get("/count")]
async fn show_count(data: web::Data<AppState>) -> impl Responder {
    format!(
        "global_count: {}\nlocal_count: {}",
        data.global_count.load(Ordering::Relaxed),
        data.local_count.get()
    )
}

#[get("/add")]
async fn add_one(data: web::Data<AppState>) -> impl Responder {
    data.global_count.fetch_add(1, Ordering::Relaxed);

    let local_count = data.local_count.get();
    data.local_count.set(local_count + 1);

    format!(
        "global_count: {}\nlocal_count: {}",
        data.global_count.load(Ordering::Relaxed),
        data.local_count.get()
    )
}

#[get("/accountinfo/{pubkey}")]
async fn accountinfo(path:web::Path<String>) -> impl Responder {
    let pubkey_str = path.into_inner();
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    let pubkey = Pubkey::from_str(&pubkey_str)
        .map_err(|_| HttpResponse::BadRequest().body("Invalid public key")).unwrap();

    // This returns Result<Account, ClientError>, not Option<Account>
    let account = client
        .get_account(&pubkey).await.unwrap();
        

    HttpResponse::Ok().json(account)
}

#[get("/accountbalance/{pubkey}")]
async fn accountbalance(path:web::Path<String>) -> impl Responder {
    let pubkey_str = path.into_inner();
    let client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let pubkey = Pubkey::from_str(&pubkey_str)
        .map_err(|_| HttpResponse::BadRequest().body("Invalid public key")).unwrap();
    let balance = client
        .get_balance(&pubkey)
        .await
        .map_err(|_| HttpResponse::InternalServerError().body("Failed to get balance"))
        .unwrap();
    HttpResponse::Ok().body(format!("Balance for {}: {}", pubkey, balance))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
     let data = AppState {
        local_count: Cell::new(0),
        global_count: Arc::new(AtomicUsize::new(0)),
    };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(data.clone())) // Share AppState across handlers
            .service(
                web::scope("/api")
                    .service(hello)
                    .service(echo)
                    .service(index)
                    .service(search)
                    .service(submit)
                    .service(show_count)
                    .service(add_one)
                    .service(accountinfo)
                    .service(accountbalance),
            )
            .route("/manual", web::get().to(manual_hello))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}