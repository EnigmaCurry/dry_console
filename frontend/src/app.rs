use crate::components::logout;
use crate::components::ButtonLink;
use crate::pages::{apps, login, routes, workstation};
use anyhow::{anyhow, Error};
pub use dry_console_dto::session::SessionState;
use gloo_events::EventListener;
use gloo_net::http::Request;
use gloo_storage;
use gloo_storage::Storage;
use patternfly_yew::prelude::*;
use strum_macros::Display;
use strum_macros::EnumIter;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;
use yew_nested_router::prelude::{Switch as RouterSwitch, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
enum TopMenuChoices {
    Workstation,
    Apps,
    Routes,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Target, EnumIter, Display)]
pub enum AppRoute {
    #[default]
    Workstation,
    Apps,
    Routes,
    Login,
}

impl Into<&'static str> for AppRoute {
    fn into(self) -> &'static str {
        match self {
            AppRoute::Login => "Login",
            AppRoute::Workstation => "Workstation",
            AppRoute::Apps => "Apps",
            AppRoute::Routes => "Routes",
        }
    }
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

async fn check_session_state() -> Result<SessionState, Error> {
    let response = Request::get("/api/session").send().await?;

    match response.status() {
        200 => {
            let session: SessionState = response.json().await?;
            Ok(session)
        }
        i => Err(anyhow!("Bad response code: {i}")),
    }
}

