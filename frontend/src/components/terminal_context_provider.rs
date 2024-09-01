use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::context::ContextProvider;
use yew::html::ChildrenProps;
use yew::prelude::*;
use yew::virtual_dom::VNode;

const SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY: &str = "terminal:show_line_numbers";
const BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY: &str = "terminal:background_color_change";
const BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY: &str = "terminal:background_color_success";
const BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY: &str = "terminal:background_color_failure";
const BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY: &str = "terminal:background_color_normal";
const TEXT_COLOR_STDOUT_LOCALSTORAGE_KEY: &str = "terminal:text_color_stdout";
const TEXT_COLOR_STDERR_LOCALSTORAGE_KEY: &str = "terminal:text_color_stderr";
const SHOW_META_STREAM_LOCALSTORAGE_KEY: &str = "terminal:show_meta_stream";

// Define the TerminalStyleSettings struct as before
#[derive(Clone, PartialEq)]
pub struct TerminalStyleSettings {
    pub background_color_normal: UseStateHandle<String>,
    pub background_color_success: UseStateHandle<String>,
    pub background_color_failure: UseStateHandle<String>,
    pub text_color_stdout: UseStateHandle<String>,
    pub text_color_stderr: UseStateHandle<String>,
    pub show_line_numbers: UseStateHandle<bool>,
    pub show_meta_stream: UseStateHandle<bool>,
    pub background_color_change: UseStateHandle<bool>,
}

// Define the TerminalStyleContext context
#[derive(Clone, PartialEq)]
pub struct TerminalStyleContext(std::rc::Rc<TerminalStyleSettings>);
impl TerminalStyleContext {
    pub fn get_settings(&self) -> &TerminalStyleSettings {
        &self.0
    }
}

pub fn toggle_line_numbers(show_line_numbers: UseStateHandle<bool>) -> Callback<bool> {
    Callback::from(move |value: bool| {
        show_line_numbers.set(value);
        LocalStorage::set(SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY, value)
            .expect("Failed to store setting in local storage");
    })
}

pub fn toggle_meta_stream(show_meta_stream: UseStateHandle<bool>) -> Callback<bool> {
    Callback::from(move |value: bool| {
        show_meta_stream.set(value);
        LocalStorage::set(SHOW_META_STREAM_LOCALSTORAGE_KEY, value)
            .expect("Failed to store setting in local storage");
    })
}

pub fn toggle_background_change(background_color_change: UseStateHandle<bool>) -> Callback<bool> {
    Callback::from(move |value: bool| {
        background_color_change.set(value);
        LocalStorage::set(BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY, value)
            .expect("Failed to store setting in local storage");
    })
}

pub fn update_success_color(background_color_success: UseStateHandle<String>) -> Callback<String> {
    Callback::from(move |color: String| {
        background_color_success.set(color.clone());
        LocalStorage::set(BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY, color)
            .expect("Failed to store setting in local storage");
    })
}

pub fn update_failure_color(background_color_failure: UseStateHandle<String>) -> Callback<String> {
    Callback::from(move |color: String| {
        background_color_failure.set(color.clone());
        LocalStorage::set(BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY, color)
            .expect("Failed to store setting in local storage");
    })
}

pub fn update_normal_background_color(
    background_color_normal: UseStateHandle<String>,
) -> Callback<String> {
    Callback::from(move |color: String| {
        background_color_normal.set(color.clone());
        LocalStorage::set(BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY, color)
            .expect("Failed to store setting in local storage");
    })
}

pub fn update_stdout_text_color(text_color_stdout: UseStateHandle<String>) -> Callback<String> {
    Callback::from(move |color: String| {
        text_color_stdout.set(color.clone());
        LocalStorage::set(TEXT_COLOR_STDOUT_LOCALSTORAGE_KEY, color)
            .expect("Failed to store setting in local storage");
    })
}

pub fn update_stderr_text_color(text_color_stderr: UseStateHandle<String>) -> Callback<String> {
    Callback::from(move |color: String| {
        text_color_stderr.set(color.clone());
        LocalStorage::set(TEXT_COLOR_STDERR_LOCALSTORAGE_KEY, color)
            .expect("Failed to store setting in local storage");
    })
}

#[function_component(TerminalContextProvider)]
pub fn terminal_context_provider(props: &ChildrenProps) -> Html {
    let background_color_normal = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY)
            .unwrap_or("#000000".to_string())
    });

    let background_color_success = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY)
            .unwrap_or("#275346".to_string())
    });

    let background_color_failure = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY)
            .unwrap_or("#712121".to_string())
    });

    let text_color_stdout = use_state(|| {
        LocalStorage::get::<String>(TEXT_COLOR_STDOUT_LOCALSTORAGE_KEY)
            .unwrap_or("#ffffff".to_string())
    });

    let text_color_stderr = use_state(|| {
        LocalStorage::get::<String>(TEXT_COLOR_STDERR_LOCALSTORAGE_KEY)
            .unwrap_or("#dc8add".to_string())
    });

    let show_line_numbers = use_state(|| {
        LocalStorage::get::<bool>(SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY).unwrap_or(false)
    });

    let show_meta_stream =
        use_state(|| LocalStorage::get::<bool>(SHOW_META_STREAM_LOCALSTORAGE_KEY).unwrap_or(true));

    let background_color_change = use_state(|| {
        LocalStorage::get::<bool>(BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY).unwrap_or(true)
    });

    let style_settings = TerminalStyleSettings {
        background_color_normal,
        background_color_success,
        background_color_failure,
        text_color_stdout,
        text_color_stderr,
        show_line_numbers,
        show_meta_stream,
        background_color_change,
    };

    let style_ctx = TerminalStyleContext(Rc::new(style_settings));

    html! {
        <ContextProvider<TerminalStyleContext> context={style_ctx}>
            {
                match &props.children {
                    VNode::VList(vlist) => html! { for vlist.iter().cloned() },
                    _ => props.children.clone(),
                }
            }
        </ContextProvider<TerminalStyleContext>>
    }
}
