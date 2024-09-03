use gloo::utils::document;
use pulldown_cmark::{html, Parser};
use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*; // Import this to use `dyn_ref`

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

                // JavaScript part: select all <a> tags inside .markdown_render and set target="_blank"
                let document = document();
                let links = document.query_selector_all(".markdown_render a").unwrap();

                for i in 0..links.length() {
                    if let Some(link) = links.item(i) {
                        if let Some(element) = link.dyn_ref::<Element>() {
                            element.set_attribute("target", "_blank").unwrap();
                        }
                    }
                }
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
