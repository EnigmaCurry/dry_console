use crate::pages::index;
use crate::pages::login;
use anyhow::Error;
use gloo_events::EventListener;
use gloo_net::http::Request;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;
use yew_nested_router::prelude::{Switch as RouterSwitch, *};

#[derive(Debug, Default, Clone, PartialEq, Eq, Target, EnumIter)]
pub enum AppRoute {
    #[default]
    Index,
    Login,
}

impl Into<&'static str> for AppRoute {
    fn into(self) -> &'static str {
        match self {
            AppRoute::Index => "Index",
            AppRoute::Login => "Login",
        }
    }
}

#[derive(Deserialize, Debug)]
struct SessionState {
    logged_in: bool,
}

#[function_component(Redirect)]
fn redirect(props: &RedirectProps) -> Html {
    let router = use_router::<AppRoute>();
    let cloned_router = router.clone();

    let to = props.to.clone();

    use_effect_with(to.clone(), move |_| {
        if let Some(router) = cloned_router {
            router.push(to);
        }
        || ()
    });

    // Render output based on whether the router context is available
    if router.is_some() {
        html! { <p>{ "Redirecting..." }</p> }
    } else {
        html! { <p>{ "Routing context not available" }</p> }
    }
}

#[derive(Properties, PartialEq)]
struct RedirectProps {
    to: AppRoute,
}

async fn check_logged_in() -> Result<bool, Error> {
    let response = Request::get("/api/session").send().await?;

    if response.status() == 200 {
        let session: SessionState = response.json().await?;
        Ok(session.logged_in)
    } else {
        Ok(false)
    }
}

#[function_component(Application)]
pub fn app() -> Html {
    let logged_in = use_state(|| false);
    let checking_session = use_state(|| true);

    {
        let logged_in = logged_in.clone();
        let checking_session = checking_session.clone();
        use_effect_with((), move |_| {
            let logged_in = logged_in.clone();
            let checking_session = checking_session.clone();
            spawn_local(async move {
                match check_logged_in().await {
                    Ok(status) => logged_in.set(status),
                    Err(_) => log::error!("Failed to fetch session status"),
                }
                checking_session.set(false); // Session check is complete
            });
            || ()
        });
    }

    html! {
        <BackdropViewer>
            <ToastViewer>
                <Router<AppRoute> default={AppRoute::Index}>
                    <RouterSwitch<AppRoute> render={move |route| {
                        if *checking_session {
                            // Optionally, you could return a loading indicator here while checking the session
                            html! { <div>{"Checking session..."}</div> }
                        } else if *logged_in || matches!(route, AppRoute::Login) {
                            switch_app_route(route, logged_in.clone())
                        } else {
                            html! { <Redirect to={AppRoute::Login} /> }
                        }
                    }} />
                </Router<AppRoute>>
            </ToastViewer>
        </BackdropViewer>
    }
}

fn switch_app_route(target: AppRoute, logged_in: UseStateHandle<bool>) -> Html {
    match target {
        AppRoute::Index => html! {<AppPage><index::Index/></AppPage>},
        AppRoute::Login => html! {<AppPage><login::Login logged_in={logged_in}/></AppPage>},
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct PageProps {
    pub children: Children,
}

fn sidebar() -> Html {
    let nav_items = AppRoute::iter()
        .map(|route| {
            let route_name: &'static str = route.clone().into();
            html_nested! {
                <NavItem>
                    <NavRouterItem<AppRoute> to={route}>
                        {route_name}
                    </NavRouterItem<AppRoute>>
                </NavItem>
            }
        })
        .collect::<Html>();

    html_nested! {
            <Nav>
                <NavList>
                    <NavExpandable title="Routes">
                        {nav_items}
                    </NavExpandable>
                </NavList>
            </Nav>
    }
    .into()
}

#[function_component(AppPage)]
fn page(props: &PageProps) -> Html {
    log::debug!("rendering page");
    let brand = html! { "brand!" };

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

    // track window width
    let window_width = use_state(|| {
        window()
            .expect("Unable to get window object")
            .inner_width()
            .expect("Unable to get window width")
            .as_f64()
            .expect("Should be a number") as f64
    });

    {
        let window_width = window_width.clone();
        use_effect_with((), move |_| {
            let window_width = window_width.clone();
            let listener = EventListener::new(&window().unwrap(), "resize", move |_| {
                let new_width = window()
                    .expect("Unable to get window object")
                    .inner_width()
                    .expect("Unable to get window width")
                    .as_f64()
                    .expect("Should be a number");
                window_width.set(new_width);
            });

            || drop(listener)
        });
    }

    let open = match *window_width {
        width if width < 1200.0 => false,
        _ => true,
    };
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

    let sidebar = html_nested! {<PageSidebar>{sidebar()}</PageSidebar>};
    html! {
        <Page {brand} {sidebar} {tools} {open}>
            { for props.children.iter() }
        </Page>
    }
}
