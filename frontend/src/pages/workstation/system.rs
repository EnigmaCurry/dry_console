use patternfly_yew::prelude::*;
use yew::prelude::*;

#[function_component(System)]
pub fn system() -> Html {
    html! {
    <Card>
            <CardTitle><h1>{"🖖 Welcome"}</h1></CardTitle>
            <CardBody>
            {"Todo items:"}
            <ol>
            <li><a href="/workstation#dependencies">{"Install dependencies"}</a></li>
            </ol>
            </CardBody>
            </Card>
    }
}
