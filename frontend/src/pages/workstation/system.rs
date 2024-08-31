use crate::components::ButtonLink;
use crate::pages::workstation::SystemInfo;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct SystemProps {
    pub system_info: SystemInfo,
}

#[function_component(System)]
pub fn system(props: &SystemProps) -> Html {
    html! {
        <Card>
            <CardTitle>
                {
                    html! { <>
                                  <h1>{ format!("🖖 Welcome {}", props.system_info.user.name) }</h1>
                                   </>
                    }
                }
            </CardTitle>
            <CardBody>
                {
                    {
                        let os_type = props.system_info.platform.os_type.to_string();
                        let os_term = if os_type == "Linux" {
                            "🐧 OS Type"
                        } else if os_type == "MacOS" {
                            "🍎 OS Type"
                        } else if os_type == "Windows" {
                            "🤑 OS Type"
                        } else {
                            "OS Type"
                        };

                        let variant_text = match props.system_info.platform.release.variant.as_str() {
                            "" => "".to_string(),
                            v => format!("({})", v.to_string().trim_matches('"'))
                        };
                        let can_sudo_text = if props.system_info.user.can_sudo {
                            "Yes"
                        } else {
                            "No, some packages may require manual installation"
                        };
                        html! {
                            <DescriptionList>
                                <DescriptionGroup term="🖥️ Workstation">
                                <code>{props.system_info.hostname.clone()}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term={os_term}>
                                <code>{format!("{} {}", os_type, props.system_info.platform.version.to_string())}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term="Distribution">
                                <code>{format!("{} {} {}", props.system_info.platform.release.name, props.system_info.platform.release.version, variant_text)}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term="Root privilege (sudo)">
                                <code>{can_sudo_text}</code>
                                </DescriptionGroup>
                                </DescriptionList>
                        }
                    }
                }
        </CardBody>
            <CardFooter>
            <h1>{"Next:"}</h1>
            <ButtonLink href="/workstation#dependencies">{"⭐️ Install dependencies"}</ButtonLink>
            </CardFooter>
        </Card>
    }
}
