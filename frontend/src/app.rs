use crate::index::*;
use log::info;
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
    let brand = html! { "brand!" };

    let sidebar = html_nested! {
        <PageSidebar>
            <Nav>
                <NavList>
                    <NavExpandable>
                        <NavItem>
                            <NavRouterItem<AppRoute> to={AppRoute::Index}>
                                {"Index"}
                            </NavRouterItem<AppRoute>>
                        </NavItem>
                    </NavExpandable>
                </NavList>
            </Nav>
        </PageSidebar>
    };

    // track dark mode state
    let darkmode = use_state_eq(|| {
        gloo_utils::window()
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .map(|m| m.matches())
            .unwrap_or_default()
    });

    // apply dark mode
    use_effect_with(*darkmode, |state| match state {
        true => gloo_utils::document_element().set_class_name("pf-v5-theme-dark"),
        false => gloo_utils::document_element().set_class_name(""),
    });

    // toggle dark mode
    let onthemeswitch = use_callback(darkmode.setter(), |state, setter| setter.set(state));

    let tools = html!(
        <Toolbar full_height=true>
            <ToolbarContent>
                <ToolbarGroup
                    modifiers={ToolbarElementModifier::Right.all()}
                    variant={GroupVariant::IconButton}
                >
                    <ToolbarItem>
                        <patternfly_yew::prelude::Switch checked={*darkmode} onchange={onthemeswitch} label="Dark Theme" />
                    </ToolbarItem>
                </ToolbarGroup>
            </ToolbarContent>
        </Toolbar>
    );

    html! {
        <Page {brand} {sidebar} {tools} open={false}>
            { for props.children.iter() }
        </Page>
    }
}
