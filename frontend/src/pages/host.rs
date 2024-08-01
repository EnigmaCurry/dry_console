use patternfly_yew::prelude::*;
use yew::prelude::*;

#[function_component(Host)]
pub fn host() -> Html {
    html! {
        <PageSection>
            <div><p>{"Host"}</p></div>
        </PageSection>
    }
}
