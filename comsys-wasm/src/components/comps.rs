use crate::components::comp_cardview::CompetitionCard;
use crate::components::context_wraps::{DropSetAction, IdListContext};
use crate::components::{AccessDeniedMessage, Button, HidingView, LoadingView, Spacer};
use crate::context::*;
use crate::fs::*;
use crate::grpc::comp::comps_list::CompView;
use crate::grpc::comp::participant::Gender;
use crate::grpc::comp::{
    self, CompDeclaration, CompetitionQueue, CompsList, JudgeScheme, NominationDeclaration,
    Participant, Team,
};
use crate::grpc::generic::id_result::Result::ObjId;
use crate::grpc::generic::{self, id_result, DatePair, Empty, IdResult, IdsList};
use crate::reqs::{get_access_shortcut, get_competition_decl_shortcut};
use crate::router::WebAppRoute;
use crate::utils::parse_dateinput;
use chrono::prelude::*;
use clerk::xslx_conv::prelude::*;
use std::collections::HashMap;
use std::default::Default;
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;
use tonic::Request;
use validator::Validate;
use wasm_bindgen::JsCast;
use web_sys::{js_sys, HtmlElement, HtmlInputElement, MouseEvent};
use yew::functional::{use_reducer, use_reducer_eq};
use yew::{
    classes, function_component, html, use_context, use_effect, use_node_ref, Callback, ContextProvider, Html, NodeRef, Properties, Reducible, TargetCast, UseReducerHandle, UseStateHandle
};
use yew_autoprops::autoprops;
use yew_router::prelude::*;

#[derive(Debug, Validate, PartialEq, Clone, Properties, Default)]
pub struct NominationFormModel {
    #[validate(length(min = 1, message = "Задайте название"))]
    pub title: String,
    pub descr: String,
    //pub categories: Option<Vec<i64>>,
    pub participants: HashMap<i64, Vec<Participant>>,
}

impl NominationFormModel {
    pub fn empty() -> Self {
        Self {
            title: "Новая номинация".to_string(),
            descr: "Без описания".to_string(),
            //categories: Some(1),
            participants: HashMap::<i64, Vec<Participant>>::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Properties)]
pub struct Nomination {
    name: String,
    parts: Vec<Team>,
}
pub(crate) enum NomEditMsg {
    Delete(usize),
    Edit(usize, NominationFormModel),
    NewEmpty,
}

#[autoprops]
#[function_component(EditableNomination)]
pub fn editable_nomination(
    model: &NominationFormModel,
    mykey: usize,
    chng_call: Callback<NomEditMsg>,
) -> Html {
    //let state = yew::use_state_eq(|| { model.clone() } );
    let disabled_state = yew::use_state_eq(|| true);

    let title_in: NodeRef = NodeRef::default();
    let desc_in: NodeRef = NodeRef::default();
    let jujdj_cnt_in: NodeRef = NodeRef::default();

    let change_disable = {
        //let state = state.clone();
        let disabled_state = disabled_state.clone();
        let title_in = title_in.clone();
        let desc_in = desc_in.clone();
        let jujdj_cnt_in = jujdj_cnt_in.clone();
        Callback::from({
            let model = model.clone();
            let chng_call = chng_call.clone();
            move |e: MouseEvent| {
                web_sys::console::log_1(&"Change status changed!".to_string().into());
                if !*(disabled_state) {
                    let new_model = NominationFormModel {
                        title: title_in.cast::<HtmlInputElement>().unwrap().value(),
                        descr: desc_in.cast::<HtmlInputElement>().unwrap().value(),
                        ..model.clone()
                    };
                    chng_call.emit(NomEditMsg::Edit(mykey, new_model));
                };
                disabled_state.set(!(*disabled_state));
            }
        })
    };
    html! {
        <div class={classes!("card")}>
            <div>
                <form>
                    <ul>
                        <li>
                            <h6 class={"marked"}>{"Номинация"}</h6>
                            <div title={"Редактировать"} class={
                                    if (*disabled_state) {classes!("edit-btn")}
                                    else {classes!("marked-ok-pill")}
                                } onclick={change_disable.clone()}> if (!*disabled_state) { {"Сохранить"} } </div>
                        </li>
                        <li><input ref={title_in.clone()} disabled={*disabled_state} type="text" value={(*model).title.clone()}/></li>
                        <li><input ref={desc_in.clone()} disabled={*disabled_state} type="text" value={(*model).descr.clone()}/></li>
                        /*<li>
                            <span>{"Категории:"}</span>
                                if (*disabled_state) {
                                    if (*model).categories.is_some() {
                                        <span class={"marked-ok-pill"}>{format!("{}", (*model).categories.unwrap())}</span>
                                    } else {
                                        <span class={"marked-err-pill"}>{"не задано"}</span>
                                    }
                                } else {
                                  <input ref={jujdj_cnt_in.clone()} disabled={*disabled_state} type="number" min=0 step=1 value={format!("{}", ((*model).categories.unwrap_or(0)) )}/>
                                }
                        </li>*/
                        <li>
                            <span>{"Выступлений:"}{(*model).participants.len()}</span>
                        </li>
                        if !(*disabled_state) {
                            <div class={classes!("marked-err-pill")} onclick={
                                let cn = chng_call.clone();
                                move |_| {
                                    cn.emit(NomEditMsg::Delete(mykey));
                                }
                            }>{"Удалить"}</div>
                        }
                    </ul>
                </form >
            </div>
        </div>
    }
}

#[function_component(NewComp)]
pub fn new_comp() -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");
    let completed_redir = yew::use_state_eq(|| None);

