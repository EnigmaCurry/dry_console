use patternfly_yew::prelude::*;
use std::{rc::Rc, time::Duration};

/// Use this to get toaster inside a component, nested in a ToastViewer:
///     let toast = get_toast(use_toaster().expect("Must be nested inside a ToastViewer"));
pub fn get_toast(toaster: Toaster) -> Rc<impl Fn(AlertType, &str)> {
    return Rc::new({
        let toaster = toaster.clone();
        move |t: AlertType, msg: &str| {
            toaster.toast(Toast {
                title: msg.into(),
                timeout: Some(Duration::from_secs(match t {
                    AlertType::Danger => 10,
                    _ => 5,
                })),
                r#type: t,
                ..Default::default()
            });
        }
    });
}
