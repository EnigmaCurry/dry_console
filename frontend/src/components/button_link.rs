use gloo_utils::window;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ButtonLinkProps {
    pub href: String,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub icon: Option<String>,
}

#[function_component(ButtonLink)]
pub fn button_link(props: &ButtonLinkProps) -> Html {
    let href = props.href.clone();

    let onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        window().location().set_href(&href).unwrap();
    });

    html! {
        <Button class="button-link" {onclick}>
            { for props.children.iter() }
        </Button>
    }
}
