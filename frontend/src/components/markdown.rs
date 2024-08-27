use gloo::console::info;
use patternfly_yew::prelude::*;
use pulldown_cmark::{html, Parser};
use web_sys::Element;
use yew::prelude::*;
#[derive(Properties, PartialEq, Clone)]
pub struct MarkdownContentProps {
    pub source: String,
}

#[function_component(MarkdownContent)]
pub fn markdown_content(props: &MarkdownContentProps) -> Html {
    let div_ref = use_node_ref();

    {
        let source = props.source.clone();
        let html_content = markdown_to_html(&source);
        let div_ref = div_ref.clone();

        use_effect_with((), move |_| {
            if let Some(div) = div_ref.cast::<Element>() {
                div.set_inner_html(&html_content);
            }
            || ()
        });
    }

    html! {
        <div class="markdown_render" ref={div_ref} />
    }
}

// Convert Markdown to HTML string
pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
