use web_sys::HtmlInputElement;
use yew::{function_component, html, use_context, Callback, Html, NodeRef};
use yew_autoprops::autoprops;

use crate::{components::{Button, LoadingView, Spacer}, context::{Context, GlobalContext}, grpc::generic::StringMessage};

#[autoprops]
#[function_component(UserCabinet)]
pub fn cabinete() -> Html {
    let ctx = use_context::<GlobalContext>().expect("no ctx found");
    
    let inp_ref = NodeRef::default();

    let change_name = Callback::from (
        {
            let inp_ref = inp_ref.clone();
            let ctx = ctx.clone();
            move |_| {   
                let inp_ref = inp_ref.clone();
                let ctx = ctx.clone();
                wasm_bindgen_futures::spawn_local(
                    async move {
                        let mut grpc_client = Context::get_user_mng_grpc_client(&ctx);
                        let resp = grpc_client.setup_selfname(
                            StringMessage{ str: inp_ref.cast::<HtmlInputElement>().unwrap().value() }
                        ).await;
                        match resp {
                            Ok(resp) => {  },
                            Err(status) => {  }

                        };
                    }
                );
            }
        }
    );

    let v = match ctx.user.ready() {
        true => {
            let dat = ctx.user.get_user_data().clone().unwrap();
            html! {
                <>
                    <form class={"stack"} >
                        <h1>{"User Cabinet"}</h1>
                        <Spacer space="1em"/>
                        <span class={"stretch"}>{"login: "} <span class={"marked"}>{dat.get_name().to_string()}</span> </span>
                        <Spacer space="0.5em"/>
                        <span class={"stretch"}>{"Имя: "} <input ref={inp_ref} value={dat.get_selfname().to_string()}/> </span>
                        <Spacer space="0.5em"/>
                        <Button text="Сохранить" onclick={change_name}/>
                    </form>
                </>
            }
        },
        false => { 
            html! {
                <>
                    <h1>{"Need authorisation"}</h1>
                    <Spacer space="1em"/>
                    <LoadingView />
                </>
            }
        },
    };
    
    html! {
        <>
            {v}
        </>
    }
}