    let finish = {
        let completed_redir = completed_redir.clone();
        Callback::from(move |res: Option<CompDeclaration>| {
            let completed_redir = completed_redir.clone();
            if let Some(decl) = res {
                web_sys::console::log_1(&format!("New Declaration: {:?}", decl).into());
                //get_auth_intercept
                let mut client = Context::get_comp_grpc_client(&ctx);
                wasm_bindgen_futures::spawn_local(async move {
                    let result = client.declare_competition(Request::new(decl)).await;
                    if let Ok(responce) = result {
                        let (meta, id_res, ext) = responce.into_parts();
                        if let Some(IdResult {
                            result: Some(ObjId(x)),
                        }) = id_res.result
                        {
                            web_sys::console::log_1(&format!("Declared: {}", x).into());
                            completed_redir.set(Some((x, id_res.staff.to_owned())))
                        } else {
                            web_sys::console::log_1(&"Declaration Error, check data!".into());
                        }
                    } else {
                        web_sys::console::log_1(
                            &"Server returned Error! Check auth and data".into(),
                        );
                        web_sys::console::log_1(&format!("Debug: {:?}", result).into());
                    }
                })
            }
        })
    };
    let navigator = use_navigator().unwrap();
    fn local_fn_upload_file_and_redirect(x: i32, navigator: Navigator) {
        wasm_bindgen_futures::spawn_local(async move {
            let window = web_sys::window().expect("no global `window` exists");
            let document = window.document().expect("should have a document on window");
            loop {
                if let Some(link) = document.get_element_by_id("download") {
                    link.dyn_into::<HtmlElement>().unwrap().click();
                    navigator.push(&WebAppRoute::ModComp { id: x });
                    break;
                }
            }
        });
    }

