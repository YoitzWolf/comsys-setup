use std::time::Duration;
use tonic::codegen::http::{HeaderName, HeaderValue};

use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower_http::cors::{AllowOrigin, Any, Cors, CorsLayer};
use tower_cookies::{Cookie, CookieManager, CookieManagerLayer, Cookies};


use crate::gen::auth::authentication_server::AuthenticationServer;

use tonic::transport::server::Routes;
use tonic_build::Service;
use tonic_web::GrpcWebService;

mod server;
mod gen;

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


    let addr = "127.0.0.1:50051".parse().unwrap();
    let auther = AuthenticationServer::new(AuthService::default()); //tonic_web::enable();

    //let pem = tokio::fs::read("./cert.pem").await.unwrap();
    //let cert = tonic::transport::Certificate::from_pem(pem);
    //tonic::transport::ClientTlsConfig::new().ca_certificate(cert);

    println!("AuthServer listening on {}", addr);

    //let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "./"]);
    /*let cert = std::fs::read_to_string("./127.0.0.1.pem")?;
    let key = std::fs::read_to_string( "./127.0.0.1-key.pem")?;

    let identity = Identity::from_pem(cert, key);*/

    Server::builder()
        .accept_http1(true)
        //.tls_config(ServerTlsConfig::new().identity(identity))?
        .layer(
            CorsLayer::new()
                //.allow_origin(AllowOrigin::mirror_request())
                .allow_origin(AllowOrigin::exact(
                        HeaderValue::from_str(&"https://127.0.0.1").unwrap()
                    )
                )
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(
                    DEFAULT_EXPOSED_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                )
                .allow_headers(
                    DEFAULT_ALLOW_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                ),
        )
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(auther)
        .serve(addr)
        .await?;

    Ok(())
}