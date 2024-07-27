use crate::index::*;
use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_nested_router::prelude::{Switch as RouterSwitch, *};

#[derive(Debug, Default, Clone, PartialEq, Eq, Target)]
pub enum AppRoute {
    #[default]
    Index,
}

#[function_component(Application)]
pub fn app() -> Html {
    html! {
        <Router<AppRoute> default={AppRoute::Index}>
            <RouterSwitch<AppRoute> render={switch_app_route} />
        </Router<AppRoute>>
    }
}

fn switch_app_route(target: AppRoute) -> Html {
    match target {
        AppRoute::Index => html! {<AppPage><Index/></AppPage>},
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct PageProps {
    pub children: Children,
}

#[function_component(AppPage)]
fn page(props: &PageProps) -> Html {
    let tools = html!();
    let brand = html!();
    html! (
        <Page {brand} {tools}>
            { for props.children.iter() }
        </Page>
    )
}