    let cctx = use_reducer_eq(|| CompetitionDeclarationWrapper::default());
    html! {

        <>
            <h1>{"Создание соревнования"}</h1>
            {
                match (*completed_redir).clone() {
                    Some((x, Some(y))) => {
                        let data: String = y.passwords.iter().map(
                            |pack| {
                                format!(
                                    "Блок: {}\n\t <Логин>:<Пароль>\n{}",
                                    pack.mark,
                                    pack.logins.iter().map(
                                        |auth| {
                                            format!(
                                                "\t{} : {}",
                                                auth.login,
                                                auth.password
                                            )
                                        }
                                    ).collect::<Vec<String>>().join("\n")
                                )
                            }
                        ).collect::<Vec<String>>()
                            .join("\n-----=====-----\n");

                        web_sys::console::log_1(&data.clone().into());

                        let array = js_sys::Array::from(&data.into());

                        let blob = web_sys::Blob:: new_with_u8_array_sequence(
                            //"CompetitionStaffPasswords.txt",
                            &array
                            //Some("text/plain"),
                            //Some(Utc::now().into())
                        ).unwrap();

                        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

                        html! {
                            <>
                                <a id="download" href={url} download="CompetitionStaffPasswords.txt"/>
                                {
                                    ({
                                        local_fn_upload_file_and_redirect(x, navigator.clone());
                                        html!{<></>}
                                    })
                                }
                                //<Redirect<WebAppRoute> to={}/>
                            </>
                        }
                    },
                    None => html!{
                        <ContextProvider<CompetitionContext> context={cctx}>
                                <NewCompForm res_callback={finish.clone()}/>
                        </ContextProvider<CompetitionContext>>
                    },
                    _ => html!{
                        <span>{"error occured"}</span>
                    }
                }
            }
        </>

    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompetitionDeclarationWrapper {
    declaration: Option<CompDeclaration>,
}

pub enum CompetitionDeclarationWrapperContextAction {
    Setup(CompDeclaration),
    Drop,
}

impl Reducible for CompetitionDeclarationWrapper {
    type Action = CompetitionDeclarationWrapperContextAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CompetitionDeclarationWrapperContextAction::Setup(t) => {
                let mut newc = self.as_ref().clone();
                newc.declaration = Some(t);
                Rc::new(newc)
            }
            CompetitionDeclarationWrapperContextAction::Drop => {
                let mut newc = self.as_ref().clone();
                newc.declaration = None;
                Rc::new(newc)
            } /*_ => {
                  self
              }*/
        }
    }
}

pub type CompetitionContext = UseReducerHandle<CompetitionDeclarationWrapper>;

#[autoprops]
#[function_component(ModComp)]
pub fn modificate_comp(cid: i32) -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");

    let compstate = use_reducer_eq(|| CompetitionDeclarationWrapper::default());
    web_sys::console::log_1(&"Mod Comp Page Loading!".to_string().into());

    if compstate.declaration.is_none() {
        get_competition_decl_shortcut(ctx.clone(), compstate.clone(), cid);
    }

    let finish = {
        //let completed_redir = completed_redir.clone();
        Callback::from(move |res: Option<CompDeclaration>| {
            /*let completed_redir = completed_redir.clone();
            if let Some(decl) = res{
                web_sys::console::log_1(&format!("New Declaration: {:?}", decl).into());
                //get_auth_intercept
                let mut client = Context::get_comp_grpc_client(&ctx);
                wasm_bindgen_futures::spawn_local(
                    async move{
                        let result = client.declare_competition(Request::new(decl)).await;
                        if let Ok(responce) = result {
                            let (meta, id_res ,ext) = responce.into_parts();
                            if let Some(IdResult{ result: Some(ObjId(x)) }  ) = id_res.result {
                                web_sys::console::log_1(&format!("Declared: {}", x).into());
                                completed_redir.set(Some(x))
                            } else {
                                web_sys::console::log_1(&"Declaration Error, check data!".into());
                            }
                        } else {
                            web_sys::console::log_1(&"Server returned Error! Check auth and data".into());
                        }
                    }
                )
            }*/
        })
    };
    html! {
        <>
            <h1>{"Изменение формы соревнования"}</h1>
            {
                if compstate.declaration.is_some() {
                    html!{
                        <ContextProvider<CompetitionContext> context={compstate}>
                            <NewCompForm res_callback={finish.clone()}/>
                            <Spacer space="2em"/>
                            /*<div class="stack">
                                <h3>{"Допуск судей"}</h3>
                                <Spacer space="0.5em"/>
                                <div>
                                    <Button text={"Сгенерировать и Получить данные входа"} onclick={ |_| {} }/>
                                </div>
                            </div>*/
                        </ContextProvider<CompetitionContext>>
                    }
                } else {
                    html!{ <LoadingView/> }
                }
            }
        </>
    }
}

// TODO при добавлении файла таблицы в NewCompForm сбрасываются часть полей формы

