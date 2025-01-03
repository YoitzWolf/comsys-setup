use std::i32;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use web_sys::console::log_1;
use web_sys::{AbortController, HtmlElement, HtmlInputElement};
use yew::platform::time::sleep;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew::{function_component, html, Html};
use yew_autoprops::autoprops;

use crate::components::{context_wraps::*, HSpacer, LoadingView};
use crate::components::{Button, DenyButton, OkButton};
use crate::context::{Context, GlobalContext};
use crate::grpc::auth::UserView;
use crate::grpc::comp::comps_list::CompView;
use crate::grpc::comp::{participant, JudgeScheme, EasyParticipant};
use crate::grpc::comp_handler::eq_message::Message;
use crate::grpc::comp_handler::{eq_message, ActiveActionState, EqHistoryMessage, EqHistoryRequest, EqMessage, FixVotingMessage, TryNext, Verification, VerifyVoteMessage, VoteMessage};
use crate::grpc::generic::{self, Id};
use crate::grpc::users::{self, Judge, Role};
use crate::{components::Spacer, grpc::comp::Team};


#[autoprops]
#[function_component(VoteView)]
pub fn vote_view(
    coid: &i32,
    action: &i32,
    judge: &Judge,
    scheme: &JudgeScheme,
    team: &Team,
    #[prop_or(None)] approvement_id: Option<i32>,
    //#[prop_or(None)] participants: &Option<Vec<EasyParticipant>>,
    #[prop_or(None)] active_view: &Option<ActiveActionState>
) -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");
    let msg_ctx = use_context::<EventQueueMessageContext>().expect("no eqm ctx found");

    let approved: UseStateHandle<Option<i32>> = use_state_eq(|| {approvement_id});

    let particips ={
        //Some(p) => {
            format!("Участники: {}", team.participants.iter().map(|x| {x.name.clone()}).collect::<Vec<String>>().join(", ") )
        //},
        //None => {
        //    "".into() // format!("Номера участников: {}", team.participants.iter().map(|x| {x.to_string()}).collect::<Vec<String>>().join(", ") )
        //},
    };

    let mark_ref = use_node_ref();

    let send_vote = Callback::from(
        {
            let ctx = ctx.clone();
            let coid = coid.clone();
            let judge = judge.clone();
            let action = action.clone();
            let scheme = scheme.clone();
            let mark_ref = mark_ref.clone();
            let approved = approved.clone(); 
            move |_| {
                let ctx = ctx.clone();
                let coid = coid.clone();
                let judge = judge.clone();
                let action = action.clone();
                let scheme = scheme.clone();
                let mark_ref = mark_ref.clone();
                let approved = approved.clone();
                wasm_bindgen_futures::spawn_local(
                    async move {
                        let mark_input = &mark_ref;
                        //web_sys::console::log_1(&format!(">> {:?}", ctx.user).into());
                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                        let author = Some( UserView::try_from(ctx.user.get_user_data().clone().unwrap()).unwrap() );
                        let res = client.push_eq_message(
                            EqMessage{
                                comp_id: coid,
                                author: author.clone(),
                                signature: "SIGNATURE HERE".to_string(),
                                message: Some(
                                    Message::VoteMessage(
                                        VoteMessage{
                                            author: author.clone(),
                                            queue_id: judge.queue,
                                            action_id: action,
                                            mark_type: scheme.get_judgement_group_name(judge.mark_group).unwrap(),
                                            mark: mark_input.cast::<HtmlInputElement>().unwrap().value().parse().unwrap()
                                        }
                                    )
                                ) 
                            }
                        ).await;
                        web_sys::console::log_1(&format!(">> {:?}", res).into());
                        match res {
                            Ok(x) => {approved.set(Some(x.into_inner().id));},
                            Err(_) => {},
                        }
                    }
                );
            }
        }
    );

    let verdict_message = match ((*approved), active_view) {
        (Some(mmsg), Some(acv)) => {
            let s : Vec<_> = acv.marks.get(&scheme.get_judgement_group_name(judge.mark_group).unwrap()).unwrap().votes.iter().filter(
                |v| {
                    web_sys::console::log_1(&format!("Check> {}, {}", mmsg, v.message_id).into());
                    v.message_id.eq(&mmsg)
                }
            ).collect();
            if s.len() == 0 {
                html!{<span class={"marked-info"}>{"Оценка ожидает подтверждения"}</span>}
            } else {
                match s.first() {
                    Some(x) => match x.verifyed() {
                        crate::grpc::comp_handler::Verification::Approve => html!{<span class={"marked-ok"}>{"Оценка принята"}</span>},
                        crate::grpc::comp_handler::Verification::Block => html!{<span class={"marked-err"}>{"Оценка отклонена"}</span>},
                        crate::grpc::comp_handler::Verification::NotChecked => html!{<span class={"marked-info"}>{"Оценка ожидает подтверждения"}</span>},
                    },
                    None => html!{<span class={"marked-err"}>{"Ошибка передачи данных"}</span>},
                }
            }
            
        },
        
        /*match x {
            1 => html!{<span class={"marked-ok"}>{"Оценка принята"}</span>},
            0 => html!{<span class={"marked-err"}>{"Оценка отклонена"}</span>},
            _ => html!{<span class={"marked-info"}>{"Оценка ожидает подтверждения"}</span>}
        },*/
        _ => {
            html!{<></>}
        },
    };

    html! {
        <>
            <h3>{"Окно оценивания"}</h3>
            <div class={"marked-info"}><h4>{format!("Номинация: {}", team.nom)}</h4></div>
            <Spacer space={"1em"}/>
            <div class={"stretch"}>
                <div class={"marked-ok-pill"}>{format!("Номер команды: {}", team.tid)}</div>
                <HSpacer space={"1em"} />
                <Spacer space={"1em"} />
                <div class={"marked-ok-pill"}>
                    {particips}
                </div>
            </div>
            <Spacer space={"1em"}/>
            <div class={classes!("accent-block")}>
                <div class={classes!("stretch")}>
                    <span class={""}>{"Ваша категория оценивания: "}</span>
                    <span class={"marked-ok"}>{scheme.get_judgement_group_name(judge.mark_group).unwrap_or("Ошибка".to_string())}</span>
                </div>
                <div class={classes!("stretch")}>
                    <span class={""}>{"Ваш номер очереди: "}</span>
                    <span class={"marked-info"}>{judge.queue+1}</span>
                </div>
                {
                    verdict_message
                }
                <form class={classes!("stretch")}>
                    //<form>
                        <input ref={mark_ref} type="number" value={"0"} min={"0"} max={"10"} step={"1"}/>
                        <HSpacer space={"1em"} />
                        <Spacer space={"1em"} />
                        <Button text={"Отправить"} onclick={send_vote}/>
                    //</form>
                </form>
            </div>
        </>
    }
}


