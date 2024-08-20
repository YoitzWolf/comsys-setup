use std::fmt::Debug;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::{function_component, classes, html, Html};
use yew_router::prelude::Link;


use crate::context::{GlobalContext};
use crate::reqs::*;
use crate::router::{Route, WebAppRoute};

#[derive(Properties, PartialEq)]
pub struct BaseTemplateProps{
    pub children: Html,
}

#[function_component(BaseTemplate)]
pub fn base_template(prop: &BaseTemplateProps) -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");
    
    html!(
        <>
            {prop.children.clone()}
        </>
    )
}


#[function_component(MainComponent)]
pub fn main_component(prop: &BaseTemplateProps) -> Html {
    let ctx =  use_context::<GlobalContext>().expect("no ctx found");

    let drop_login_onclick = {
        let ctx = ctx.clone();
        Callback::from(
            move |_: MouseEvent| {
                drop_me_shortcut(ctx.clone());
            }
        )
    };

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
                            <li>
                                <Link<WebAppRoute>
                                    to={WebAppRoute::Comps}
                                >{"мои соревнования"}</Link<WebAppRoute>>
                            </li>
                            <li onclick={drop_login_onclick}>{"выход"}</li>
                        </ul>
                    </div>
                }
            </div>
        </>
    )
}