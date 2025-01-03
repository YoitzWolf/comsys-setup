extern crate alloc;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use comp_handler_service::CompHandlerService;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

use r#gen::comp_handler::competition_handler_server::CompetitionHandlerServer;
use gen::users::user_manage_server::UserManageServer;
use tokio::sync::Mutex;
use tonic::codegen::http::HeaderName;
use tonic::transport::Server;
use tonic_async_interceptor::{async_interceptor, AsyncInterceptor};
use tower_http::cors::{AllowOrigin, Any, Cors, CorsLayer};
use tonic_build::Service;

mod auth_service;
mod users_service;
mod gen;
mod auth_backend;
mod competition_backend;
mod schema;
mod models;
mod db_mng;
mod comp_decl_service;
mod comp_handler_service;


use auth_service::AuthService;


use tracing::*;
use tracing_subscriber;
use tracing_appender;
use users_service::UsersService;
use crate::auth_backend::prelude::*;
use crate::comp_decl_service::CompDeclService;
use crate::gen::comp::competition_declarator_server::CompetitionDeclaratorServer;
use crate::gen::auth::authentication_server::AuthenticationServer;

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 5] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin", "access-token", "transfer-encoding"];
const DEFAULT_ALLOW_HEADERS: [&str; 8] =
    [
        "x-grpc-web",
        "content-type",
        "x-user-agent",
        "grpc-timeout",
        "access-control-allow-credentials",
        "access-control-allow-headers",
        "access-token",
        "transfer-encoding"
    ];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenvy::dotenv()?;

    let file_appender = tracing_appender::rolling::daily("./", "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_writer(non_blocking)
        .init();
    
    let tgen = Arc::new(Mutex::new(auth_backend::tokens::TokenGenerator::with_secet(env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set").as_bytes() )));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    /*let db_con = AsyncPgConnection::establish(&database_url).await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));*/

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Arc::new(Mutex::new(Pool::builder(config).build()?));

    let addr = env::var("BACKEND_HOST").expect("BACKEND_HOST must be set").parse().unwrap();

    let auther = AuthenticationServer::new(
        AuthService::new(
            Arc::clone(&tgen),
            Arc::clone(&pool)
        )
    );

    let auth_interceptor = AuthCheckInterceptor::new(Arc::clone(&tgen));

    let comper = CompetitionDeclaratorServer::new(
        CompDeclService::new(
            Arc::clone(&tgen),
            Arc::clone(&pool)
        )
    );

    let comp_handler = CompetitionHandlerServer::new(
        CompHandlerService::new(
            Arc::clone(&pool)
        )
    );

    let users = UserManageServer::new(
        UsersService::new(
            Arc::clone(&pool)
        )
    );

    let authed_comper_router = tower::ServiceBuilder::new()
        .layer(async_interceptor(auth_interceptor.clone()))
        .service(comper);

    let authed_comp_handler_router = tower::ServiceBuilder::new()
        .layer(async_interceptor(auth_interceptor.clone()))
        .service(comp_handler);

    let authed_users_router = tower::ServiceBuilder::new()
        .layer(async_interceptor(auth_interceptor.clone()))
        .service(users);
        //
        //.(comper).into_inner();

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
        .add_service(authed_comper_router)
        .add_service(authed_comp_handler_router)
        .add_service(authed_users_router)
        .serve(addr)
        .await?;

    Ok(())
}
