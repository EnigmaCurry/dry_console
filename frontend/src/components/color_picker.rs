use gloo::console::debug;
use js_sys::{Function, Object, Reflect};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ColorPickerProps {
    pub onchange: Callback<String>,
    pub color: String,
}

#[function_component(ColorPicker)]
pub fn color_picker(ColorPickerProps { color, onchange }: &ColorPickerProps) -> Html {
    let parent_ref = use_node_ref();

    let parent_ref_clone = parent_ref.clone();
    let color = color.clone();
    let onchange = onchange.clone();

    use_effect_with(parent_ref.clone(), move |_| {
        let parent = parent_ref_clone
            .cast::<Element>()
            .expect("Failed to cast NodeRef to Element");
        let parent = Rc::new(RefCell::new(parent));

        // Set the initial background color on the div
        parent
            .borrow()
            .set_attribute("style", &format!("background: {}", color))
            .expect("Failed to set initial background color");

        let window = web_sys::window().expect("no global `window` exists");

        let closure = Closure::wrap(Box::new(move || {
            let options = Object::new();

            Reflect::set(
                &options,
                &JsValue::from_str("parent"),
                &parent.borrow().clone().into(),
            )
            .expect("Failed to set parent option");

            Reflect::set(
                &options,
                &JsValue::from_str("popup"),
                &JsValue::from_str("false"),
            )
            .expect("Failed to set popup option");

            // Pass the initial color prop to the picker
            Reflect::set(
                &options,
                &JsValue::from_str("color"),
                &JsValue::from_str(&color),
            )
            .expect("Failed to set color option");

            let picker_class = Reflect::get(&window, &"Picker".into())
                .expect("Picker is not defined")
                .dyn_into::<Function>()
                .expect("Picker is not a class");

            let picker_instance = Reflect::construct(&picker_class, &js_sys::Array::of1(&options))
                .expect("Failed to create Picker instance");

            let on_change = {
                let parent_inner = Rc::clone(&parent);
                let onchange = onchange.clone();
                Closure::wrap(Box::new(move |color: JsValue| {
                    let color_string = Reflect::get(&color, &"hex".into())
                        .expect("Failed to get hex")
                        .as_string()
                        .expect("Failed to convert hex to String");

                    parent_inner
                        .borrow()
                        .set_attribute("style", &format!("background: {}", color_string))
                        .expect("Failed to set background color");

                    onchange.emit(color_string);
                }) as Box<dyn FnMut(JsValue)>)
            };

            Reflect::set(&picker_instance, &"onChange".into(), on_change.as_ref())
                .expect("Failed to set onChange callback");

            on_change.forget();
        }) as Box<dyn Fn()>);

        let function = closure.as_ref().unchecked_ref::<Function>();
        function
            .call0(&JsValue::NULL)
            .expect("Failed to execute closure");

        closure.forget();

        || ()
    });

    html! {
        <div ref={parent_ref} class="color_picker"></div>
    }
}
