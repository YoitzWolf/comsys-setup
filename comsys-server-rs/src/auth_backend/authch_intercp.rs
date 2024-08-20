use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Status, Request};
use tonic::server::NamedService;
use tonic::service::Interceptor;
use tonic_async_interceptor::AsyncInterceptor;
use crate::auth_backend::tokens::TokenGenerator;
use crate::auth_service::GetAuthor;

#[derive(Clone)]
pub struct AuthCheckInterceptor {
    pub token_generator: Arc<Mutex<TokenGenerator>>
    /*token_generator: Arc<Mutex<TokenGenerator>>,
    db_con: Arc<Mutex<Pool<AsyncPgConnection>>>*/
}

impl AuthCheckInterceptor {
    pub fn new(
        token_generator: Arc<Mutex<TokenGenerator>>,
        /*db_con: Arc<Mutex<Pool<AsyncPgConnection>>>*/
    ) -> Self {
        Self {
            token_generator
        }
    }

    pub fn authenticate(
        &self,
        mut request: Request<()>,
    ) -> impl Future<Output = Result<Request<()>, Status>> + Send + 'static {
        let tgen = self.token_generator.clone();
        let metadata = request.metadata().clone(); // Clone the request metadata
        async move {
            let att = metadata.get("auth-token");
            if let Some(hv) = att {
                if let Ok(x) = hv.to_str() {
                    match tgen.lock().await.decode(request.get_author(), x.to_string()).await {
                        Ok(x) => {
                            request.extensions_mut().insert(x.claims);
                            Ok(request)
                        }
                        Err(e) => {Err(Status::permission_denied("Wrong token!"))}
                    }
                } else {
                    Err(Status::unauthenticated("Invalid Header"))
                }
            } else {
                Err(Status::unauthenticated("Need auth"))
            }
        }
    }
}

impl AsyncInterceptor for AuthCheckInterceptor {
    type Future = Pin<Box<dyn Future<Output = Result<Request<()>, Status>> + Send + 'static>>;

    fn call(&mut self, request: Request<()>) -> Self::Future {
        let fut = self.authenticate(request);
        Box::pin(fut)
    }
}
