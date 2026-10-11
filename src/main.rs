use actix_files::{Files, NamedFile};
use actix_web::{App, HttpServer, HttpResponse, web,
                dev::{ServiceRequest, ServiceResponse, fn_service}};
use db::DbPool;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use anyhow::{anyhow};
use reqwest::Client;

mod api;
pub mod schema;
pub mod queries;
pub mod structs;
pub mod constants;
pub mod crypto;
mod db;
pub mod middleware;
pub mod password;
pub mod utils;
pub mod services;

use services::registery::Registry;
use structs::oauth::PendingAuths;

pub type Result<T> = anyhow::Result<T>;

fn parse_args(addr: &mut String, port: &mut u16) -> Option<bool> {
    let mut args = std::env::args();
    let (mut i, len) = (0, args.len());
    let mut arg = args.next();
    while arg.is_some() {
        match arg?.as_str() {
            "--ssl" => return Some(true),
            "-b" | "--bind" => {
                if len - i < 2 {
                    log::error!("Bad Usage: bind needs 2 arguments (address & port)");
                    return None;
                }
                if let Some(a) = args.next() {
                    *addr = a;
                }
                if let Some(p) = args.next()
                    && let Ok(pn) = p.parse::<u16>()
                {
                    *port = pn;
                }
            }
            _ => {}
        }
        arg = args.next();
        i += 1;
    }
    Some(false)
}

async fn spa_fallback(req: ServiceRequest) -> actix_web::Result<ServiceResponse> {
    let (req, _) = req.into_parts();

    if req.path().starts_with("/api/") {
        let res = HttpResponse::NotFound().finish();
        return Ok(ServiceResponse::new(req, res));
    }

    let file = NamedFile::open(format!("{}/index.html", constants::WEB_ROOT))?;
    let res = file.into_response(&req);
    Ok(ServiceResponse::new(req, res))
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let jwt_key = match std::env::var("SECRET") {
        Ok(s) => s.into_bytes(),
        Err(err) => { println!("Error: {}", err); return Err(anyhow!(err)) }
    };

    let mut addr = match std::env::var("SERVER_HOST") {
        Ok(host) => host,
        Err(err) => { println!("Error: {}", err); return Err(anyhow!(err)) }
    };

    let mut port = match std::env::var("SERVER_PORT") {
        Ok(port) => { match port.parse::<u16>() {
            Ok(port) => port,
            Err(err) => { println!("Error: {}", err); return Err(anyhow!(err)) }
        }
        },
        Err(err) => { println!("Error: {}", err); return Err(anyhow!(err)) }
    };

    let db_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => { println!("Error: {}", err); return Err(anyhow!(err)) }
    };

    db::run_migrations(&db_url).await?;
    let db: DbPool = db::connect(&db_url)?;
    let db = web::Data::new(db);

    let ssl = parse_args(&mut addr, &mut port).ok_or(anyhow::anyhow!("Error parsing arguments."))?;

    log::info!("Listening {addr}:{port}...");

    let pending = web::Data::new(PendingAuths::new());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .app_data(web::Data::new(jwt_key.clone()))
            .app_data(web::Data::new(Registry::new()))
            .app_data(web::Data::new(Client::new()))
            .app_data(pending.clone())
            .configure(api::configure)
            .service(
                Files::new("/", constants::WEB_ROOT)
                    .index_file("index.html")
                    .default_handler(fn_service(spa_fallback)),
            )
    });

    let bind = if ssl {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file("key.pem", SslFiletype::PEM)
            .unwrap_or_default();
        builder
            .set_certificate_chain_file("cert.pem")
            .unwrap_or_default();

        server.bind_openssl((addr, port), builder)?
    } else {
        server.bind((addr, port))?
    };

    bind.run().await?;
    Ok(())
}
