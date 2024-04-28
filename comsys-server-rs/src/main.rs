use std::env;
use std::sync::Arc;
use std::time::Duration;

use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncPgConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

use dotenvy::dotenv;

use tokio::sync::Mutex;
use tonic::codegen::http::{HeaderName, HeaderValue};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower_http::cors::{AllowOrigin, Any, Cors, CorsLayer};
use tower_cookies::{Cookie, CookieManager, CookieManagerLayer, Cookies};
use tonic_build::Service;

use crate::gen::auth::authentication_server::AuthenticationServer;

mod server;
mod gen;
mod auth_backend;
mod schema;
mod models;
mod db_mng;

use server::AuthService;


use tracing::{debug, error, info, span, warn, Level};
use tracing_subscriber;
use tracing_appender;

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 4] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin", "access-token"];
const DEFAULT_ALLOW_HEADERS: [&str; 7] =
    [
        "x-grpc-web",
        "content-type",
        "x-user-agent",
        "grpc-timeout",
        "access-control-allow-credentials",
        "access-control-allow-headers",
        "access-token"
    ];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let file_appender = tracing_appender::rolling::daily("./", "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .init();
    
    let mut tgen = auth_backend::tokens::TokenGenerator::with_secet(b"#/-K@L1YUU~GaD@r|<20Ul$0u!%U/");

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    /*let db_con = AsyncPgConnection::establish(&database_url).await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));*/

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Pool::builder(config).build()?;

    let addr = "127.0.0.1:50051".parse().unwrap();
    let auther = AuthenticationServer::new(
        AuthService::new(
            Arc::new(Mutex::new(tgen)),
            Arc::new(Mutex::new(pool))
        )
    );

    println!("AuthServer listening on {}", addr);
    
    Server::builder()
        .accept_http1(true)
        //.tls_config(ServerTlsConfig::new().identity(identity))?
        .layer(
            CorsLayer::new()
                //.allow_origin(AllowOrigin::mirror_request())
                .allow_origin(
                    AllowOrigin::predicate(|origin, _| {
                        origin.as_bytes().starts_with(b"https://")
                    })
                    /*AllowOrigin::exact(
                        HeaderValue::from_str(&"https://127.0.0.1").unwrap()
                    )*/
                )
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(
                    DEFAULT_EXPOSED_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>()
                )
                .allow_headers(
                    DEFAULT_ALLOW_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>()
                ),
        )
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(auther)
        .serve(addr)
        .await?;

    Ok(())
}