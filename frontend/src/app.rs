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
        <BackdropViewer>
          <ToastViewer>
            <Router<AppRoute> default={AppRoute::Index}>
              <RouterSwitch<AppRoute> render={switch_app_route} />
            </Router<AppRoute>>
          </ToastViewer>
        </BackdropViewer>
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
     </PageSidebar> };
    html! (
        <Page {tools} {brand} {sidebar} >
            { for props.children.iter() }
        </Page>
    )
}
