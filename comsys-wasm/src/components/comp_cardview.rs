use yew::{classes, function_component, html, use_context, use_state_eq, Html};
use yew_autoprops::autoprops;

use crate::{components::{HSpacer, LoadingView, Spacer}, context::{Context, GlobalContext}, grpc::generic::IdsList};


#[autoprops]
#[function_component(CompetitionCard)]
pub fn competition_card(coid: i32) -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");
    
    let view = use_state_eq(|| {None});

    if let None = *view {      
        let view = view.clone();
        wasm_bindgen_futures::spawn_local(
            {
                let view = view.clone();
                //let cid_list = cid_list.clone();
                async move{
                    let mut client = Context::get_comp_grpc_client(&ctx);
                    let result = client.get_comps_views( IdsList{obj_ids: vec![coid]} ).await;
                    if let Ok(responce) = result {
                        let (meta, id_lst ,ext) = responce.into_parts();
                        if let Some(x) = id_lst.comp_views.get(&coid) {
                            view.set(Some(x.clone()));
                        }
                    } else {
                        web_sys::console::log_1(&"Server returned Error!".into());
                    }
                }
            }
        );
    }

    html! {
        match (*view).clone() {
            Some(view) => {
                let decl = view.declaration.unwrap();
                html!{<>
                    <div class={classes!("event-card")}>
                    <div class={"event-header"}>
                            <h4>{decl.title}</h4>
                            <HSpacer space="0.5em"/>
                            <a href={format!("/webapp/comps/mod/{}", coid)} class={"marked"}>{"изменить"}</a>
                            <HSpacer space="0.5em"/>
                            <a href={format!("/webapp/comps/running/{}", coid)} class={"marked"}>{"перейти"}</a>
                        </div>
                        <div class={classes!("event-data")}>
                            <div>
                                <div class={classes!("card")}>
                                    <span>{decl.descr.unwrap_or("Нет описания".to_string())}</span>
                                    /*<div>
                                        <ul>
                                            <li>{"Даты проведения: "}<span class={"marked"}>{"24.09.24"}</span></li>
                                            <li>{"Регистрация до: "}<span class={"marked"}>{"21.09.24"}</span>{": "}<span class={"marked-ok"}>{"открыто"}</span></li>
                                            <li>{"Даты проведения: "}<span class={"marked"}>{"24.09.24"}</span></li>
                                        </ul>
                                    </div>*/
                                </div>
                                /*<div class={classes!("card")}>
                                    <div>
                                        <ul>
                                            <li>{"Трансляция: "}<span class={"marked-err"}>{"нет"}</span></li>
                                            <li>{"Организатор: "}<span class={"marked"}>{"ЦК ВКПб"}</span></li>
                                            <li>{"Результаты: "}<span class={"marked-err"}>{"нет"}</span></li>
                                        </ul>
                                    </div>
                                </div>*/
                            </div>
                        </div>
                    </div>
                    <Spacer space="0.5em"/>
                </>}
            },
            None => {
                html!{
                    <LoadingView />
                }
            },
        }
    }
}
