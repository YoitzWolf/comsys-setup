use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};

use yew::prelude::*;
use yew::{function_component, classes, html, Html, Callback};

use crate::context::{GlobalContext};
use crate::reqs::{drop_me_shortcut, get_access_shortcut};

use super::simple::*;

#[function_component(BaseHeader)]
pub fn base_header() -> Html {
    //let screen_width = use_state(
    //    || get_viewport_width()
    //);
    //web_sys::console::log_1(&format!("> Size: {:?}", screen_width).into());
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

#[function_component(UserView)]
pub fn user_view() -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");
    let show_login_view_onclick = {
        let ctx = ctx.clone();
        Callback::from(
            move |ev| {
                web_sys::window()
                    .expect("Unable to get web_sys::window()")
                    .document()
                    .expect("Unable to get web_sys::window().document()")
                    .get_element_by_id("login_window")
                    .expect("login_window noderef not found")
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
                let mut doc = web_sys::window()
                    .expect("Unable to get web_sys::window()")
                    .document()
                    .expect("Unable to get web_sys::window().document()");
                let login = doc
                    .get_element_by_id("login-input")
                    .unwrap()
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .value();
                let password = doc
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
            move |_: MouseEvent| {
                drop_me_shortcut(ctx.clone());
            }
        )
    };
    let hide_login_view_onclick = {
        Callback::from(
            move |e: MouseEvent| {
                let form = e.target().unwrap().dyn_ref::<HtmlElement>().unwrap().clone();
                if form.id().eq(&"login_window") {
                    web_sys::window()
                        .expect("Unable to get web_sys::window()")
                        .document()
                        .expect("Unable to get web_sys::window().document()")
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
            <div class="user-block">
                //<div>
                    <div class="avatar"></div>
                    <div>{"username"}</div>
                //</div>
            </div>
            //<Button text={"Выйти".to_string()} onclick={drop_login_onclick}/>
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