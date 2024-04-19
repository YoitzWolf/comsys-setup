use web_sys::MouseEvent;
use yew::{Callback, classes, function_component, Html, html};
use yew_autoprops::autoprops;

#[autoprops]
#[function_component(Button)]
pub fn button(text: &String, onclick: Callback<MouseEvent>) -> Html {
    html!(
        <button onclick={ onclick } type={ "button".to_string() } class={ classes!("user-view", "btn") } >
            { text }
        < / button >
    )
}