#[function_component(Application)]
pub fn app() -> Html {
    let session_state = use_state(|| SessionState::default());
    let checking_session = use_state(|| true);

    {
        let session_state = session_state.clone();
        let checking_session = checking_session.clone();
        use_effect_with((), move |_| {
            let session_state = session_state.clone();
            let checking_session = checking_session.clone();
            spawn_local(async move {
                match check_session_state().await {
                    Ok(state) => {
                        session_state.set(state);
                    }
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
                <Router<AppRoute> default={AppRoute::Workstation}>
            <RouterSwitch<AppRoute> render={move |route| {
                        if *checking_session {
                            // Optionally, you could return a loading indicator here while checking the session
                            html! { <div>{"Checking session..."}</div> }
                        } else if session_state.logged_in || matches!(route, AppRoute::Login) {
                            switch_app_route(route, session_state.clone())
                        } else {
                            html! { <Redirect to={AppRoute::Login} /> }
                        }
                    }} />
                </Router<AppRoute>>
            </ToastViewer>
        </BackdropViewer>
    }
}

fn switch_app_route(target: AppRoute, session_state: UseStateHandle<SessionState>) -> Html {
    match target {
        AppRoute::Login => {
            html! {<AppPage session_state={session_state.clone()}><login::Login session_state={session_state.clone()}/></AppPage>}
        }
        AppRoute::Workstation => {
            html! {<AppPage {session_state}><workstation::Workstation/></AppPage>}
        }
        AppRoute::Apps => {
            html! {<AppPage {session_state}><apps::Apps/></AppPage>}
        }
        AppRoute::Routes => {
            html! {<AppPage {session_state}><routes::Routes/></AppPage>}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct AppPageProps {
    pub children: Children,
    pub session_state: UseStateHandle<crate::app::SessionState>,
}

#[function_component(TopBarMenu)]
fn top_bar_menu() -> Html {
    let navigator = use_router::<AppRoute>().unwrap();
    //log::info!("{:?}", navigator.active_target);
    let choice = match navigator.active_target {
        None => None,
        Some(ref c) => match c {
            AppRoute::Login => Some(TopMenuChoices::Workstation),
            AppRoute::Workstation => Some(TopMenuChoices::Workstation),
            AppRoute::Apps => Some(TopMenuChoices::Apps),
            AppRoute::Routes => Some(TopMenuChoices::Routes),
            #[allow(unreachable_patterns)]
            _ => None,
        },
    };
    let selected = use_state(|| choice);
    let callback = {
        let selected = selected.clone();
        use_callback(selected.clone(), move |input: TopMenuChoices, selected| {
            selected.set(Some(input));
            let route = match input {
                TopMenuChoices::Workstation => AppRoute::Workstation,
                TopMenuChoices::Apps => AppRoute::Apps,
                TopMenuChoices::Routes => AppRoute::Routes,
            };
            navigator.push(route); // This will navigate and trigger a re-render
            ()
        })
    };

    html! {
        <ToggleGroup>
            <ToggleGroupItem
                text="Workstation"
                key=0
                onchange={let cb = callback.clone(); move |_| { cb.emit(TopMenuChoices::Workstation); () }}
                selected={*selected == Some(TopMenuChoices::Workstation)}
            />
            <ToggleGroupItem
                text="Apps"
                key=1
                onchange={let cb = callback.clone(); move |_| { cb.emit(TopMenuChoices::Apps); () }}
                selected={*selected == Some(TopMenuChoices::Apps)}
            />
            <ToggleGroupItem
                text="Routes"
                key=2
                onchange={let cb = callback.clone(); move |_| { cb.emit(TopMenuChoices::Routes); () }}
                selected={*selected == Some(TopMenuChoices::Routes)}
            />
        </ToggleGroup>
    }
}

fn sidebar(
    darkmode: UseStateHandle<bool>,
    onthemeswitch: Callback<bool>,
    session_state: UseStateHandle<crate::app::SessionState>,
) -> Html {
    // let nav_items = AppRoute::iter()
    //     .map(|route| {
    //         let route_name: &'static str = route.clone().into();
    //         html_nested! {
    //             <NavItem>
    //                 <NavRouterItem<AppRoute> to={route}>
    //                     {route_name}
    //                 </NavRouterItem<AppRoute>>
    //             </NavItem>
    //         }
    //     })
    //     .collect::<Html>();

    html_nested! {
        <Nav>
            <NavList>
                // <NavExpandable title="Routes" expanded={false}>
                //     {nav_items}
                // </NavExpandable>
                <NavExpandable title="Preferences" expanded={true}>
                    <NavItem>
                        <patternfly_yew::prelude::Switch
                            checked={*darkmode}
                            onchange={onthemeswitch}
                            label="Dark Theme"
                        />
                    </NavItem>
                </NavExpandable>
                <NavExpandable title="Session" expanded={true}>
                    <NavItem>
                      <logout::Logout {session_state}/>
                    </NavItem>
                </NavExpandable>
                <NavExpandable title="Source code" expanded={true}>
                    <NavItem>
                      <ButtonLink href="https://github.com/EnigmaCurry/dry_console">{"Github"}</ButtonLink>
                    </NavItem>
                </NavExpandable>
            </NavList>
        </Nav>
    }
    .into()
}

#[function_component(AppPage)]
fn page(props: &AppPageProps) -> Html {
    log::debug!("rendering page");
    let brand = html! { <a href="/">{"dry_console"}</a> };

    let darkmode = use_state_eq(|| {
        if let Some(storage) = gloo_storage::LocalStorage::get("dark_mode").ok() {
            storage
        } else {
            gloo_utils::window()
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .map(|m| m.matches())
                .unwrap_or_default()
        }
    });

    {
        let darkmode = darkmode.clone();
        use_effect_with(*darkmode, move |state| {
            if let Err(e) = gloo_storage::LocalStorage::set("dark_mode", *state) {
                log::error!("Failed to store dark mode state: {:?}", e);
            }

            match state {
                true => gloo_utils::document_element().set_class_name("pf-v5-theme-dark"),
                false => gloo_utils::document_element().set_class_name(""),
            }
        });
    }

    // toggle dark mode
    let onthemeswitch = use_callback(darkmode.setter(), |state, setter| setter.set(state));

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

    let sidebar = html_nested! {<PageSidebar>{sidebar(darkmode.clone(), onthemeswitch.clone(), props.session_state.clone())}</PageSidebar>};
    let tools = html!(
        <Toolbar full_height=true>
            <ToolbarContent>
                <ToolbarGroup
                    modifiers={ToolbarElementModifier::Right.all()}
                    variant={GroupVariant::IconButton}
             >
             { if props.session_state.logged_in {
                 html! { <TopBarMenu /> }
             } else {
                 html! { }
             }}
                </ToolbarGroup>
            </ToolbarContent>
        </Toolbar>
    );

    html! {
        <Page {brand} {sidebar} {tools} {open}>
            { for props.children.iter() }
        </Page>
    }
}
