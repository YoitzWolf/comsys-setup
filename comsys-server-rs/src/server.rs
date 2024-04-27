use std::fmt::{Debug, Formatter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use argon2::PasswordVerifier;
use diesel_async::AsyncPgConnection;
use jsonwebtoken::TokenData;
use serde_json::to_string_pretty;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use tonic::metadata::MetadataValue;
use diesel_async::pooled_connection::deadpool::Pool;

use tonic_web::ResponseFuture;
use tower_cookies::{Cookie, Cookies};
use tower_cookies::cookie::SameSite;
use tower_cookies::cookie::time::Duration;


use crate::gen::auth::authentication_server::{Authentication, AuthenticationServer};
use crate::gen::auth::*;

use tracing::{debug, error, info, span, warn, Level, instrument};
use tracing_subscriber;
use crate::auth_backend::tokens::{TokenClaim, TokenGenerator};

pub trait GetAuthor{
    fn get_author(&self) -> String;
}

impl<T> GetAuthor for Request<T>{
    fn get_author(&self) -> String {
        format!("{}/{}", self.remote_addr().unwrap_or(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 666)
        ), "123")
    }
}


fn get_access_cookie<'a>(addr: String, token : (String, i32), ttl:i64) -> tower_cookies::Cookie<'a> {
    tower_cookies::Cookie::build(("access-token", token.0))
        .path("/api/auth.Authentication/GetAuth")
        //.path("/api/auth.Authentication/Refresh")
        .domain(addr)
        .http_only(true)
        .max_age(Duration::seconds(ttl))
        .same_site(SameSite::None)
        .secure(true)
        .build()
}

//#[derive(Default, Debug)]
pub struct AuthService {
    token_generator: Arc<Mutex<TokenGenerator>>,
    db_con: Arc<Mutex<Pool<AsyncPgConnection>>>
}

impl Debug for AuthService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthService")
    }
}

impl AuthService {
    pub(crate) fn new(token_generator: Arc<Mutex<TokenGenerator>>, db_con: Arc<Mutex<Pool<AsyncPgConnection>>>) -> Self {
        Self {
            token_generator,//: TokenGenerator::new(),
            db_con
        }
    }
}

#[tonic::async_trait]
impl Authentication for AuthService {
    #[instrument]
    async fn authorize(&self, request: Request<AuthRequest>) -> Result<Response<AuthResult>, Status> {
        //info!(target: "gRPC: ", "Received new authorize request: {:?}", request);
        let author = request.get_author();
        let (meta, ext, req) = request.into_parts();

        let mut correctreq = false;
        let mut this_user: Option<crate::models::User> = None;
        {
            if let Ok(mut con) = self.db_con.lock().await.get().await {
                let db_resp = crate::db_mng::user_mng::get_by_login(&mut con, &req.login).await;
                if let Ok(user) = db_resp {
                    if let Ok(hash) = argon2::PasswordHash::new(&user.hash) {
                        correctreq = argon2::Argon2::default().verify_password(req.password.as_bytes(), &hash).is_ok();
                        if correctreq { this_user = Some(user); }
                    }
                }
            } else {
                return Result::Err(Status::internal("Database Error"));
            }
        };

        if correctreq && this_user.is_some() { // PASSWORD CORRECT -> //&& let Some(this_user) = this_user
            let this_user = this_user.unwrap();
            //println!("{:?}", this_user);
            let mut claim = TokenClaim::random(
                TokenType::Access.into(),
                author,
                this_user.login,
            );

            let token = self.token_generator.lock().await.generate(
                &mut claim
            ).await.unwrap_or(("generation_error".to_string(), 0));

            let mut resp: Response<AuthResult>;
            let is_web = meta.contains_key("x-grpc-web".to_string());
            if is_web { // GRPC-WEB BRANCH
                let mut addr = "".to_string();
                if let Some(r) = meta.get("Host") {
                    addr = r.to_str().unwrap_or(&"").to_string() // Setup address used to access service via user's agent
                }
                resp = Response::new(AuthResult {
                    result: Some(auth_result::Result::Token(
                        Token {
                            value: "grpc-web-uses-cookies".to_string(),
                            token_type: Some(token.1),
                        }
                    ))
                });
                resp.metadata_mut().insert(
                    "set-cookie",
                    //format!("access-token={token_value};Domain=127.0.0.1;Secure=true;Path=/api/auth.Authentication/GetAuth;HttpOnly=true;Max-Age=2678400;SameSite=strict").to_string().parse().unwrap() // HttpOnly
                    get_access_cookie(
                        addr,
                        token,
                        self.token_generator.lock().await.get_acc_ttl() as i64
                    ).to_string().parse().unwrap()
                );
            } else { // NO GRPC-WEB BRANCH
                resp = Response::new(AuthResult {
                    result: Some(auth_result::Result::Token(
                        Token {
                            value: token.0,
                            token_type: Some(token.1),
                        }
                    ))
                });
            }
            Ok(resp)
        } else { // UNABLE TO VERIFY PASSWORD ->
            Ok (
                Response::new(
                    AuthResult{
                        result: Some(auth_result::Result::Error(AuthFailError::InvalidData.into())),
                    }
                )
            )
        }
    }

