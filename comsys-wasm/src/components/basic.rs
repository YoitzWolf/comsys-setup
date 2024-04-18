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


#[function_component(BaseHeader)]
pub fn base_header() -> Html {
    html!(
        <div class={classes!("header")}>
            <h1 class={classes!("col-1")}>{"ComSys"}</h1>
            <button class="menu-button"/>
            <ul class={classes!("menu")}>
                <li class={classes!("btn")}><a href={"/"}>{"Главная"}</a></li>
                <li class={classes!("btn")}>{"О приложении"}</li>
            </ul>
            <UserView/>
        </div>
    )
}

#[autoprops]
#[function_component(Button)]
pub fn button(text: &String, onclick: Callback<MouseEvent>) -> Html {
    html!(
        <button onclick={ onclick } type={ "button".to_string() } class={ classes!("user-view", "btn") } >
            { text }
        < / button >
    )
}

#[function_component(UserView)]
pub fn user_view() -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");
    let show_login_view_onclick = {
        let ctx = ctx.clone();
        Callback::from(
            move |ev| {
                //web_sys::console::log_1(&"Login event starting!".to_string().into());
                //web_sys::console::log_1(&format!("Ctx: {:?}", ctx).to_string().into());
                //get_access_shortcut(ctx.clone(), "123".to_string(),"123".to_string());
                //login_view_shown.set(true);
                gloo::utils::document()
                    .get_element_by_id("login_window")
                    .unwrap()
                    .set_class_name(
                        "global_fill"
                    );
            }
        )
    };

    let login_onclick = {
        let ctx = ctx.clone();
        Callback::from(
            move |ev| {
                //web_sys::console::log_1(&"Login event starting!".to_string().into());
                //web_sys::console::log_1(&format!("Ctx: {:?}", ctx).to_string().into());

                //login_view_shown.set(true);
                let login = gloo::utils::document()
                    .get_element_by_id("login-input")
                    .unwrap()
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .value();
                let password = gloo::utils::document()
                    .get_element_by_id("password-input")
                    .unwrap()
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .value();
                get_access_shortcut(ctx.clone(), login, password);
            }
        )
    };

    let drop_login_onclick = {
        let ctx = ctx.clone();
        Callback::from(
            move |_| {
                drop_me_shortcut(ctx.clone());
            }
        )
    };

    let hide_login_view_onclick = {
        Callback::from(
            move |e: MouseEvent| {
                let form = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().clone();
                if form.id().eq(&"login_window") {
                    gloo::utils::document()
                        .get_element_by_id("login_window")
                        .unwrap()
                        .set_class_name(
                            "global_fill_hiding"
                        );
                }

            }
        )
    };

    html!(
        <>
        if (ctx.user.get_token()).is_none() {

            //if (*login_view_shown) {
                <div id={"login_window"} class={classes!("global_fill_hiding", "hidden")} onclick={hide_login_view_onclick}>
                    <div class={classes!("col-5", "card")} onclick={|_| {}}>
                        <span>{"Вход"}</span>
                        <input id="login-input"    type="login" placeholder="login"/>
                        <input id="password-input" type="password" placeholder="password"/>
                        <Button text={"Войти".to_string()} onclick={login_onclick}/>
                    </div>
                </div>
            //}
            <Button text={"Войти".to_string()} onclick={show_login_view_onclick}/>
        } else {
            <Button text={"Выйти".to_string()} onclick={drop_login_onclick}/>
            /*if ctx.user.ready() {
                <h1>
                    // user view
                    { ctx.user.get_user_data().as_ref().unwrap().get_name() }
                </h1>
            } else {
                <h1>
                    {"unknown"}
                </h1>
            }*/
        }
        </>
    )
}