#[autoprops]
#[function_component(JudgeView)]
pub fn judge_view(coid: &i32, judge: &Judge, schem: &JudgeScheme) -> Html {
    let gctx = use_context::<GlobalContext>().expect("no ctx found");
    let ctx = use_context::<EventQueueMessageContext>().expect("no eqm ctx found");

    let active_view = ctx.0.iter().rev().find(|x| {
        match &x.message {
            Some(
                EqMessage { comp_id, author, signature, message: Some(msg) }
            ) => {
                match &msg {
                    eq_message::Message::SetActiveAction(active_action_state) => (active_action_state.qid==judge.queue),
                    eq_message::Message::ClearQueueAction(id) => (id.id == judge.queue),
                    _ => false,
                }
            },
            _ => false,
        }
    });

    let last_my_message = ctx.0.iter().rev().find(|x| {
        match &x.message {
            Some(
                EqMessage { comp_id, author, signature, message: Some(msg) }
            ) => {
                match &msg {
                    eq_message::Message::VoteMessage(m) => {
                        //x.message_id > active_view.unwrap().message_id &&
                        author.clone().unwrap().uid.eq(&gctx.user.get_user_data().as_ref().unwrap().uid)
                    },
                    _ => false,
                }
            },
            _ => false,
        }
    });
    web_sys::console::log_1(&format!("Last msg>> {:?}", last_my_message).into());

    let vw = match active_view {
        Some(view) => {
            match &view.message {
                Some(EqMessage{ comp_id, author, signature, message: Some(msg) } ) => {
                    match msg {
                        eq_message::Message::SetActiveAction(active_action_state) => {
                            html! {<>
                                /*<div class={"stretch"}>
                                    <div style={"max-width:45%"}>
                                        <QueueActiveViewer
                                            coid={coid}
                                            quid={0}
                                            team={active_action_state.team.as_ref().unwrap().clone()}
                                            schem={schem.clone()}
                                            participants={Some(active_action_state.participants.clone())}
                                            active_view={Some(active_action_state.clone())}
                                        />
                                    </div>
                                    <HSpacer space={"0.5em"}/>
                                    <div style={"max-width:45%"}>
                                        <QueueActiveViewer
                                        coid={coid}
                                        quid={1}
                                        team={active_action_state.team.as_ref().unwrap().clone()}
                                        schem={schem.clone()}
                                        participants={Some(active_action_state.participants.clone())}
                                        active_view={Some(active_action_state.clone())}
                                    />
                                    </div>
                                </div>*/

                                <VoteView
                                    coid={coid}
                                    action={active_action_state.aid}
                                    judge={judge.clone()}
                                    scheme={schem.clone()}
                                    team={active_action_state.team.as_ref().unwrap().clone()}
                                    approvement_id={
                                        match last_my_message {
                                            Some(x) => Some(x.message_id.clone()),
                                            None => None
                                        }
                                    }
                                    //participants={Some(active_action_state.participants.clone())}
                                    active_view={Some(active_action_state.clone())}
                                />
                            </>}
                        },
                        eq_message::Message::ClearQueueAction(id) => {
                            html! {
                                <>
                                    <span class={"marked"}>{"Нет активного выступления на вашей очереди"}</span>
                                    <LoadingView/>
                                </>
                            }
                        },
                        _ => {
                            html! {
                                <>
                                    <span class={"marked-err-pill"}>{"Произошла ошибка. Сообщите об этом администратору"}</span>
                                    <LoadingView/>
                                </>
                            }
                        },
                    }
                }
                _ => {
                    html! {
                        <>
                            <span class={"marked-err-pill"}>{"Произошла ошибка. Сообщите об этом администратору"}</span>
                            <LoadingView/>
                        </>          
                    }
                },
            }
        },
        None => {
            html! {
                <>
                    <span class={"marked-err-pill"}>{"Произошла ошибка. Сообщите об этом администратору"}</span>
                    <LoadingView/>
                </>          
            }
        },
    };

    html! {
        <>
            {"Судейство"}
            { vw }
        </>
    }
}



