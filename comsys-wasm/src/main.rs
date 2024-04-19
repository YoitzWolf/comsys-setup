use std::sync::Arc;
use yew::prelude::*;
use yew::props;

mod components;
mod context;
mod grpc;
mod reqs;

use components::*;
use crate::context::{Context, GlobalContext, GlobalContextProvider};

#[function_component]
fn App() -> Html {
    //let ctx = use_reducer_eq(|| Context::default());

    html! {
        <GlobalContextProvider>
            <BaseTemplate>
                <BaseHeader/>
                <MainComponent>
                    <div>{"F"}</div>
                </MainComponent>
                <div class={"footer"}><div>{"footer here"}</div></div>
            </BaseTemplate>
        </GlobalContextProvider>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}