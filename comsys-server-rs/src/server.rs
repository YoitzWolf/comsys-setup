use std::net::SocketAddr;
use serde_json::to_string_pretty;
use serde_json::Value::String;
use tonic::{transport::Server, Request, Response, Status};
use tonic::metadata::MetadataValue;

use tonic_web::ResponseFuture;
use tower_cookies::{Cookie, Cookies};
use tower_cookies::cookie::SameSite;
use tower_cookies::cookie::time::Duration;


use crate::gen::auth::authentication_server::{Authentication, AuthenticationServer};
use crate::gen::auth::*;

use tracing::{debug, error, info, span, warn, Level, instrument};
use tracing_subscriber;

#[derive(Default, Debug)]
pub struct AuthService{

}

#[tonic::async_trait]
impl Authentication for AuthService {
    #[instrument]
    async fn authorize(&self, request: Request<AuthRequest>) -> Result<Response<AuthResult>, Status> {
        //info!(target: "gRPC: ", "Received new authorize request: {:?}", request);
        let token_value = "AccessTokenValue".to_string();

        let is_web = request.metadata().contains_key("x-grpc-web".to_string());
        let mut resp = Response::new(AuthResult {
            result: Some(auth_result::Result::Token(
                Token{
                    value: if is_web {"access token uses cookies for grpc-web".into()} else {token_value.clone()},
                    token_type: Some(TokenType::Access.into())
                }
            ))
        });
        if is_web { // Setup Cookies
            let mut addr = "".to_string();
            if let Some(r) = request.metadata().get("Host") {
                addr = r.to_str().unwrap_or(&"").to_string()
            }
            resp.metadata_mut().insert(
                "set-cookie",
                //format!("access-token={token_value};Domain=127.0.0.1;Secure=true;Path=/api/auth.Authentication/GetAuth;HttpOnly=true;Max-Age=2678400;SameSite=strict").to_string().parse().unwrap() // HttpOnly
                tower_cookies::Cookie::build(("access-token", token_value))
                    .path("/api/auth.Authentication/GetAuth")
                    .domain(addr)
                    .http_only(true)
                    .max_age(Duration::days(31))
                    .same_site(SameSite::None)
                    .secure(true)
                    .build().to_string().parse().unwrap()
            );
        }
        Ok(
            resp
        )
    }

    #[instrument]
    async fn get_auth(&self, request: Request<GetAuthTokenRequest>) -> Result<Response<AuthResult>, Status> {
        let meta = request.metadata();
        let mut resp = Response::new(AuthResult {
            result: None
        });
        //info!(target: "METADATA: ", "meta: {:?}", meta);
        if let Some(d) = meta.get("cookie") {
            {
                //info!(target: "gRPC: ", "Cookies found!");
                for i in tower_cookies::Cookie::split_parse(d.to_str().unwrap()) {
                    if let Ok(cook) = i {
                        match cook.name() {
                            "access-token" => {
                                resp = Response::new(AuthResult {
                                    result: Some(auth_result::Result::Token(
                                        Token {
                                            value: "this-is-great-auth-token".to_string(),
                                            token_type: Some(TokenType::Auth.into()),
                                        }
                                    ))
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(
            resp
        )
    }

    async fn refresh_access(&self, request: Request<RefreshAccessTokenRequest>) -> Result<Response<AuthResult>, Status> {
        todo!()
    }

    async fn drop_token(&self, request: Request<DropTokenRequest>) -> Result<Response<DropResult>, Status> {
        //info!(target: "gRPC: ", "Received new authorize request: {:?}", request);
        let mut addr = "".to_string();

        let (meta, ext, req ) = request.into_parts();
        let is_web = meta.contains_key("x-grpc-web".to_string());
        let allowed = true;

        let drop_completed = allowed && true;

        let mut resp = Response::new(DropResult { is_done: allowed });

        if  drop_completed &&
            is_web &&
            req.to_drop.is_some_and(
                |t| {
                    t.token_type.is_some_and(
                        |tt| { tt.eq(&(TokenType::Access as i32)) }
                    )
                }
            )
             {
                 if let Some(r) = meta.get("Host") {
                     addr = r.to_str().unwrap_or(&"").to_string()
                 }
                 resp.metadata_mut().insert(
                "set-cookie",
                    tower_cookies::Cookie::build(("access-token", "removed") )
                        .path("/api/auth.Authentication/GetAuth")
                        .domain(addr)
                        .http_only(true)
                        .max_age(Duration::nanoseconds(0))
                        .same_site(SameSite::None)
                        .secure(true)
                        .build().encoded().to_string().parse().unwrap()

                );
        }

        Ok(
            resp
        )
    }

    async fn registration(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResult>, Status> {
        todo!()
    }
}

