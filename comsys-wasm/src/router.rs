use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use tokio::io::AsyncRead;
use yew::platform::time::sleep;
use yew::{function_component, Html, html};
use yew_router::prelude::*;
use yew::prelude::*;

use crate::components::{BaseHeader, BaseTemplate, MainComponent};
use crate::components::comp_cardview::CompetitionCard;
use crate::components::comps::{CompetitionListViewer, ModComp, NewComp};
use crate::context::GlobalContext;
use crate::components::Button;
use crate::grpc::comp_handler::eq_message::Message;
use crate::grpc::comp_handler::{EqHistoryMessage, EqMessage, VoteMessage};
use crate::grpc::generic;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/webapp")]
    WebAppBare,
    #[at("/webapp/*")]
    WebApp,
}

#[derive(Clone, Routable, PartialEq)]
pub enum WebAppRoute {
    #[at("/webapp")]
    Bare,
    #[at("/webapp/testing")]
    Test,
    #[at("/webapp/users/*")]
    Users,
    #[at("/webapp/comps")]
    Comps,
    #[at("/webapp/comps/new")]
    NewComp,
    #[at("/webapp/comps/mod/:id")]
    ModComp{id: i32},
    #[at("/webapp/comps/view/:id")]
    ViewComp{id: i32},
    #[at("/webapp/comps/running/*")]
    RunningComp
}

#[derive(Clone, Routable, PartialEq)]
pub enum UsersRoute {
    #[at("/webapp/users/*")]
    Bare,
    #[at("/webapp/users/new")]
    New,
    #[at("/webapp/users/cabinet")]
    Cabinet,
}


//#[derive(Clone, Routable, PartialEq)]
// enum CompProcessingApp {
//  #[at("/webapp/comps/processing/:id/*")]
//  Some
//}

pub fn switch_main(route: Route) -> Html {
    html! {
        <BaseTemplate>
            {
                match route {
                    Route::Home => html!{ <Home/> },
                    Route::WebAppBare => html!{ html! { <Switch<WebAppRoute> render={switch_webapp} /> } },
                    Route::WebApp => html!{ html! { <Switch<WebAppRoute> render={switch_webapp} /> } },
                }
            }
            <div class={"footer"}><div>{"footer here"}</div></div>
        </BaseTemplate>
    }
}


pub fn switch_webapp(route: WebAppRoute) -> Html {
    html!{
        <>
        <BaseHeader/>
        <MainComponent>
        {
            match route {
                WebAppRoute::Bare => html! { <Redirect<Route> to={Route::Home}/> },

                WebAppRoute::Test => html!{
                    <TestUnit/>
                },

                WebAppRoute::Users => html!{ html! { <Switch<UsersRoute> render={switch_users} /> } },
                WebAppRoute::Comps => html! {
                    <CompetitionListViewer/>
                },
                WebAppRoute::NewComp => html! {
                    <NewComp/>
                },
                WebAppRoute::ModComp{id} => html! {
                    <ModComp cid={id}/>
                },
                _ => html! { <Redirect<Route> to={Route::Home}/> },
            }
        }
        </MainComponent>
        </>
    }
}

#[function_component(TestUnit)]
pub fn test_unit () -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");

    let runcl = Callback::from(
        {
            let ctx = ctx.clone();
            move |_| {
                let ctx = ctx.clone();
                wasm_bindgen_futures::spawn_local(
                    async move {
                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                        let res = client.run(generic::Id{id: 1}).await;
                        web_sys::console::log_1(&format!(">> {:?}", res).into());
                    }
                );
            }
        }
    );

    let say_shit = Callback::from(
        {
            let ctx = ctx.clone();
            move |_| {
                let ctx = ctx.clone();
                wasm_bindgen_futures::spawn_local(
                    async move {
                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                        let res = client.push_eq_message(
                            EqMessage{ comp_id: 1, author: None, signature: "BOBA".to_string(), message: Some(Message::VoteMessage(VoteMessage{..Default::default()})) }
                        ).await;
                        web_sys::console::log_1(&format!(">> {:?}", res).into());
                    }
                );
            }
        }
    );

    let text = use_state(|| {"start".to_string()});

    let listen = Callback::from(
        {   
            let text = text.clone();
            //let ctx = ctx.clone();
            move |_| {
                text.set("Clicked".to_string());
                web_sys::console::log_1(&"Start streaming request..".into());
                let ctx = ctx.clone();
                let text = text.clone();
                let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                wasm_bindgen_futures::spawn_local(
                    async move {
                        text.set("WAIT FOWU STWEAMING".to_string());
                        let mut stream_response = client.start_eq_message_stream(generic::Id{id:1})
                            .await
                            .expect("Stream expected..")
                            .into_inner();
                        text.set("WAIT FOWU STWEAMING MEWSAGE".to_string());
                        loop {
                            web_sys::console::log_1(&"WAIT FOWU STWEAMING MEWSAWGE".into());
                            text.set("WAIT FOWU STWEAMING MEWSAWGE".to_string());
                            let response = stream_response.message().await.expect("stream message");
                            text.set(format!("STWEAMINGW:: {:?}", response));
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                );
            }
        }
    );

    html! {
        <>
            <h1>{"Testing Page"}</h1>
            <Button text="run" onclick={runcl} />
            <Button text="send" onclick={say_shit} />
            <Button text="start listen" onclick={listen} />
            <a>{(text.clone()).to_string()}</a>
        </>
    }
}

pub fn switch_users(route: UsersRoute) -> Html {
    html! {
        <>
            { "Users" }
        </>
    }
}


#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <>
            <BaseHeader/>
            <MainComponent>
                <div class={"stack"}>
                        <h1>{"Главная страница"}</h1>
                        <div>
                            {"Система проведения оценки соревнований."}
                        </div>
                </div>
            </MainComponent>
        </>
    }
}