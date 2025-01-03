use std::rc::Rc;
use std::sync::Arc;
//use http_body_util::combinators::UnsyncBoxBody;
//use hyper::body::Bytes;
//use hyper_util::client::legacy::connect::HttpConnector;
//use bytes::Bytes;

//use http_body_util::Full;
//use tonic::client::GrpcService;
use tonic::codegen::InterceptedService;

use tonic_web_wasm_client::Client;
use tonic_web_wasm_client::options::{Credentials, FetchOptions};

//use hyper_util::rt::TokioExecutor;
//use hyper_util::client::legacy::Client as Client;
//use tonic::service::interceptor::InterceptorLayer;
use tonic::Status;
//use tonic_web::{GrpcWebCall, GrpcWebClientLayer, GrpcWebClientService, GrpcWebService};

use tower::Layer;
use yew::prelude::*;
use yew_autoprops::autoprops;
use crate::grpc::auth::{Token, TokenType, UserView};
use crate::grpc::auth::authentication_client::AuthenticationClient;
use crate::grpc::auth_client_interceptor::AuthInterceptor;
use crate::grpc::comp::competition_declarator_client::CompetitionDeclaratorClient;
use crate::grpc::comp_handler::competition_handler_client::CompetitionHandlerClient;
use crate::grpc::users::user_manage_client::UserManageClient;
use crate::reqs::{get_auth_shortcut, get_me_shortcut};


#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserData {
    // uid
    pub(crate) uid: i32,
    // login
    pub(crate) username: String,
    // selfname
    pub(crate) selfname: String
}

impl UserData {
    pub fn get_selfname(&self) -> &str {
        &self.selfname
    }

    pub fn get_name(&self) -> &str {
        &self.username
    }
}


#[derive(Clone, Debug, Default)]
pub struct UserContext {
    auth_token: Option<Token>,
    user_data: Option<UserData>,
}

impl From<UserData> for UserView {
    fn from(value: UserData) -> Self {
        UserView {
            uid: value.uid,
            login: value.username,
            selfname: value.selfname,
        }
    }
}

impl UserContext {
    pub fn ready(&self) -> bool {
        self.auth_token.is_some() && self.user_data.is_some()
    }

    pub fn clear(&mut self) {
        self.auth_token = None;
        self.user_data = None;
    }

    pub fn get_user_data(&self) -> &Option<UserData> {
        &self.user_data
    }

