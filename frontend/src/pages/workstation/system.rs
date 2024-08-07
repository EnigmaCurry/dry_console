use gloo::net::http::Request;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VChild;

#[function_component(System)]
pub fn system() -> Html {
    html! {
        <>
        {" System "}
        </>
    }
}
