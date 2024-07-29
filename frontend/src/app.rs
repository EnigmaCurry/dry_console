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

    html! {
        <BackdropViewer>
          <ToastViewer>
            <Router<AppRoute> default={AppRoute::Index}>
              <RouterSwitch<AppRoute> render={move |target| switch_app_route(target, close_sidebar.clone(), sidebar_open.clone())} />
            </Router<AppRoute>>
          </ToastViewer>
        </BackdropViewer>
    }
}

fn switch_app_route(
    target: AppRoute,
    close_sidebar: Callback<MouseEvent>,
    sidebar_open: UseStateHandle<bool>,
) -> Html {
    match target {
        AppRoute::Index => {
            html! {<AppPage close_sidebar={close_sidebar} sidebar_open={sidebar_open}><Index/></AppPage>}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct PageProps {
    pub children: Children,
    pub close_sidebar: Callback<MouseEvent>,
    pub sidebar_open: UseStateHandle<bool>,
}

#[function_component(AppPage)]
fn page(props: &PageProps) -> Html {
    let tools = html! { "tools!" };
    let brand = html! { "brand!" };
    let sidebar = html_nested! {
        <PageSidebar open={false}>
          <Nav>
            <NavList>
              <NavExpandable title="Navigation">
                // <NavItem>
                //   <a href="#" onclick={props.close_sidebar.reform(|e: MouseEvent| {
                //       e.prevent_default();
                //       e
                //   })}>
                    <NavRouterItem<AppRoute> to={AppRoute::Index}>
                      {"Index"}
                    </NavRouterItem<AppRoute>>
                //   </a>
                // </NavItem>
              </NavExpandable>
            </NavList>
          </Nav>
        </PageSidebar>
    };

    html! (
        <Page {tools} {brand} {sidebar}>
            { for props.children.iter() }
        </Page>
    )
}
