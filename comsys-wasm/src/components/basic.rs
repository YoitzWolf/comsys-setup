use std::fmt::Debug;
use tonic::{Request, Status};
use tonic_web_wasm_client::options::{Credentials, FetchOptions};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlFormElement, HtmlInputElement};
use yew_autoprops::autoprops;

use yew::prelude::*;
use yew::{function_component, classes, html, Html, Callback};
use yew::context::_ContextProviderProps::{children};


use crate::grpc::auth::*;
use authentication_client::*;
use crate::context::{ContextAction, GlobalContext};
use crate::grpc::auth::auth_result::Result::Token;
use crate::reqs::{drop_me_shortcut, get_access_shortcut, get_auth_shortcut};

use super::header::*;

#[derive(Properties, PartialEq)]
pub struct BaseTemplateProps{
    pub children: Html,
}

#[function_component(BaseTemplate)]
pub fn base_template(prop: &BaseTemplateProps) -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");
    
    if ctx.user.get_token().is_none() { 
        get_auth_shortcut(ctx); 
    }
    
    html!(
        <>
            {prop.children.clone()}
        </>
    )
}


#[function_component(MainComponent)]
pub fn main_component(prop: &BaseTemplateProps) -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");
    html!(
        <>
            <div class={classes!("main", "inline-fx-container")}>
                <div style={"min-height:90vh;"}>
                    {prop.children.clone()}
                </div>
                if ctx.user.ready() {
                    <div id={"side-user-menu"}>
                        <div style={"text-align:right;font-weight:bold;"}>
                            {"Меню пользователя"}
                        </div>
                        <ul>
                            <li>{"аккаунт"}</li>
                            <li>{"мои соревнования"}</li>
                            <li>{"выход"}</li>
                        </ul>
                    </div>
                }
            </div>
        </>
    )
}