    #[instrument]
    async fn get_auth(&self, request: Request<GetAuthTokenRequest>) -> Result<Response<AuthResult>, Status> {
        let meta = request.metadata();
        let author = request.get_author();
        let mut resp = Response::new(AuthResult {
            result: None
        });
        //info!(target: "METADATA: ", "meta: {:?}", meta);

        let mut access_token_data: Option<TokenData<TokenClaim>> = None;

        if let Some(d) = meta.get("cookie") {
            // GRPC WEB
                //info!(target: "gRPC: ", "Cookies found!");
                for i in tower_cookies::Cookie::split_parse(d.to_str().unwrap()) {
                    if let Ok(cook) = i {

                        match cook.name() {
                            "access-token" => { // Valid answer starts here
                                let access = self.token_generator
                                    .lock().await
                                    .decode(
                                        author.clone(),
                                        cook.value().to_string()
                                    ).await;
                                //info!(target: "gRPC: ", ">>{:?}", access);
                                match access {
                                    Ok(acc_token_data) => {  // Valid Access Token
                                        access_token_data = Some(acc_token_data);
                                    }
                                    Err(_) => {}
                                }
                            } // Valid answer ends here
                            _ => {}
                        }
                    }
                }
        } else {
            // CHECK REQUEST DATA
            if let Some(token) = request.into_inner().access_token {
                let tval = token.value;
                let access = self.token_generator.lock().await.decode(author.clone(), tval).await;
                match access {
                    Ok(acc_token_data) => {  // Valid Access Token
                        access_token_data = Some(acc_token_data);
                    }
                    Err(_) => {}
                }
            }

        }
        //info!(target: "TOKEN", ">>{:?}", access_token_data);
        if let Some(token) = access_token_data {
            // ASK DB HERE

            let mut claim = TokenClaim::random(TokenType::Auth as i32, author, token.claims.user_id);
            if let Ok(autht) = self.token_generator.lock().await.generate(
                &mut claim
            ).await {
                resp = Response::new(AuthResult {
                    result: Some(auth_result::Result::Token(
                        Token {
                            value: autht.0,
                            token_type: Some(autht.1),
                        }
                    ))
                });
            }
        } else {
            resp = Response::new(
                AuthResult{
                    result: Some(
                        auth_result::Result::Error(AuthFailError::InvalidData as i32)
                    ),
                }
            )
        }

        Ok( resp )
    }

    async fn refresh_access(&self, request: Request<RefreshAccessTokenRequest>) -> Result<Response<AuthResult>, Status> {
        let (meta, ext, req ) = request.into_parts();
        let is_web = meta.contains_key("x-grpc-web".to_string());

        todo!()
    }

    async fn drop_token(&self, request: Request<DropTokenRequest>) -> Result<Response<DropResult>, Status> {
        //info!(target: "gRPC: ", "Received new authorize request: {:?}", request);
        let mut addr = "".to_string();
        // TODO add managenment of sessions. I.e remote control etc
        let (meta, ext, req ) = request.into_parts();
        let is_web = meta.contains_key("x-grpc-web".to_string());
        //let allowed = true;
        let drop_completed = true;
        let mut resp = Response::new(DropResult { is_done: drop_completed });

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
                    get_access_cookie(
                        addr,
                        ("removed".to_string(), 0),
                        0i64
                    ).to_string().parse().unwrap()

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