#[autoprops]
#[function_component(SecretaryView)]
pub fn secretary_view(
    coid: &i32,
    schem: &JudgeScheme,
    nqueues: &i32,
) -> Html {
    let rez = schem.sizes();
    let gctx = use_context::<GlobalContext>().expect("no ctx found");
    let ctx = use_context::<EventQueueMessageContext>().expect("no eqm ctx found");

    let active_views = (0..*nqueues).map(
        |i| {
            ctx.0.iter().rev().find(|x| {
                match &x.message {
                    Some(
                        EqMessage { comp_id, author, signature, message: Some(msg) }
                    ) => {
                        match &msg {
                            eq_message::Message::SetActiveAction(active_action_state) => (active_action_state.qid==i),
                            eq_message::Message::ClearQueueAction(id) => (id.id == i),
                            _ => false,
                        }
                    },
                    _ => false,
                }
            })
        }
    ).collect::<Vec<Option<&EqHistoryMessage>>>();
    web_sys::console::log_1(&format!("NQ>> {:?}", nqueues).into());
    web_sys::console::log_1(&format!("Views>> {:?}", active_views).into());

    let vw : Vec<_> = active_views.iter().enumerate().map(
        |(i, active_view)| {
            match active_view {
                Some(view) => {
                    match &view.message {
                        Some(EqMessage{ comp_id, author, signature, message: Some(msg) } ) => {
                            match msg {
                                eq_message::Message::SetActiveAction(active_action_state) => {

                                    let fix = Callback::from(
                                        {
                                            let gctx = gctx.clone();
                                            let user = gctx.user.get_user_data().as_ref().unwrap().clone();
                                            let coid = coid.clone();
                                            move |_| {
                                                let gctx = gctx.clone();
                                                let user = user.clone();
                                                let coid = coid.clone();
                                                wasm_bindgen_futures::spawn_local(
                                                    async move {
                                                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&gctx);
                                                        let res = client.push_eq_message(
                                                            EqMessage{
                                                                comp_id: coid.clone(),
                                                                author: Some(
                                                                    UserView{
                                                                        uid: user.uid,
                                                                        login: user.username,
                                                                        selfname: user.selfname
                                                                    }
                                                                ),
                                                                signature: "SIGNATURE".to_string(),
                                                                message: Some(
                                                                    Message::FixVoting(
                                                                        FixVotingMessage { queue_id: i as i32, verdict: Verification::Approve.into() }
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


                                    let trailer = html! {
                                        <>
                                        <HSpacer space="1em"/>
                                        <Button
                                            text={"Фиксировать"}
                                            onclick={fix}
                                        />
                                        </>
                                    };

                                    html! {<>
                                            //<div>
                                                <QueueActiveViewer
                                                    coid={coid}
                                                    quid={i as i32}
                                                    team={active_action_state.team.as_ref().unwrap().clone()}
                                                    schem={schem.clone()}
                                                    //participants={Some(active_action_state.participants.clone())}
                                                    active_view={Some(active_action_state.clone())}
                                                    trailer={
                                                        trailer
                                                    }
                                                />
                                            //</div>
                                            <HSpacer space={"0.5em"}/>
                                            <Spacer space={"0.5em"}/>
                                    </>}
                                },
                                eq_message::Message::ClearQueueAction(id) => {

                                    let next = Callback::from(
                                        {
                                            let gctx = gctx.clone();
                                            let user = gctx.user.get_user_data().as_ref().unwrap().clone();
                                            let coid = coid.clone();
                                            move |_| {
                                                let gctx = gctx.clone();
                                                let user = user.clone();
                                                let coid = coid.clone();
                                                wasm_bindgen_futures::spawn_local(
                                                    async move {
                                                        let mut client = crate::context::Context::get_comp_handler_grpc_client(&gctx);
                                                        let res = client.push_eq_message(
                                                            EqMessage{
                                                                comp_id: coid.clone(),
                                                                author: Some(
                                                                    UserView{
                                                                        uid: user.uid,
                                                                        login: user.username,
                                                                        selfname: user.selfname
                                                                    }
                                                                ),
                                                                signature: "SIGNATURE".to_string(),
                                                                message: Some(
                                                                    Message::TryNext(
                                                                        TryNext { queue_id: i as i32 }
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

                                    html! {
                                        <>
                                            //<div>
                                                <div class={classes!("accent-white-block")}>
                                                    <div class={"stack"}>
                                                        <span class={"marked"}>{"Нет активного выступления на этой очереди"}</span>
                                                        <Button
                                                            text={"Следующий участник"}
                                                            onclick={next}
                                                        />
                                                    </div>
                                                </div>
                                                <HSpacer space={"0.5em"}/>
                                                <Spacer space={"0.5em"}/>
                                            //</div>
                                            //<LoadingView/>
                                        </>
                                    }
                                },
                                _ => {
                                    html! {
                                        <>
                                            <span class={"marked-err-pill"}>{"Произошла ошибка 3. Сообщите об этом администратору или подождите"}</span>
                                            <LoadingView/>
                                        </>
                                    }
                                },
                            }
                        }
                        _ => {
                            html! {
                                <>
                                    <span class={"marked-err-pill"}>{"Произошла ошибка 2. Сообщите об этом администратору или подождите"}</span>
                                    <LoadingView/>
                                </>          
                            }
                        },
                    }
                },
                None => {

                    let run_comp = Callback::from(
                        {
                            let gctx = gctx.clone();
                            let coid = coid.clone();
                            move  |_| {
                                let gctx = gctx.clone();
                                let coid = coid.clone();
                                wasm_bindgen_futures::spawn_local(
                                    async move {
                                        let mut client = Context::get_comp_handler_grpc_client(&gctx);
                                        let resp = client.run(
                                            Id { id: coid }
                                        ).await;
                                        match resp {
                                            Ok(_) => {
                                                web_sys::window()
                                                    .unwrap()
                                                    .location()
                                                    .reload()
                                                    .unwrap();
                                            },
                                            Err(_) => {

                                            },
                                        };
                                    }
                                );
                            }
                        }
                    );

                    html! {
                        <>
                            <span class={"marked-err-pill"}>{"Соревнование не запущено"}</span>
                            <Spacer space="0.5em"/>
                            <Button text={"Запустить"} onclick={run_comp}/>
                            <Spacer space="0.5em"/>
                            <LoadingView/>
                        </>          
                    }
                },
            }
        }
    ).collect();

    html! {
        <>
            {"Секретариат"}
            <div class={"stretch"}>
                { vw }
            </div>
        </>
    }
}

#[autoprops]
#[function_component(VerificationView)]
pub fn verify_view(
    coid: &i32,
    schem: &JudgeScheme,
    //nqueues: &i32,
    queue_id: &i32
    //#[prop_or(vec![])] active_views: &Vec<Option<ActiveActionState>>
) -> Html {

    let rez = schem.sizes();

    let gctx = use_context::<GlobalContext>().expect("no ctx found");
    let ctx = use_context::<EventQueueMessageContext>().expect("no eqm ctx found");

    /*let active_views = (0..*nqueues).map(
        |i| {
            
        }
    ).collect::<Vec<Option<&EqHistoryMessage>>>();*/

    let active_view = ctx.0.iter().rev().find(|x| {
        match &x.message {
            Some(
                EqMessage { comp_id, author, signature, message: Some(msg) }
            ) => {
                match &msg {
                    eq_message::Message::SetActiveAction(active_action_state) => (active_action_state.qid.eq(queue_id)),
                    eq_message::Message::ClearQueueAction(id) => (id.id.eq(queue_id)),
                    _ => false,
                }
            },
            _ => false,
        }
    });

    web_sys::console::log_1(&format!("NQ>> {:?}", queue_id).into());
    web_sys::console::log_1(&format!("View>> {:?}", active_view).into());

    //let vw : Vec<_> = active_views.iter().enumerate().map(
        //|(i, active_view)| {
            let vw = match active_view {
                Some(view) => {
                    match &view.message {
                        Some(EqMessage{ comp_id, author, signature, message: Some(msg) } ) => {
                            match msg {
                                eq_message::Message::SetActiveAction(active_action_state) => {
                                    html! {<>
                                            <VerificationActiveViewer
                                                    coid={coid}
                                                    quid={queue_id.clone()}
                                                    //team={active_action_state.team.as_ref().unwrap().clone()}
                                                    schem={schem.clone()}
                                                    //participants={Some(active_action_state.participants.clone())}
                                                    active_view={Some(active_action_state.clone())}
                                            />
                                            <HSpacer space={"0.5em"}/>
                                            <Spacer space={"0.5em"}/>
                                    </>}
                                },
                                eq_message::Message::ClearQueueAction(id) => {
                                    html! {
                                        <>
                                            <span class={"marked"}>{"Нет активного выступления на этой очереди"}</span>
                                            <LoadingView/>
                                        </>
                                    }
                                },
                                _ => {
                                    html! {
                                        <>
                                            <span class={"marked-err-pill"}>{"Произошла ошибка. Сообщите об этом администратору"}</span>
                                            <LoadingView/>
                                        </>
                                    }
                                },
                            }
                        }
                        _ => {
                            html! {
                                <>
                                    <span class={"marked-err-pill"}>{"Произошла ошибка. Сообщите об этом администратору"}</span>
                                    <LoadingView/>
                                </>          
                            }
                        },
                    }
                },
                None => {
                    html! {
                        <>
                            <span class={"marked-err-pill"}>{"Произошла ошибка. Сообщите об этом администратору"}</span>
                            <LoadingView/>
                        </>          
                    }
                },
            };
        //}
    //).collect();

    html! {
        <>
            {"Арбитор"}
            <div class={"stretch"}>
                { vw }
            </div>
        </>
    }
}


#[autoprops]
#[function_component(QueueActiveViewer)]
pub fn queue_active_viewer(
    coid: &i32,
    quid: &i32,
    schem: &JudgeScheme,
    #[prop_or(None)] team: &Option<Team>,
    //#[prop_or(None)] participants: &Option<Vec<EasyParticipant>>,
    #[prop_or(None)] active_view:  &Option<ActiveActionState>,
    #[prop_or(None)] trailer: &Option<Html>
) -> Html {
    let rez = schem.sizes();
    let results_table = rez.iter().enumerate().map(
        |(i, n)| {
            let active_view_view = match &active_view {
                Some(v) => {
                    match v.marks.get(&schem.get_judgement_group_name(i.try_into().unwrap()).unwrap()) {
                        Some(votes) => {
                            let mut ready = votes.votes.iter().map(
                                |x| {
                                    html!{ <> <span class={
                                        match x.verifyed() {
                                            crate::grpc::comp_handler::Verification::Approve => {classes!("little-card")},
                                            crate::grpc::comp_handler::Verification::Block => {classes!("little-card-err")},
                                            crate::grpc::comp_handler::Verification::NotChecked => {classes!("little-card-wait")}
                                        }
                                    }>{x.mark}</span> <HSpacer space="10px"/> <Spacer space="10px"/> </> }
                                }
                            ).collect::<Vec<VNode>>();
                            while ready.len() < (*n).try_into().unwrap() {
                                ready.push(
                                    html!{ <> <span class={"little-card"}>{"-"}</span> <HSpacer space="10px"/> <Spacer space="10px"/> </> }
                                );
                            }
                            ready
                        },
                        None => (0..(n.clone().try_into().unwrap())).map(|x| {
                            html!{ <> <span class={"little-card"}>{"-"}</span> <HSpacer space="10px"/> <Spacer space="10px"/> </> }
                        } ).collect::<Vec<Html>>(),
                    }
                },
                None => vec![]
            };
            html! {
                <> 
                    <Spacer space="0.5em"/>
                    <span class={"marked"}>{schem.get_judgement_group_name(i.try_into().unwrap()).unwrap() + ": "}</span>
                    <Spacer space="0.5em"/>
                    <div class={"stretch"}>
                        <HSpacer space="0.5em"/>
                        {
                            active_view_view
                        }
                    </div>
                </>
            }
        }
    ).collect::<Vec<Html>>();

    html! {
        <>
            <div class={classes!("accent-white-block","min-45")}>
                <div class={classes!("stretch")}>
                    <span class={""}>{"Очередь: "}</span>
                    <HSpacer space="1em"/>
                    <span class={"marked-info"}>{quid+1}</span>
                </div>
                {match (team, active_view) {    
                    (Some(t), Some(av)) => {
                        let particips = match team {
                            Some(t) => {
                                html! {
                                    <>
                                    <span>{"Участники: "}</span>
                                    {
                                        t.participants.iter().map(|x| html!{<span class={"marked-ok-pill"}> {x.name.clone()} </span>}).collect::<Vec<VNode>>()
                                    }
                                    </>
                                }
                            },
                            None => {
                                //format!("Номера участников: {}", .join(", ") )
                                html! {
                                    <>
                                        <span>{"Участики не получы"}</span>
                                    //<span>{"Номера участников: "}</span>
                                    //{
                                    //    t.participants.iter().map(|x| html!{<span class={"marked-ok-pill"}> {x.to_string().clone()} </span>}).collect::<Vec<VNode>>()
                                    //}
                                    </>
                                }
                            },
                        };
                        html! {
                            <>
                                <div class={"marked-info"}><h4>{format!("Номинация: {}", t.nom)}</h4></div>
                                <Spacer space={"1em"}/>
                                <div class={"stack"}>
                                    <div class={"marked-ok-pill"}>{format!("Номер команды: {}", t.tid)}</div>
                                    <div>
                                        <HSpacer space={"1em"} />
                                        <Spacer space={"1em"} />
                                    </div>
                                    //<div class={"marked-ok-pill"}>
                                        {particips}
                                    //</div>
                                </div>
                                {
                                    results_table
                                }
                            </>
                        }
                    },
                    _ => {
                        html!{"Очередь сейчас не активна"}
                    }
                }}
                <div>
                {
                    match trailer {
                        Some(x) => x.clone(),
                        None => html!{<> </>}
                    }
                }
                </div>
            </div>
        </>
    }
}


/// Same as QueueActiveViewer but with verify_all button
#[autoprops]
#[function_component(VerificationActiveViewer)]
pub fn queue_view(
    coid: &i32,
    quid: &i32,
    schem: &JudgeScheme,
    // #[prop_or(None)] team: &Option<Team>,
    // #[prop_or(None)] participants: &Option<Vec<EasyParticipant>>,
    #[prop_or(None)] active_view:  &Option<ActiveActionState>
) -> Html {

    let ctx = use_context::<GlobalContext>().expect("no ctx found");

    let mut to_verify : Vec<i32> = vec![];
    let rez = schem.sizes();
    rez.iter().enumerate().for_each(
        |(i, n)| {
            let active_view_view = match &active_view {
                Some(v) => {
                    match v.marks.get(&schem.get_judgement_group_name(i.try_into().unwrap()).unwrap()) {
                        Some(votes) => {
                            let mut ready = votes.votes.iter().for_each(
                                |x| {
                                    match x.verifyed() {
                                        crate::grpc::comp_handler::Verification::NotChecked => {
                                            if x.message_id != -1 {
                                                to_verify.push(x.message_id);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            );
                        },
                        None => {}
                    }
                },
                None => {}
            };
        }
    ); 
    let send_all = Callback::from({
        let gctx = ctx.clone();
        let to_verify = to_verify.clone();
        let coid = coid.clone();
        let quid = quid.clone();
        move |_| {
            let gctx = ctx.clone();
            let to_verify = to_verify.clone();
            let coid = coid.clone();
            let quid = quid.clone();
            wasm_bindgen_futures::spawn_local(
                async move {
                    let mut client = crate::context::Context::get_comp_handler_grpc_client(&gctx);
                    for target in to_verify {
                        let res = client.push_eq_message(
                            {
                                let udata = gctx.user.get_user_data().clone().unwrap();
                                EqMessage{
                                    comp_id: coid,
                                    author: Some(UserView{
                                        uid: udata.uid,
                                        login: udata.get_name().to_string(),
                                        selfname: udata.selfname.clone()
                                    }),
                                    signature: "SIGNATURE".to_string(),
                                    message: Some(
                                        eq_message::Message::VerifyMessage(
                                            VerifyVoteMessage {
                                                target_message_id: target,
                                                queue_id: quid,
                                                verdict: Verification::Approve.into()
                                            }
                                        )
                                    ) 
                                }
                            }
                        ).await;web_sys::console::log_1(&format!(">> {:?}", res).into());
                    }
                }
            );
        }
    });
    html! {
        <>
            <QueueActiveViewer
                coid={coid}
                quid={quid}
                schem={schem.clone()}
                // team={team.clone()}
                // participants={participants.clone()}
                active_view={active_view.clone()}
                trailer={
                    Some(
                        html! {
                            <>
                            <Spacer space="0.5em"/>
                            <div>
                                <Button text={"Подтвердить всех"} onclick={send_all}/>
                            </div>
                            </>
                        }
                    )
                }
            />
        </>
    }
}

#[autoprops]
#[function_component(MessageView)]
pub fn message_view(
    input_ref: NodeRef,
    message: &EqHistoryMessage,
    #[prop_or(None)] votable_callbacks:
        Option<(Callback<MouseEvent>, Callback<MouseEvent>)>
) -> Html {
    let mut view: Html = html!{<>{"Empty message"}</>};
    if let Some(eqm) = message.message.clone() {
        let msg = eqm.message.unwrap();
        view = match msg {
            eq_message::Message::FinesSetup(fines) => {
                html! {
                    <>
                        <div class={classes!("stack")}>
                            <div class={classes!("stretch")}>   
                                <div class={classes!("stack")}> 
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("marked", "bold")}>{format!("очередь: {};", fines.queue_id + 1)}</span>
                                        <span class={classes!("marked", "bold")}>{format!("выступление: {};", fines.action_id   )}</span>
                                    </div>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("bold")}>{format!("сбавки: {:?};", fines.fines)}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </>
                }
            },
            eq_message::Message::VoteMessage(vote) => {
                let vote_view = {
                    if let Some(callbacks) = votable_callbacks {
                        html! {
                            <div class={classes!("stretch", "vote-block")}>
                                <OkButton onclick={callbacks.0}/>
                                <DenyButton onclick={callbacks.1}/>
                            </div>
                        }
                    } else {
                        html!{<> {""} </>}
                    }
                };
                html! {
                    <>
                        <div class={classes!("stack")}>
                            <div class={classes!("stretch")}>   
                                <div class={classes!("stack")}>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("marked", "bold")}>{format!("очередь: {};", vote.queue_id + 1)}</span>
                                        <span class={classes!("marked", "bold")}>{format!("выступление: {};", vote.action_id   )}</span>
                                    </div>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("bold")}>{format!("характеристика: {};", vote.mark_type)}</span>
                                        <span class={classes!("bold")}>{format!("оценка: {}", vote.mark)}</span>
                                    </div>
                                </div>
                                <Spacer space={"0.25em"}/>
                                {vote_view}
                            </div>
                        </div>
                    </>
                }
            },
            eq_message::Message::VerifyMessage(verify) => {
                html!{
                    <div class={classes!("stack")}>
                            <div class={classes!("stretch")}>   
                                <div class={classes!("stack")}>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("marked", "bold")}>{format!("Подтверждение для: #{};", verify.target_message_id)}</span>
                                        <span class={
                                                match verify.verdict() {
                                                    crate::grpc::comp_handler::Verification::Block => classes!("marked-err", "bold"),
                                                    crate::grpc::comp_handler::Verification::Approve => classes!("marked-ok", "bold"),
                                                    crate::grpc::comp_handler::Verification::NotChecked => classes!("bold"),
                                                }
                                            }>
                                            {
                                                format!(
                                                    "Вердикт: {};",
                                                    match verify.verdict() {
                                                        crate::grpc::comp_handler::Verification::Block => "Отказано",
                                                        crate::grpc::comp_handler::Verification::Approve => "Принято",
                                                        crate::grpc::comp_handler::Verification::NotChecked => "Ожидает проверки",
                                                    }
                                                )
                                            }
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                }
            },
            eq_message::Message::FixVoting(fix) => {
                html! {
                    <>
                        <div class={classes!("stack")}>
                            <div class={classes!("stretch")}>   
                                <div class={classes!("stack")}>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("marked", "bold")}>{format!("очередь: {};", fix.queue_id + 1)}</span>
                                        //<span class={classes!("marked", "bold")}>{format!("выступление: {};", fix.action_id   )}</span>
                                    </div>
                                    <div class={classes!("stretch")}>
                                        <span class={
                                                match fix.verdict() {
                                                    crate::grpc::comp_handler::Verification::Block => classes!("marked-err", "bold"),
                                                    crate::grpc::comp_handler::Verification::Approve => classes!("marked-ok", "bold"),
                                                    crate::grpc::comp_handler::Verification::NotChecked => classes!("bold"),
                                                }
                                            }>
                                            {
                                                format!(
                                                    "Вердикт: {};",
                                                    match fix.verdict() {
                                                        crate::grpc::comp_handler::Verification::Block => "Отказано",
                                                        crate::grpc::comp_handler::Verification::Approve => "Принято",
                                                        crate::grpc::comp_handler::Verification::NotChecked => "Ожидает проверки",
                                                    }
                                                )
                                            }
                                        </span>
                                    </div>
                                </div>
                                <Spacer space={"0.25em"}/>
                            </div>
                        </div>
                    </>
                }
            },
            eq_message::Message::TryNext(next) => {
                html! {
                    <>
                        <div class={classes!("stack")}>
                            <div class={classes!("stretch")}>   
                                <div class={classes!("stack")}>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("marked", "bold")}>{format!("очередь: {};", next.queue_id + 1)}</span>
                                    </div>
                                </div>
                                <Spacer space={"0.25em"}/>
                            </div>
                        </div>
                    </>
                }
            },
            eq_message::Message::Block(block) => {
                html! {
                    <>
                        <div class={classes!("stack")}>
                            <div class={classes!("stretch")}>   
                                <div class={classes!("stack")}>
                                    <div class={classes!("stretch")}>
                                        <span class={classes!("marked-err", "bold")}>{"События приостановлены администратором"}</span>
                                    </div>
                                </div>
                                <Spacer space={"0.25em"}/>
                            </div>
                        </div>
                    </>
                }
            },
            eq_message::Message::SetActiveAction(_) => html!{<>{"ViewUpdate"}</>},
            eq_message::Message::ClearQueueAction(_) => html!{<>{"Clear"}</>}, // need only when debug
        }   
    };
    html! {
        <>
            <div ref={input_ref} id={message.generate_view_id()} class={classes!("accent-block")}>
                <div class={classes!("stretch")}>
                    <span class={classes!("bold")}>{format!("#{}", message.message_id)}</span>
                    <span class={"marked"}>{format!("{}", message.clone().message.unwrap().message.unwrap().represent())}</span>
                    <span>{format!(
                                    " от: {}",
                                        message
                                        .clone()
                                        .message.unwrap()
                                        .author.unwrap()
                                        .login
                                )}</span>
                </div>
                {view}
            </div>   
        </>
    }
}

#[autoprops]
#[function_component(MessageQueueView)]
pub fn message_queue(coid: i32, #[prop_or(None)] view_type: &Option<users::role::Role>) -> Html {
    let gctx = use_context::<GlobalContext>().expect("no ctx found");
    let ctx = use_context::<EventQueueMessageContext>().expect("no eqm ctx found");
    let mut message_refs = use_mut_ref(|| {HashMap::<i32, NodeRef>::new()});
    //let mut view_type = use_mut_ref(|| { view_type.clone() });
    use_effect({
        let ctx = ctx.clone();
        let message_refs = message_refs.clone();
        let view_type = view_type.clone();
        move || {
            ctx.0.iter().for_each(
                |x| {
                    match &x.message {
                        Some(xm) => {
                            match &xm.message {
                                Some(xmm) => match xmm {
                                    eq_message::Message::VerifyMessage(verify) => {
                                        log_1(&format!("Verify found: {:?}", verify).into());
                                        log_1(&format!("View type: {:?}", view_type).into());
                                        match &message_refs.borrow_mut().get(&verify.target_message_id) {
                                            Some(node) => {
                                                log_1(&"Node found".into());
                                                let node = node.cast::<HtmlElement>().unwrap();
                                                match view_type {
                                                    Some(vt) => {
                                                        match vt {
                                                            users::role::Role::Moderator(_) => {
                                                                let mut list = node.get_elements_by_class_name("vote-block");
                                                                for i in 0..list.length() {
                                                                   match list.item(i)  {
                                                                       Some(el) => {
                                                                            el.set_inner_html("");
                                                                       },
                                                                       None => {
                                                                       },
                                                                   }
                                                                }
                                                            },
                                                            users::role::Role::Arbitor(generic::Id{id: quid}) => {
                                                                let mut list = node.get_elements_by_class_name("vote-block");
                                                                for i in 0..list.length() {
                                                                   match list.item(i)  {
                                                                       Some(el) => {
                                                                            el.set_inner_html("");
                                                                       },
                                                                       None => {
                                                                       },
                                                                   }
                                                                }
                                                            },
                                                            _ => {}
                                                        }
                                                    },
                                                    None => {
                                                    },
                                                };
                                                match verify.verdict() {
                                                    crate::grpc::comp_handler::Verification::Block => {
                                                        node.set_class_name(&"err-block")
                                                    },
                                                    crate::grpc::comp_handler::Verification::Approve => {
                                                        node.set_class_name(&"ok-block")
                                                    },
                                                    crate::grpc::comp_handler::Verification::NotChecked => {},
                                                }
                                            },
                                            None => {
                                                log_1(&"Node NOT found".into());
                                            },
                                        }
                                    },
                                    _ => {
                                    }
                                },
                                None => {}
                            }
                        },
                        None => {}
                    }
                }
            );
        }
    });
    let msglist: Vec<Html> = ctx.0.iter().map(|x| {
        message_refs.borrow_mut().insert(x.message_id, NodeRef::default());
        let accept_deny_callback_factory = |target: i32, verdict: Verification, quid: i32| {
                Callback::from({
                    let gctx = gctx.clone();
                    move |e| {
                        let gctx = gctx.clone();
                        wasm_bindgen_futures::spawn_local(
                            async move {
                                let mut client = crate::context::Context::get_comp_handler_grpc_client(&gctx);
                                let res = client.push_eq_message(
                                    {
                                        let udata = gctx.user.get_user_data().clone().unwrap();
                                        EqMessage{
                                            comp_id: coid,
                                            author: Some( UserView{
                                                uid: udata.uid,
                                                login: udata.get_name().to_string(),
                                                selfname: udata.selfname.clone()
                                            }),
                                            signature: "SIGNATURE".to_string(),
                                            message: Some(
                                                eq_message::Message::VerifyMessage(
                                                    VerifyVoteMessage {
                                                        target_message_id: target,
                                                        queue_id: quid,
                                                        verdict: verdict as i32
                                                    }
                                                )
                                            ) 
                                        }
                                    }
                                ).await;
                                web_sys::console::log_1(&format!(">> {:?}", res).into());
                            }
                        );
                    }
                })
            //}
        };
        html! {
            <>
                <MessageView
                    input_ref={message_refs.borrow_mut().get(&x.message_id).unwrap()}
                    message={x.clone()}
                    votable_callbacks={
                        match view_type {
                            Some(r) => {
                                match &x.message {
                                    Some(msg) => {
                                        match &msg.message {
                                            Some(msg) => {
                                                match msg {
                                                    Message::VoteMessage(vote_message) => {
                                                        match r {
                                                            users::role::Role::Moderator(_) => {
                                                                Some(
                                                                    (
                                                                        accept_deny_callback_factory.clone()(
                                                                            x.message_id,
                                                                            Verification::Approve,
                                                                            vote_message.queue_id
                                                                        ),// Accept
                                                                        accept_deny_callback_factory.clone()(
                                                                            x.message_id,
                                                                            Verification::Block,
                                                                            vote_message.queue_id
                                                                        ) // Deny
                                                                    )
                                                                )
                                                            },
                                                            users::role::Role::Arbitor(generic::Id{id: quid}) => {
                                                                if vote_message.queue_id.eq(quid) {
                                                                    Some(
                                                                        (
                                                                            accept_deny_callback_factory.clone()(
                                                                                x.message_id, 
                                                                                Verification::Approve,
                                                                                *quid
                                                                            ),// Accept
                                                                            accept_deny_callback_factory.clone()(
                                                                                x.message_id, 
                                                                                Verification::Block,
                                                                                *quid
                                                                            )// Deny
                                                                        )
                                                                    )
                                                                } else {
                                                                    None
                                                                }
                                                            },
                                                            users::role::Role::Secretary(_) => None,
                                                            users::role::Role::Judge(_) => None,
                                                            users::role::Role::Watcher(_) => None,
                                                        }
                                                    },
                                                    _ => None,
                                                }
                                            },
                                            None => None,
                                        }
                                    },
                                    None => None,
                                }
                            },
                            None => None,
                        }
                    }
                />
                <Spacer space={"0.2em"}/>
            </>
        }
     }).collect();
    html! {
        <>
            <h4>{"Поток сообщений"}</h4>
            <div class={classes!("cards-stack")}>
                { msglist.iter().rev().cloned().collect::<Vec<Html>>() }
            </div>
        </>
    }
}

#[autoprops]
#[function_component(CompHandlerApp)]
pub fn comp_handler_app(cid: i32) -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");
    let controller = AbortController::new().unwrap();
    let message_ctx: UseReducerHandle<_> = use_reducer_eq(|| EQMContext::default() );
    let user_state: UseStateHandle<Option<Vec<users::role::Role>>> = use_state(|| None );
    let comp_view_state: UseStateHandle<Option<CompView>> = use_state(|| None );
    use_effect({
        let ctx = ctx.clone();
        let message_ctx = message_ctx.clone();
        let user_state = user_state.clone();
        let comp_view_state = comp_view_state.clone();
        move || {
            let ctx = ctx.clone();
            let message_ctx = message_ctx.clone();
            let comp_view_state = comp_view_state.clone();
            wasm_bindgen_futures::spawn_local(
                {   
                    let ctx = ctx.clone();
                    let message_ctx = message_ctx.clone();
                    let controller = controller.clone();
                    let comp_view_state = comp_view_state.clone();
                    async move {
                        if ctx.user.ready() && !controller.signal().aborted() {
                            let mut client = crate::context::Context::get_comp_handler_grpc_client(&ctx);
                            if user_state.is_none() && !controller.signal().aborted() {
                                let mut u_client = crate::context::Context::get_user_mng_grpc_client(&ctx);
                                match u_client.get_my_comp_role(generic::Id{id: cid}).await {
                                    Ok(r) => {
                                        user_state.set(Some(r.into_inner().roles.iter().filter_map(|r| {r.role}).collect()));
                                        controller.abort();
                                    },
                                    Err(_) => controller.abort(),
                                }
                            }
                            if comp_view_state.is_none() && !controller.signal().aborted() {
                                let mut u_client = crate::context::Context::get_comp_grpc_client(&ctx);
                                match u_client.get_comps_views( generic::IdsList{
                                    obj_ids: vec![cid]
                                }).await {
                                    Ok(r) => {
                                        comp_view_state.set(r.into_inner().comp_views.get(&cid).cloned());
                                        controller.abort();
                                    },
                                    Err(_) => controller.abort(),
                                }
                            };
                            {
                                'globc: loop {
                                    if controller.signal().aborted() {
                                        break 'globc;
                                    };

                                    if message_ctx.0.len() == 0 {
                                        match client.pull_eq_message_history(
                                            EqHistoryRequest{ comp_id: cid, deep: i32::MAX }
                                        ).await {
                                            Ok(history) => {
                                                let v = EqmContextAction::Connect(history.into_inner().history);
                                                web_sys::console::log_1(&format!("History >> {:?}", v).into());
                                                message_ctx.dispatch(v);
                                                break 'globc;
                                            },
                                            Err(_) => {
                                                web_sys::console::log_1(&"No History!".into());
                                                break 'globc
                                            },
                                        };
                                        
                                    };
                                    let mut stream_response = 
                                        client.start_eq_message_stream(Id{id: cid});  
                                    match stream_response.await {
                                        Ok(stream_response) => {
                                            let mut stream_response = stream_response.into_inner();
                                            'streamloop: loop {
                                                if controller.signal().aborted() {
                                                    break 'globc;
                                                };
                                                let responseopt = stream_response.message().await;
                                                if let Ok(response) = responseopt {
                                                    match response {
                                                        Some(r) => {
                                                            message_ctx.dispatch(EqmContextAction::Add(r));
                                                        },
                                                        None => {
                                                            break 'globc;
                                                        },
                                                    }
                                                }
                                            }
                                        },
                                        Err(_) => {
                                            //break;
                                            sleep(Duration::from_secs(5)).await;
                                        },
                                    };
                            } }
                        };
                    }
                }
            );

            move || {
                controller.abort();
            }
        }
    }); // use-effect 
    let last = match (*user_state).clone() {
        Some(v) => {
            let mut s = v;
            s.sort_by_key(|x| {x.weight()});
            s.iter().last().cloned()
        },
        None => None
    };
    //log_1(&format!("Authed as role: {:?}", last).into());
    if (*comp_view_state).is_none() { return html!{<LoadingView/>} }
    let stateview = {
        match (*user_state).clone() {
            Some(v) => {
                let mut s = v.clone();
                s.sort_by_key(|x| {x.weight()});
    
                match s.iter().last() {
                    Some(x) => {
                        match x {
                            //users::role::Role::Moderator(_) => todo!(),
                            users::role::Role::Moderator(_) | users::role::Role::Secretary(_) => html!{
                                <>
                                    <SecretaryView
                                        coid={cid}
                                        nqueues={(*comp_view_state).clone().unwrap().declaration.unwrap().queues.len() as i32}
                                        schem={(*comp_view_state).clone().unwrap().declaration.unwrap().scheme()}
                                    />
                                </>
                            },
                            users::role::Role::Arbitor(Id{id: quid}) => html!{
                                <VerificationView
                                    coid={cid}
                                    //nqueues={(*comp_view_state).clone().unwrap().declaration.unwrap().queues.len() as i32}
                                    queue_id={quid.clone()}
                                    schem={(*comp_view_state).clone().unwrap().declaration.unwrap().scheme()}
                                />
                            },
                            users::role::Role::Judge(j) => html!{
                                //<>
                                    <JudgeView
                                        coid={cid}
                                        judge={j.clone()}
                                        schem={(*comp_view_state).clone().unwrap().declaration.unwrap().scheme()}
                                    />
                                //</>
                            },
                            _ => html!{<>{"Not implemented yet"}</>},
                        }
                    },
                    None => {
                        html!{<> </>}
                    },
                }
            },
            None => {
                html!{<LoadingView/>}
            },
        }
    };

    html! {
        <>
            <ContextProvider<EventQueueMessageContext> context={message_ctx.clone()}>
                {
                    match (*comp_view_state).clone() {
                        Some(state) => {
                            html! {
                                <>                     
                                        <div class={classes!("stretch", "col-12")}>
                                            //{
                                                /*state.declaration.clone().unwrap().queues.iter().map(
                                                    |coq| {        
                                                        html!{
                                                            <QueueActiveView qui={coq.id.clone()} view={state.clone()} />
                                                        }
                                                    }
                                                ).collect::<Html>()*/
                                            //}
                                        </div>
                                        { stateview }
                                    
                                </>
                            }
                            
                        },
                        None => html! {<></>},
                    }
                }
                <Spacer space="0.5em"/>
                <MessageQueueView coid={cid} view_type={
                    last
                } />
            </ContextProvider<EventQueueMessageContext>>
        </>
    }
}

// ------------------------------------------------------------------------------------
//                                  Useful Functions Here
// ------------------------------------------------------------------------------------

impl EqHistoryMessage {
    pub fn generate_view_id(&self) -> String {
        format!("eqhmessage-view-{}", self.message_id)
    }
}

pub fn queue_context_filter(msg_ctx: &Vec<EqHistoryMessage>, qui: i32) -> Vec<EqHistoryMessage> {
    let mut interested_ids = HashSet::<i32>::new();
    msg_ctx.iter().filter(
        |eqm| {
             if (match &eqm.message {
                Some(hms) => {
                    match &hms.message {
                        Some(ms) => {
                            match ms {
                                eq_message::Message::FinesSetup(x) => {
                                    qui.eq(&x.queue_id)
                                 },
                                eq_message::Message::VoteMessage(x) => {
                                   qui.eq(&x.queue_id)
                                },
                                eq_message::Message::VerifyMessage(x) => {
                                    interested_ids.contains(&x.target_message_id)
                                },
                                eq_message::Message::FixVoting(x) => {
                                    qui.eq(&x.queue_id)
                                },
                                eq_message::Message::TryNext(x) => {
                                    qui.eq(&x.queue_id)
                                },
                                eq_message::Message::Block(x) => {
                                    true // TODO ??
                                },
                                eq_message::Message::SetActiveAction(active_action_state) => todo!(),
                                eq_message::Message::ClearQueueAction(id) => todo!(),
                            }
                        },
                        None => false
                    }
                },
                None => false
            }) {
                interested_ids.insert(eqm.message_id);
                true
            } else {
                false
            }
        }
    ).cloned().collect()
}