#[autoprops]
#[function_component(NewCompForm)]
pub fn new_comp_form(res_callback: &Callback<Option<CompDeclaration>>) -> Html {
    let decl = use_context::<CompetitionContext>().expect("no ctx found");
    let ctx = use_context::<GlobalContext>().expect("no ctx found");
    let filedata: UseStateHandle<Option<Vec<Vec<NominationDeclaration>>>> =
        yew::use_state_eq(|| None);
    let file_status = yew::use_state_eq(|| {
        if decl.declaration.is_none() {
            FileLoadStatus::Waiting
        } else {
            FileLoadStatus::Finished("Данные загружены с сервера".to_string())
        }
    });

    let name_ref = use_node_ref();
    let desc_ref = use_node_ref();
    let place_ref = use_node_ref();
    let date_from_ref = use_node_ref();
    let date_to_ref = use_node_ref();
    let scheme_ref = use_node_ref();
    let query_ref = use_node_ref();

    // title_in.cast::<HtmlInputElement>().unwrap().value()

    use_effect(
        {
            let query_ref = query_ref.clone();
            let decl = decl.clone();
            move || {
                if let Some(val) = query_ref.cast::<HtmlInputElement>() {    
                    if val.value().len() == 0 {
                        val.set_value(&{
                            if let Some(x) = &decl.declaration {
                                x.queues.len()
                            } else {2}
                        }.to_string());
                    }
                }
            }
        }
    );

    let finish = res_callback.reform({
        let name_ref = name_ref.clone();
        let desc_ref = desc_ref.clone();
        let place_ref = place_ref.clone();
        let date_from_ref = date_from_ref.clone();
        let date_to_ref = date_to_ref.clone();
        let scheme_ref = scheme_ref.clone();
        let query_ref = query_ref.clone();
        let filedata = filedata.clone();

        //let noms = noms.clone();
        let ctx = ctx.clone();
        move |_: web_sys::MouseEvent| {
            web_sys::console::log_1(&"Try to parse Comp. Form".into());
            if (ctx.user.get_token()).is_some() {
                web_sys::console::log_1(&"User exist...".into());
                let declar = CompDeclaration {
                    title: name_ref.cast::<HtmlInputElement>().unwrap().value(),
                    public: false,
                    related_organisation_id: 1,
                    queues: {
                        /*let mut n =
                            i32::from_str(&query_ref.cast::<HtmlInputElement>().unwrap().value())
                                .unwrap_or(2);
                        if n <= 0 {
                            n = 1;
                        }*/
                        let nomination_list = if let Some(v) = &(*filedata) {
                            v
                        } else {
                            &vec![]
                        };
                        web_sys::console::log_1(&format!("qUEUES: {:?}", &nomination_list.len()).into());
                        query_ref.cast::<HtmlInputElement>().unwrap().set_value(&nomination_list.len().to_string());
                        nomination_list.iter().enumerate().map(
                            |(qid, noms)| {
                                CompetitionQueue {
                                    id: i32::try_from(qid).unwrap(),
                                    nomination_list: {
                                        noms.clone()
                                    },
                                }
                            }
                        ).collect()
                        /*(0..n)
                            .map(|q| {
                                CompetitionQueue {
                                    id: q,
                                    nomination_list: {
                                        nomination_list[usize::try_from(q).unwrap()].clone()
                                        //v.clone()
                                    }, //.iter<Vec<>>()
                                       //.cloned()
                                       //.skip(q as usize)
                                       //.step_by(n as usize)
                                       //.collect()
                                }
                            })
                            .collect()*/
                    },
                    //part_list: if let Some((d, _)) = &(*filedata) { d.clone() } else { vec![] },
                    scheme: i32::from_str(&scheme_ref.cast::<HtmlInputElement>().unwrap().value())
                        .unwrap_or(0),
                    dates: {
                        let fr = date_from_ref.cast::<HtmlInputElement>().unwrap().value();
                        let to: String = date_to_ref.cast::<HtmlInputElement>().unwrap().value();
                        let fr_split = parse_dateinput(fr);
                        let to_split = parse_dateinput(to);
                        if let (Some(fr), Some(to)) = (fr_split, to_split) {
                            web_sys::console::log_1(&format!("Dates: {:?}, {:?}", fr, to).into());
                            Some(DatePair {
                                begins: Some(
                                    fr, //Timestamp::date(fr.0, fr.1, fr.2).unwrap()
                                ),
                                ends: Some(
                                    to, //Timestamp::date(to.0, to.1, to.2).unwrap()
                                ),
                            })
                        } else {
                            web_sys::console::log_1(
                                &format!("Dates: {:?}, {:?}", fr_split, to_split).into(),
                            );
                            web_sys::console::log_1(&"Invalid Dates!".into());
                            None
                        }
                    },
                    place: {
                        let s = place_ref.cast::<HtmlInputElement>().unwrap().value();
                        if !s.is_empty() {
                            Some(s)
                        } else {
                            None
                        }
                    },
                    descr: {
                        let s = desc_ref.cast::<HtmlInputElement>().unwrap().value();
                        if !s.is_empty() {
                            Some(s)
                        } else {
                            None
                        }
                    },
                };
                if declar.is_valid() {
                    Some(declar)
                } else {
                    None
                }
            } else {
                None
            }
        }
    });

    let upload_table_callback = Callback::from({
        let filedata = filedata.clone();
        let file_status = file_status.clone();
        let query_ref = query_ref.clone();
        move |e: web_sys::Event| {
            let filedata = filedata.clone();
            let file_status = file_status.clone();
            let query_ref = query_ref.clone();
            web_sys::console::log_1(&"File Uploading".to_string().into());
            file_status.set(FileLoadStatus::Loading);
            let input: HtmlInputElement = e.target_unchecked_into();
            let files = input.files();
            if let Some(files) = files {
                if let Some(file) = files.item(0) {
                    let file_name = file.name().clone();
                    filedata.set(None);
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(res) = gloo_file::futures::read_as_bytes(&file.into()).await {
                            if let Ok(declaration) = read_packaged_table::<NominationDeclaration>(
                                res,
                                {
                                    
                                    
                                    // u32::from_str(&query_ref.cast::<HtmlInputElement>().unwrap().value())
                                    //     .unwrap_or(2)
                                    u32::from_str(&query_ref.cast::<HtmlInputElement>().unwrap().value()).unwrap_or(2)
                                },
                                3,
                                1,
                                |x: &String| true,
                            )
                            .await
                            {
                                web_sys::console::log_1(
                                    &format!(
                                        "qs: {:?}, {:?}",
                                        &declaration.len(),
                                        &{
                                            query_ref.cast::<HtmlInputElement>().unwrap().value()
                                            //u32::from_str(&query_ref.cast::<HtmlInputElement>().unwrap().value())
                                        }
                                    ).into()
                                );
                                file_status.set(FileLoadStatus::Finished(file_name));
                                filedata.set(Some(declaration));
                            } else {
                                file_status.set(FileLoadStatus::Error);
                            }
                        } else {
                            file_status.set(FileLoadStatus::Error);
                        }
                    });
                } else {
                    file_status.set(FileLoadStatus::Error);
                    web_sys::console::log_1(&"Unable to read first file".into());
                }
            } else {
                file_status.set(FileLoadStatus::Error);
                web_sys::console::log_1(&"Unable to get FileList".into());
            }
        }
    });

    html! {
        <>
            if (ctx.user.get_token()).is_some() {
                <div class={"stack"}>
                    <h1>{"Форма соревнования:"}</h1>
                    <form class={"stack"}>
                        <label for="new-comp-title">{"Название соревнования:"}</label>
                        { if let Some(x) = &decl.declaration { html!{<input ref={name_ref} id={"new-comp-title"} type="text" value={x.title.clone()}/>} } else { html!{<input ref={name_ref} id={"new-comp-title"} type="text" />} } }

                        <label for="new-comp-place">{"Место проведения:"}</label>
                        {
                                if let Some(
                                    CompDeclaration{place: Some(place), ..}
                                ) = &decl.declaration {
                                    html!{
                                        <input ref={place_ref} id={"new-comp-place"} type="text" value={place.clone()}/>
                                    }
                                } else {
                                    html!{
                                        <input ref={place_ref} id={"new-comp-place"} type="text"/>
                                    }
                                }
                        }

                        <label for="new-comp-description">{"Описание:"}</label>
                        {
                            if let Some(
                                    CompDeclaration{descr: Some(descr), ..}
                                ) = &decl.declaration {
                                    html!{
                                        <textarea ref={desc_ref} id={"new-comp-description"} type="text" placeholder={"Описание"} value={descr.clone()}></textarea>
                                    }
                                } else {
                                    html!{
                                        <textarea ref={desc_ref} id={"new-comp-description"} type="text" placeholder={"Описание"}></textarea>
                                    }
                                }
                        }

                        <div class="tablebox">
                            <div>
                                <label for="new-comp-start-date">{"Дата начала:"}</label>
                                {
                                    if let Some(
                                        CompDeclaration{dates: Some(DatePair{begins: Some(res), ..}), ..}
                                    ) = &decl.declaration {
                                        let date: DateTime<Local> = DateTime::<Utc>::from_timestamp(res.seconds, 0).unwrap().into();
                                        html!{
                                            <input ref={date_from_ref} id="new-comp-start-date" type="date" value={date.format("%Y-%m-%d").to_string()}/>
                                        }
                                    } else {
                                        html!{ <input ref={date_from_ref} id="new-comp-start-date" type="date"/> }
                                    }
                                }
                            </div>
                            <div>
                                <label for="new-comp-end-date">{"Дата окончания:"}</label>
                                {
                                    if let Some(
                                        CompDeclaration{dates: Some(DatePair{ends: Some(res), ..}), ..}
                                    ) = &decl.declaration {
                                        let date: DateTime<Local> = DateTime::<Utc>::from_timestamp(res.seconds, 0).unwrap().into();
                                        html!{
                                            <input ref={date_to_ref} id="new-comp-end-date" type="date" value={date.format("%Y-%m-%d").to_string()}/>
                                        }
                                    } else {
                                        html!{ <input ref={date_to_ref} id="new-comp-end-date" type="date"/> }
                                    }
                                }
                            </div>
                        </div>
                        <div class="tablebox">
                            <div>
                                <label for="judge-scheme">{"Схема оценивания:"}</label>
                                //<input class={"hidden"} list={"judge-scheme-list"}/>
                                <select ref={scheme_ref} id={"judge-scheme"}>
                                 <option selected={if let Some(x) = &decl.declaration {x.scheme == 0} else {true} } value={(JudgeScheme::FourFourOne as i32).to_string()}>{"4/4/1"}</option>
                                 <option selected={if let Some(x) = &decl.declaration {x.scheme == 1} else {true} } value={(JudgeScheme::FourFourTwo as i32).to_string()}>{"4/4/2"}</option>
                                 <option selected={if let Some(x) = &decl.declaration {x.scheme == 2} else {false} }value={(JudgeScheme::SixSixTwo as i32).to_string()}>{"6/6/2"}</option>
                                </select>
                            </div>
                            <div>
                                <label for="queries">{"Очередей:"}</label>
                                <input ref={query_ref} id="queries" type="number" min=1 step=1/>
                            </div>
                        </div>

                        <span class={"marked-info"}>{"Система считывает данные формата .xlsx. Пожалуйста, обратитесь к "}<b>{"руководству Пользователя"}</b>{"за пояснениями к форме таблицы!"}</span>
                        <label for="file-upload" class={classes!("marked", "cursor", "inline_btn")}>
                            {"Выбрать Файл со списком участников"}
                            <div title={"Загрузить Файл"} class={classes!("add-file-btn")}/>
                        </label>
                        <input
                            class={"hidden"}
                            id="file-upload"
                            type="file"
                            accept=".xlsx"
                            multiple={false}
                            onchange={ upload_table_callback } />
                        {
                            match (*file_status).clone() {
                                FileLoadStatus::Blocked => {
                                    html!{<AccessDeniedMessage/>}
                                },
                                FileLoadStatus::Waiting => {
                                    html!{<></>}
                                },
                                FileLoadStatus::Loading => {
                                    html!{ <LoadingView/> }
                                },
                                FileLoadStatus::Error => {
                                    html!{ <span class={"marked-err-pill"}>{"Неверный Формат Файла!"}</span>}
                                },
                                FileLoadStatus::Finished(x) => {
                                    html!{
                                        <>
                                            <span class={"marked-ok-pill"}>{"Файл загружен: "}{x}</span>
                                            /*

                                                <HidingView>
                                                <table class={"table"}>
                                                    <caption>
                                                        {"Список участников"}
                                                    </caption>
                                                    <tbody contenteditable={"true"}>
                                                    {
                                                        //let decl = decl.clone();
                                                        match &decl.declaration {
                                                            Some(x) => {
                                                                x.part_list.iter().map(
                                                                    |participant| {
                                                                        html! {
                                                                            <tr>
                                                                                <td scope="row">{participant.uid}</td>
                                                                                <td scope="row">{participant.name.clone()}</td>
                                                                                <td scope="row">{ Gender::try_from(participant.gender).unwrap().into_rus_flag() }</td>
                                                                                <td scope="row">{
                                                                                    match participant.birthdate {
                                                                                            Some(x) => DateTime::from_timestamp(x.seconds, 0).unwrap().format("%d.%m.%Y").to_string(),
                                                                                            None => "unknown".to_string(),
                                                                                        }
                                                                                }</td>
                                                                            </tr>
                                                                        }
                                                                    }
                                                                ).collect::<Html>()
                                                            },
                                                            None => {
                                                                html!{
                                                                    <></>
                                                                }
                                                            }
                                                        }
                                                    }
                                                    </tbody>
                                                </table>
                                            </HidingView>

                                             */
                                        </>
                                    }
                                }
                            }
                        }
                        <span class={"marked-info"}>
                            {"*Если вы загрузили не тот файл - просто загрузите новый. Система позволяет загрузить только одну таблицу!"}
                        </span>
                        <span class={"marked-info"}>
                            {"*После загрузки файла система сама создаст Номинации и Списки участников. Если списка участников еще нет, то его можно будет задать далее."}
                        </span>
                        /*<div class={classes!("card-list")}>
                            //<span class={"marked-info"}>{"Данные входа для судейства будут созданы автоматически после отправки формы."}</span>
                            <div class={classes!("card-list-inner")}> // Nominations
                                {
                                    (*noms).clone().into_iter().enumerate().map(
                                        |i| {
                                            html!{
                                                <EditableNomination
                                                    key={i.0}
                                                    mykey={i.0}
                                                    model={i.1.clone()}
                                                    chng_call={change_noms.clone()}
                                                />
                                            }
                                        }
                                    ).collect::<Html>()
                                }
                            </div>
                        </div>*/
                        <Button disabled={false} text={"Сохранить и отправить"} onclick={ finish }/>
                    </form>
                </div>
            } else {
                <div class={"stack"}>
                    <AccessDeniedMessage/>
                    <LoadingView/>
                </div>
            }
        </>
    }
}

