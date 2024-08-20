use web_sys::MouseEvent;
use yew::prelude::*;
use yew_autoprops::autoprops;

#[autoprops]
#[function_component(Button)]
pub fn button(text: &String, onclick: Callback<MouseEvent>, #[prop_or(false)] disabled: bool) -> Html {
    html!(
        <button disabled={disabled} onclick={ onclick } type={ "button".to_string() } class={ classes!("btn") } >
            { text }
        < / button >
    )
}

#[autoprops]
#[function_component(Spacer)]
pub fn spacer(space: &String) -> Html {
    html!(
        <div style={format!("display:block;width=100%;height:{};", space)}>
        </div>
    )
}

#[autoprops]
#[function_component(NoStyleButton)]
pub fn not_styled_button(text: &String, onclick: Callback<MouseEvent>, #[prop_or(false)] disabled: bool) -> Html {
    html!(
        <button disabled={disabled} onclick={ onclick } type={ "button".to_string() } class={ classes!("no-styled-btn") }> //  
            { text }
        < / button >
    )
}

#[function_component(AccessDeniedMessage)]
pub fn acc_denied_msg() -> Html {
    html! {
        <span class={"marked-err"}>{"Нет доступа!"}</span>
    }
}


#[function_component(LoadingView)]
pub fn loading() -> Html {
    html! {
        <div class="lds-dual-ring"></div>
    }
}

#[autoprops]
#[function_component(HidingView)]
pub fn vertical_hiding_view(children: &Html) -> Html {
    let hidden = use_state_eq(|| {true});

    let onclick = Callback::from({
        let hidden = hidden.clone();
        move |_| {
            hidden.set(! *hidden);
        }
    });

    html! {
        <>
            <div class="hiding-box">
                <div class="hide-button marked" onclick={onclick}>
                    {
                        if *hidden {
                            {"раскрыть"}
                        } else {
                            {"скрыть"}
                        }
                    }
                </div>
                {
                    if ! *hidden {
                        {children.clone()}
                    } else { html!{{""}} }
                }
            </div>
        </>
    }
}