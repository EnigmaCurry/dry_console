use patternfly_yew::prelude::*;
use yew::prelude::*;

#[function_component(Index)]
pub fn index() -> Html {
    html! {
        <PageSection>
            <div><p>{"Hello patternfly-yew"}</p></div>
        </PageSection>
    }
}
