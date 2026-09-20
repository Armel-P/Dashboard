use actix_files::Files;
use actix_web::{App, HttpServer, web};

mod api;
mod db;
// mod entities;
// mod models;

pub type Result<T> = anyhow::Result<T>;

use actix_files::NamedFile;
use actix_web::HttpRequest;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use anyhow::{anyhow};

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

async fn spa(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path = req.path();

    if path.starts_with("/api/") {
        return Err(actix_web::error::ErrorNotFound("not found"));
    }

    if path.starts_with("/public/") && let Some((_, p)) = path.trim_start_matches('/').split_once('/') {
        return Ok(NamedFile::open(format!("./web/dist/{p}"))?);
    }

    Ok(NamedFile::open("./web/dist/index.html")?)
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

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

    let db_url = std::env::var("DATABASE_URL").expect("Error: Missing 'DATABASE_URL' env variable.");

    let db = db::connect(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Error connecting to Database: {e}"))?;

    let ssl = parse_args(&mut addr, &mut port).ok_or(anyhow::anyhow!("Error parsing arguments."))?;

    log::info!("Listening {addr}:{port}...");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(api::configure)
            .service(Files::new("/assets", "./web/dist/assets"))
            .default_service(web::route().to(spa))
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