#[autoprops]
#[function_component(CompetitionListViewer)]
pub fn competition_list_view() -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");

    let mut cid_list = use_reducer_eq(|| IdListContext::default());

    if cid_list.0.len() == 0 {
        let mut client = Context::get_comp_grpc_client(&ctx);
        wasm_bindgen_futures::spawn_local({
            let cid_list = cid_list.clone();
            async move {
                let result = client.get_comps_ids(Empty {}).await;
                if let Ok(responce) = result {
                    let (meta, id_lst, ext) = responce.into_parts();
                    cid_list.dispatch(DropSetAction::Set(IdListContext(id_lst.obj_ids.into())));
                } else {
                    web_sys::console::log_1(&"Server returned Error!".into());
                }
            }
        })
    } else {
        web_sys::console::log_1(&"Mew :3".into());
    }
    // TODO: сделать странички, чтобы не запрашивать все въюшки

    /*wasm_bindgen_futures::spawn_local(
        {
            let ctx = ctx.clone();
            let mut cid_list = cid_list.clone();
            async move {
                let mut client = Context::get_comp_grpc_client(&ctx);
                match client.get_comps_ids(Request::new(Empty{})).await {
                    Ok(x) => {
                        cid_list.dispatch(DropSetAction::Set(IdListContext(x.into_inner().obj_ids)));
                    },
                    Err(_) => {

                    },
                };
            }
        }
    );*/

    let v = (*cid_list.0.clone())
        .iter()
        .map(|i| {
            html! {
                <>
                    <CompetitionCard
                        coid={i.clone()}
                    />
                </>
            }
        })
        .collect::<Vec<Html>>();

    html! {
        <>
            <a href={"/webapp/comps/new"} class={"marked"}>{"создать новое"}</a>
            <div class={"stack"}>
                { v }
            </div>
        </>
    }
}
