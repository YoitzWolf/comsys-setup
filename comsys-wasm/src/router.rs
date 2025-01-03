use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use tokio::io::AsyncRead;
use yew::platform::time::sleep;
use yew::{function_component, Html, html};
use yew_router::prelude::*;
use yew::prelude::*;

use crate::components::cabinet::UserCabinet;
use crate::components::{BaseHeader, BaseTemplate, MainComponent};
use crate::components::comp_cardview::CompetitionCard;
use crate::components::comps::{CompetitionListViewer, ModComp, NewComp};
use crate::context::GlobalContext;
use crate::components::Button;
use crate::grpc::auth::UserView;
use crate::grpc::comp_handler::eq_message::{self, Message};
use crate::grpc::comp_handler::{EqHistoryMessage, EqMessage, TryNext, Verification, VoteMessage};
use crate::grpc::generic;
use crate::components::comp_handler::*;

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
    #[at("/webapp/comps/running/:id")]
    RunningComp{id: i32},
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
                WebAppRoute::RunningComp{id} => html! {
                    <CompHandlerApp cid={id}/>
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
    let CID = 6;
    let runcl = Callback::from(
        {
            let ctx = ctx.clone();
            move |_| {
                let ctx = ctx.clone();
                wasm_bindgen_futures::spawn_local(
                    async move {
                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                        let res = client.run(generic::Id{id: CID}).await;
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
                        //web_sys::console::log_1(&format!(">> {:?}", ctx.user).into());
                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                        let res = client.push_eq_message(
                            EqMessage{
                                comp_id: CID,
                                author: Some(
                                    UserView{
                                        uid: -1,
                                        login: ctx.user.get_user_data().clone().unwrap().get_name().to_string(),
                                        selfname: ctx.user.get_user_data().clone().unwrap().selfname.clone()
                                    }
                                ),
                                signature: "BOBA".to_string(),
                                message: Some(
                                    Message::VoteMessage(
                                        VoteMessage{
                                            author: Some(
                                                UserView{
                                                    uid: -1,
                                                    login: ctx.user.get_user_data().clone().unwrap().get_name().to_string(),
                                                    selfname: ctx.user.get_user_data().clone().unwrap().selfname.clone()
                                                }
                                            ),
                                            queue_id: 0,
                                            action_id: 1,
                                            mark_type: "Исполнение".to_string(),
                                            mark: 10 
                                        }
                                    )
                                ) 
                            }
                        ).await;
                        web_sys::console::log_1(&format!(">> {:?}", res).into());
                    }
                );
            }
        }
    );

    let say_shit2 = Callback::from(
        {
            let ctx = ctx.clone();
            move |_| {
                let ctx = ctx.clone();
                wasm_bindgen_futures::spawn_local(
                    async move {
                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                        let res = client.push_eq_message(
                            EqMessage{
                                comp_id: CID,
                                author: Some(UserView{
                                    uid: -1,
                                    login: ctx.user.get_user_data().clone().unwrap().get_name().to_string(),
                                    selfname: ctx.user.get_user_data().clone().unwrap().selfname.clone()
                                }),
                                signature: "BOBA".to_string(),
                                message: Some(
                                    Message::TryNext(
                                        TryNext { queue_id: 0 }
                                    )
                                ) 
                            }
                        ).await;
                        web_sys::console::log_1(&format!(">> {:?}", res).into());
                    }
                );
            }
        }
    );

    let text = use_state(|| {"start".to_string()});

    html! {
        <>
            <div class={"stack"}>    
                <h1>{"Testing Page"}</h1>
                <Button text="run" onclick={runcl} />
                <Button text="send vote" onclick={say_shit} />
                <Button text="next" onclick={say_shit2} />
                <CompHandlerApp cid={CID}/>
            </div>
        </>
    }
}

pub fn switch_users(route: UsersRoute) -> Html {
    match route {
        UsersRoute::Bare => html! { <> { "Not implemented" } </> },
        UsersRoute::New => html! { <> { "Not implemented" } </> },
        UsersRoute::Cabinet => html! {
            <UserCabinet/>
        }
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