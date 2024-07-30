use crate::index::*;
use log::debug;
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
    let sidebar_open = use_state(|| false);
    let toggle_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_: MouseEvent| {
            sidebar_open.set(!*sidebar_open);
            debug!("sidebar_open: {:?}", *sidebar_open);
        })
    };
    let close_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_: MouseEvent| {
            sidebar_open.set(false);
            debug!("sidebar_open: {:?}", *sidebar_open);
        })
    };

    let close_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_| {
            sidebar_open.set(false);
        })
    };

    html! {
        <BackdropViewer>
          <ToastViewer>
            <Router<AppRoute> default={AppRoute::Index}>
              <RouterSwitch<AppRoute> render={move |target| switch_app_route(target, sidebar_open.clone(), close_sidebar.clone())} />
            </Router<AppRoute>>
          </ToastViewer>
        </BackdropViewer>
    }
}

fn switch_app_route(
    target: AppRoute,
    sidebar_open: UseStateHandle<bool>,
    close_sidebar: Callback<()>,
) -> Html {
    match target {
        AppRoute::Index => {
            html! {<AppPage sidebar_open={sidebar_open} close_sidebar={close_sidebar}><Index/></AppPage>}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct PageProps {
    pub children: Children,
    pub sidebar_open: UseStateHandle<bool>,
    pub close_sidebar: Callback<()>,
}

#[function_component(AppPage)]
fn page(props: &PageProps) -> Html {
    let tools = html! { "tools!" };
    let brand = html! { "brand!" };

    let sidebar = html_nested! {
        <PageSidebar>
            <Nav>
                <NavList>
                    <NavExpandable>
                        <NavRouterItem<AppRoute> to={AppRoute::Index}>
                            {"Index"}
                        </NavRouterItem<AppRoute>>
                    </NavExpandable>
                </NavList>
            </Nav>
        </PageSidebar>
    };

    html! {
        <Page {tools} {brand} {sidebar} open={*props.sidebar_open}>
            { for props.children.iter() }
        </Page>
    }
}
