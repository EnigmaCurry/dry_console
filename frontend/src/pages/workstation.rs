use dry_console_dto::workstation::{Platform, WorkstationState, WorkstationUser};
use gloo::console::debug;
use gloo::net::http::Request;
use gloo_events::EventListener;
use gloo_utils::window;
use patternfly_yew::prelude::*;
use std::rc::Rc;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};
use yew::html::Children;
use yew::prelude::*;

mod dependencies;
mod install_d_rymcg_tech;
mod system;

#[derive(Clone, PartialEq, Debug)]
pub struct SystemInfo {
    hostname: String,
    platform: Platform,
    user: WorkstationUser,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemInfoContext {
    pub system_info: UseStateHandle<Option<Rc<SystemInfo>>>,
}

#[derive(Clone, Debug, Properties, PartialEq)]
struct SystemInfoProps {
    children: Children,
}

#[function_component(SystemInfoProvider)]
fn system_info_provider(props: &SystemInfoProps) -> Html {
    let system_info = use_state(|| None);
    {
        let system_info = system_info.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if system_info.is_none() {
                let fetched_data: WorkstationState = Request::get("/api/workstation/")
                    .send()
                    .await
                    .expect("Failed to fetch data")
                    .json()
                    .await
                    .expect("Failed to parse JSON");

                let hostname = fetched_data.hostname;
                let platform = fetched_data.platform;
                let user = fetched_data.user;
                let info = SystemInfo {
                    hostname,
                    platform,
                    user,
                };
                debug!(format!("{:?}", info));
                system_info.set(Some(Rc::new(info)));
            }
        });
    }

    let context =
        use_context::<UseStateHandle<Option<Rc<SystemInfo>>>>().expect("context not found");
    context.set((*system_info).clone());

    html! { for props.children.iter() }
}

#[hook]
fn use_system_info() -> Option<Rc<SystemInfo>> {
    use_context::<UseStateHandle<Option<Rc<SystemInfo>>>>()
        .unwrap()
        .as_ref()
        .cloned()
}

#[derive(Clone, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum WorkstationTab {
    System,
    Dependencies,
    DRymcgTech,
}

#[function_component(WorkstationTabs)]
fn tabs() -> Html {
    let system_info_context = use_context::<UseStateHandle<Option<Rc<SystemInfo>>>>();
    let selected = use_state_eq(|| {
        let location = window().location();
        let hash = location.hash().unwrap_or_default();
        WorkstationTab::from_str(hash.trim_start_matches('#')).unwrap_or(WorkstationTab::System)
    });

    let reload_trigger = use_state(|| 0);
    {
        let selected = selected.clone();
        let reload_trigger = reload_trigger.clone();
        use_effect_with((), move |_| {
            //debug!("Registering hashchange listener");
            let window = window();
            let listener = EventListener::new(&window.clone(), "hashchange", move |_event| {
                let location = window.location();
                let hash = location.hash().unwrap_or_default();
                //debug!("hashchange event triggered, current hash: {}", &hash);
                if let Ok(tab) = WorkstationTab::from_str(hash.trim_start_matches('#')) {
                    if tab == WorkstationTab::Dependencies {
                        //debug!("Dependencies tab selected, triggering reload");
                        reload_trigger.set(*reload_trigger + 1);
                    }
                    selected.set(tab);
                }
            });
            listener.forget(); // Keep the listener active

            // Cleanup when the component unmounts
            || {
                //debug!("Cleaning up hashchange listener");
            }
        });
    }

    let onselect = {
        let selected = selected.clone();
        //let reload_trigger = reload_trigger.clone();
        Callback::from(move |index: WorkstationTab| {
            window().location().set_hash(index.as_ref()).unwrap();
            if index == WorkstationTab::Dependencies {
                //reload_trigger.set(*reload_trigger + 1);
            }
            selected.set(index);
        })
    };

    if let Some(system_info) = system_info_context
        .as_ref()
        .and_then(|ctx| ctx.as_ref().map(Rc::as_ref))
    {
        html! (
            <div class="tabs">
                <Tabs<WorkstationTab> detached=true {onselect} selected={(*selected).clone()} r#box=true>
                    <Tab<WorkstationTab> index={WorkstationTab::System} title="System"/>
                    <Tab<WorkstationTab> index={WorkstationTab::Dependencies} title="Dependencies"/>
                    <Tab<WorkstationTab> index={WorkstationTab::DRymcgTech} title="d.rymcg.tech"/>
                </Tabs<WorkstationTab>>
                <section hidden={(*selected) != WorkstationTab::System}>
                <system::System system_info={(*system_info).clone()}/>
                </section>
                <section hidden={(*selected) != WorkstationTab::Dependencies} key={*reload_trigger}>
                <dependencies::DependencyList system_info={(*system_info).clone()} reload_trigger={*reload_trigger} selected_tab={(*selected).clone()} />
                </section>
                <section hidden={(*selected) != WorkstationTab::DRymcgTech} key={*reload_trigger}>
                    <install_d_rymcg_tech::InstallDRyMcGTech reload_trigger={*reload_trigger} selected_tab={(*selected).clone()} />
                </section>
            </div>
        )
    } else {
        html! {
            <div>{ "Loading system info..." }</div>
        }
    }
}

#[function_component(Workstation)]
pub fn workstation() -> Html {
    let system_info = use_state(|| None);
    html! {
        <PageSection>
            <ContextProvider<UseStateHandle<Option<Rc<SystemInfo>>>> context={system_info.clone()}>
                <SystemInfoProvider>
                    <WorkstationTabs/>
                </SystemInfoProvider>
            </ContextProvider<UseStateHandle<Option<Rc<SystemInfo>>>>>
        </PageSection>
    }
}
