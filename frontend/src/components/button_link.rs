use gloo_utils::window;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ButtonLinkProps {
    pub href: String,
    #[prop_or_default]
    pub target: Option<String>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub icon: Option<String>,
}

#[function_component(ButtonLink)]
pub fn button_link(props: &ButtonLinkProps) -> Html {
    let href = props.href.clone();
    let target = props.target.clone().unwrap_or_else(|| "_self".into());

    let onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        let _ = window()
            .open_with_url_and_target(&href, &target)
            .expect(&format!(
                "failed to open url with target: href:{href} target:{target}"
            ));
    });

    html! {
        <Button class="button-link" {onclick}>
            { for props.children.iter() }
        </Button>
    }
}
