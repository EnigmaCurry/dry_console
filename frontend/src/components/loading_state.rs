use patternfly_yew::prelude::*;
use yew::prelude::*;

#[function_component(LoadingState)]
pub fn loading_state() -> Html {
    html! {
        <Card>
            <CardTitle><p><h1>{"⌛️ Loading ..."}</h1></p></CardTitle>
            <CardBody>
                <div class="flex-center">
                    <Spinner size={SpinnerSize::Custom(String::from("80px"))} aria_label="Contents of the custom size example" />
                </div>
            </CardBody>
        </Card>
    }
}