    pub fn set_token(&mut self, token: Token) -> Result<(), ()> {
        if token.token_type.is_some_and(
            |tt| { tt.eq(&(TokenType::Auth as i32)) }
        ){
            self.auth_token = Some(token);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_data(&mut self, data: UserData) -> Result<(), ()> {
        self.user_data = Some(data);
        Ok(())
    }

    pub fn drop_token(&mut self) {
        self.auth_token = None;
    }

    pub fn get_token_arc(&self) -> Arc<&Option<Token>> {
        Arc::new(&self.auth_token)
    }

    pub fn get_token(&self) -> &Option<Token> {
        &self.auth_token
    }

    pub fn same_state(&self, other: &Self) -> bool {
        /*(
            self.auth_token.is_some() && other.get_token().is_some() && (
                self.auth_token.clone().unwrap()
            )
        ) || (
            self.auth_token.is_none() && other.get_token().is_none()
        ) && (
            (self.user_data.is_none() && other.user_data.is_none())
             ||
            (self.user_data.is_some() && other.user_data.is_some() && (
                self.user_data.clone().unwrap() == other.user_data.clone().unwrap()
            )) 
        )*/
        (
            match (self.auth_token.clone(), other.auth_token.clone()) {
                (Some(x), Some(y)) => {
                    x.value.eq(&y.value)
                },
                (None, None) => true,
                _ => false
            }
        ) && (
            match (self.user_data.clone(), other.user_data.clone()){
                (Some(x), Some(y)) => {
                    x == y
                },
                (None, None) => true,
                _ => false
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub user: UserContext,
    //auth_client: AuthenticationClient<tonic_web_wasm_client::Client>
}

impl std::default::Default for Context {
    fn default() -> Self {
        //web_sys::console::log_1(&"Default Context creating!".to_string().into());
        Self {
            user: UserContext::default(),
            //auth_client: Self::get_auth_grpc_client(),
        }

    }
}


//pub type GlobalContextProvider = yew::ContextProvider< Context >;
pub type GlobalContext = UseReducerHandle<Context>;

#[autoprops]
#[function_component(GlobalContextProvider)]
pub fn global_context_provider(children: &Html) -> Html {
    let ctx = use_reducer(|| Context::default() );

    if ctx.user.get_token().is_none() {
        get_auth_shortcut(
            ctx.clone(),
            Callback::from(|r:Result<GlobalContext, Status>| {})
        );
    } else if ctx.user.get_user_data().is_none() {
        get_me_shortcut(ctx.clone(), Callback::from(|_|{}));
    }

    html! {
        <ContextProvider<GlobalContext> context={ctx}>
            {children.clone()}
        </ContextProvider<GlobalContext>>
    }
}

pub enum ContextAction {

    SetupAuth(Token),
    SetupData(UserData),
    DropAuth

}

impl Reducible for Context {
    type Action = ContextAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ContextAction::SetupAuth(t) => {
                let mut newc = self.as_ref().clone();
                web_sys::console::log_1(&format!("Auth token installation: {:?}", newc.user.set_token(t)).into());
                Rc::new(newc)
            }
            ContextAction::DropAuth => {
                let mut newc = self.as_ref().clone();
                newc.user.clear();
                Rc::new(newc)
            }
            ContextAction::SetupData(d) => {
                let mut newc = self.as_ref().clone();
                //web_sys::console::log_1(&format!("User Data loaded: {:?}", d).into());
                newc.user.set_data(UserData{username: d.username, uid: d.uid, selfname: d.selfname}).unwrap();
                Rc::new(newc)
            },
             /*_ => {
                 self
             }*/
        }
    }
}

impl Context {
    pub fn get_web_client() -> Client {
        Client::new_with_options(
            format!("{}/api", web_sys::window().unwrap().location().origin().unwrap().to_string()).to_string(),
            FetchOptions::new().credentials(Credentials::Include)
        )
    }

    pub fn get_auth_grpc_client() -> AuthenticationClient<Client> {
        AuthenticationClient::new(Self::get_web_client())
    }

    pub fn get_comp_grpc_client(ctx: &Self) -> CompetitionDeclaratorClient<InterceptedService<Client, AuthInterceptor>> {
        CompetitionDeclaratorClient::with_interceptor(Self::get_web_client(), AuthInterceptor::new(ctx))
    }

    pub fn get_comp_handler_grpc_client(ctx: &Self) -> CompetitionHandlerClient<InterceptedService<Client, AuthInterceptor>> {
        CompetitionHandlerClient::with_interceptor(Self::get_web_client(), AuthInterceptor::new(ctx))
    }

    pub fn get_user_mng_grpc_client(ctx: &Self) -> UserManageClient<InterceptedService<Client, AuthInterceptor>> {
        UserManageClient::with_interceptor(Self::get_web_client(), AuthInterceptor::new(ctx))
    }

    /*
    pub fn get_web_client() -> LocClient {

        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
        let svc = tower::ServiceBuilder::new()
            .layer(GrpcWebClientLayer::new())
            .service(client);
        svc
        //Client::new_with_options(
        //    format!("{}/api", web_sys::window().unwrap().location().origin().unwrap().to_string()).to_string(),
        //    FetchOptions::new().credentials(Credentials::Include)
        //)
    }

    pub fn get_web_client_intercepted(ctx: &Self) -> InterceptedLocClient {

        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
        let svc = tower::ServiceBuilder::new()
            .layer(GrpcWebClientLayer::new())
            .service(InterceptedService::new(client, AuthInterceptor::new(ctx)));
        svc
    }

    fn back_uri() -> tonic::codegen::http::Uri {
        format!("{}/api", web_sys::window().unwrap().location().origin().unwrap().to_string()).to_string().try_into().unwrap()
    }

    pub fn get_auth_grpc_client() -> AuthenticationClient<LocClient> {
        AuthenticationClient::with_origin(Self::get_web_client(), Self::back_uri() )
    }

    pub fn get_comp_grpc_client(ctx: &Self) -> CompetitionDeclaratorClient<InterceptedLocClient> {

        let client: Client<HttpConnector, UnsyncBoxBody<Bytes, Status>> = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
        let svc = tower::ServiceBuilder::new()
            .layer(GrpcWebClientLayer::new())
            .service(client);
    
        //let intercepted = InterceptedService::new(svc, AuthInterceptor::new(ctx));

       let with_origin = CompetitionDeclaratorClient::with_origin(svc, Self::back_uri());
        todo!()
    }

    pub fn get_comp_handler_grpc_client(ctx: &Self) -> CompetitionHandlerClient<InterceptedLocClient> {
        //CompetitionHandlerClient::with_origin(Self::get_web_client_intercepted(ctx), Self::back_uri());
        todo!()
    }*/


}
//type LocClient = GrpcWebClientService<Client<HttpConnector, GrpcWebCall<http_body_util::combinators::UnsyncBoxBody<Bytes, Status>>>>;
//type InterceptedLocClient = GrpcWebClientService<InterceptedService<Client<HttpConnector, GrpcWebCall<http_body_util::combinators::UnsyncBoxBody<Bytes, Status>>>, AuthInterceptor>>;

impl PartialEq for Context {
    fn eq(&self, other: &Self) -> bool {
        self.user.same_state(&other.user)
    }
}