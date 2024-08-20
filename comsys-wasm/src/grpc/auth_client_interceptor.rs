
use tonic::{Request, Status};
use tonic::metadata::{Ascii, MetadataValue};
use tonic::service::Interceptor;
use crate::context::Context;


pub struct AuthInterceptor {
    value: MetadataValue<Ascii>
}

impl AuthInterceptor {
    pub fn new(context: &Context) -> Self {
        let value:  MetadataValue<Ascii> = if let Some(t) = (context.user.get_token()) {
            t.value.clone()
        } else {
            "0".to_string()
        }.parse().expect("Invalid token chars??");

        Self{
            value
        }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.metadata_mut().insert("auth-token", self.value.clone());

        Ok(request)
    }
}