use std::ops::{BitAndAssign, BitXor};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};

use yew::prelude::*;
use yew::{function_component, classes, html, Html, Callback};
use yew_router::prelude::Link;

use crate::context::{GlobalContext};
use crate::reqs::{drop_me_shortcut, get_access_shortcut, registration_shortcut};
use crate::router::Route;

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
                <li>
                    <Link<Route>
                        classes={classes!("btn")}
                        to={Route::Home}
                    >{"Главная"}</Link<Route>>
                </li>
                <li><a class={classes!("btn")}>{"О приложении"}</a></li>
            </ul>
            <UserView/>
        </div>
    )
}

#[function_component(UserView)]
pub fn user_view() -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");
    let login_selected = use_state_eq(|| {true});
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
                get_access_shortcut(ctx.clone(), login, password, Callback::from(|_| {}));
            }
        )
    };

    let register_onclick = {
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
                let password_repeat = doc
                    .get_element_by_id("password-input-repeat")
                    .unwrap()
                    .dyn_ref::<HtmlInputElement>()
                    .unwrap()
                    .value();
                if !password.eq(&password_repeat) {
                    // TODO Error show
                } else {
                    let supervisor_code = doc
                        .get_element_by_id("code-input")
                        .unwrap()
                        .dyn_ref::<HtmlInputElement>()
                        .unwrap()
                        .value();

                    registration_shortcut(ctx.clone(), login.clone(), password.clone(), supervisor_code,
                    {
                        let ctx = ctx.clone();
                        Callback::from(
                            move |x| {
                                match x {
                                    Ok((login, password)) => {
                                        get_access_shortcut(ctx.clone(), login, password, Callback::from(|_| {}));
                                    },
                                    Err(_) => {

                                    }
                                }
                            }
                        )
                    }
                    );
                }
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

    let change_user_menu_visibility = {
        Callback::from(
            move |e: MouseEvent| {
                //web_sys::console::log_1(&format!("Change user menu-sidebar visibility").into());
                let mut sidebar = web_sys::window()
                    .unwrap()//.expect("Unable to get web_sys::window()")
                    .document()
                    .unwrap()//.expect("Unable to get web_sys::window().document()")
                    .get_element_by_id("side-user-menu")
                    .unwrap();//.expect("Unable to get user side-bar menu element!");
                if sidebar.class_list().contains("hidden") { sidebar.class_list().remove_1("hidden"); }
                else { sidebar.class_list().add_1("hidden"); }
                //web_sys::console::log_1(&format!("vis: {}", sidebar.class_list().value()).into());
            }
        )
    };

    let logreg_switch = {
        let login_selected = login_selected.clone();
        Callback::from(
            move |e: MouseEvent| {
                login_selected.set(! *login_selected);
            }
        )
    };

    html!(
        <>
        if (ctx.user.get_token()).is_none() {
            //if (*login_view_shown) {
                <div id={"login_window"} class={classes!("global_fill_hiding", "hidden")} onclick={hide_login_view_onclick}>
                    <div class={classes!("col-5", "card")} onclick={|_| {}}>
                    if (*login_selected) {
                        <span>{"Вход"} </span>
                        <input id="login-input"    type="text" placeholder="login"/>
                        <input id="password-input" type="password" placeholder="password"/>
                        <Button text={"Войти".to_string()} onclick={login_onclick}/>
                        <NoStyleButton text={"создать новый аккаунт".to_string()} onclick={logreg_switch}/>
                    } else {
                        <span>{"Регистрация"} </span> 
                        <input id="login-input"    type="text" placeholder="login"/>
                        <input id="password-input" type="password" placeholder="password"/>
                        <input id="password-input-repeat" type="password" placeholder="password repeat"/>
                        <input id="code-input" type="text" placeholder="Код приглашения"/>
                        <Button text={"Зарегистрировать Пользователя".to_string()} onclick={register_onclick}/>
                        <NoStyleButton text={"уже есть аккаунт".to_string()} onclick={logreg_switch}/>
                    }
                        
                    </div>
                </div>
            //}
            <Button text={"Войти".to_string()} onclick={show_login_view_onclick}/>
        } else {
            <div class="user-block" onclick={change_user_menu_visibility}>
                //<div>
                    <div class="avatar"></div>
                    /*<div>{"username"}</div>*/
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