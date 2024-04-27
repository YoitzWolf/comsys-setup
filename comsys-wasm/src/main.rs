use std::sync::Arc;
use yew::prelude::*;
use yew::props;

mod components;
mod context;
mod grpc;
mod reqs;
mod utils;

use components::*;
use crate::components::competitions::CompetitionCard;
use crate::context::{Context, GlobalContext, GlobalContextProvider};

#[function_component(App)]
fn app() -> Html {
    //let ctx = use_reducer_eq(|| Context::default());

    //let selfctx = use_context::<App>().expect("no ctx found");

    html! {
        <GlobalContextProvider>
            <BaseTemplate>
                <BaseHeader/>
                <MainComponent>
                    <div class={"stack"}>
                        {
                            (0..5).into_iter().map(|_| {
                                html!{
                                    <CompetitionCard/>
                                }
                            }).collect::<Vec<Html>>()
                        }
                    </div>
                </MainComponent>
                <div class={"footer"}><div>{"footer here"}</div></div>
            </BaseTemplate>
        </GlobalContextProvider>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}