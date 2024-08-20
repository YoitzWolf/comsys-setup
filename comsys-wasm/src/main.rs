use yew::prelude::*;
use yew_router::{BrowserRouter, Switch};

mod components;
mod context;
mod grpc;
mod reqs;
mod utils;
mod router;
mod fs;
mod xslx_conv;

use crate::context::{GlobalContextProvider};
use crate::router::{Route, switch_main};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <GlobalContextProvider>
            <BrowserRouter>
                <Switch<Route> render={switch_main} />
            </BrowserRouter>
        </GlobalContextProvider>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}