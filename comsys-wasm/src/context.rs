use std::rc::Rc;
use std::sync::{Arc};
use tonic_web_wasm_client::options::{Credentials, FetchOptions};


use yew::prelude::*;
use yew_autoprops::autoprops;

use crate::grpc::auth::{Token, TokenType};
use crate::grpc::auth::authentication_client::AuthenticationClient;


#[derive(Clone, Debug, Default)]
pub struct UserData {
    username: String
}

impl UserData {
    pub fn get_name(&self) -> &str {
        &self.username
    }
}


#[derive(Clone, Debug)]
pub struct UserContext {
    auth_token: Option<Token>,
    user_data: Option<UserData>,
}

impl Default for UserContext {
    fn default() -> Self {
        Self {
            auth_token: None,
            user_data: None
        }
    }
}

impl UserContext {

    pub fn ready(&self) -> bool {
        self.auth_token.is_some()
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
        (
            self.auth_token.is_some() && other.get_token().is_some()
        ) || (
            self.auth_token.is_none() && other.get_token().is_none()
        )
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub user: UserContext,
    auth_client: AuthenticationClient<tonic_web_wasm_client::Client>
}

impl std::default::Default for Context {
    fn default() -> Self {
        //web_sys::console::log_1(&"Default Context creating!".to_string().into());
        Self {
            user: UserContext::default(),
            auth_client: Self::get_auth_grpc_client(),
        }

    }
}


//pub type GlobalContextProvider = yew::ContextProvider< Context >;
pub type GlobalContext = UseReducerHandle<Context>;

#[autoprops]
#[function_component(GlobalContextProvider)]
pub fn global_context_provider(children: &Html) -> Html {
    let ctx = use_reducer(|| Context::default() );

    html! {
        <ContextProvider<GlobalContext> context={ctx}>
            {children.clone()}
        </ContextProvider<GlobalContext>>
    }
}




pub enum ContextAction {

    SetupAuth(Token),
    DropAuth

}

impl Reducible for Context {
    type Action = ContextAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {

        match action {
            ContextAction::SetupAuth(t) => {
                let mut newc = self.as_ref().clone();
                web_sys::console::log_1(&format!("Auth token installation: {:?}", newc.user.set_token(t)).into());
                return Rc::new(newc);
            }
            ContextAction::DropAuth => {
                let mut newc = self.as_ref().clone();
                newc.user.clear();
                return Rc::new(newc);
            }
             _ => {
                 self
             }
        }
    }
}

impl Context {

    pub fn get_web_client() -> tonic_web_wasm_client::Client {
        tonic_web_wasm_client::Client::new_with_options(
            format!("{}/api", web_sys::window().unwrap().location().origin().unwrap().to_string()).to_string(),
            FetchOptions::new().credentials(Credentials::Include)
        )
    }

    pub fn get_auth_grpc_client() -> AuthenticationClient<tonic_web_wasm_client::Client> {
        AuthenticationClient::new(Self::get_web_client())
    }
}

impl PartialEq for Context {
    fn eq(&self, other: &Self) -> bool {
        self.user.same_state(&other.user)
    }
}