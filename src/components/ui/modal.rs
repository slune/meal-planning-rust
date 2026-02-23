use leptos::prelude::*;
use leptos::ev::{MouseEvent, KeyboardEvent};

#[component]
pub fn ConfirmModal(
    /// Whether the modal is visible
    show: Signal<bool>,
    /// Callback when user confirms
    on_confirm: impl Fn() + 'static + Clone + Send + Sync,
    /// Callback when user cancels
    on_cancel: impl Fn() + 'static + Clone + Send + Sync,
    /// Modal title
    #[prop(default = "Confirm Action".to_string())]
    title: String,
    /// Modal message/body
    #[prop(default = "Are you sure you want to proceed?".to_string())]
    message: String,
    /// Text for confirm button
    #[prop(default = "Confirm".to_string())]
    confirm_text: String,
    /// Text for cancel button
    #[prop(default = "Cancel".to_string())]
    cancel_text: String,
    /// Button variant: primary, danger
    #[prop(default = "primary".to_string())]
    variant: String,
) -> impl IntoView {
    let on_confirm = StoredValue::new(on_confirm);
    let on_cancel = StoredValue::new(on_cancel);

    // Handle backdrop click
    let handle_backdrop_click = move |_ev: MouseEvent| {
        // Close modal when backdrop is clicked (handled by the outer div click)
        on_cancel.with_value(|f| f());
    };

    // Handle ESC key press
    let handle_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Escape" {
            on_cancel.with_value(|f| f());
        }
    };

    let button_class = if variant == "danger" {
        "btn btn-danger"
    } else {
        "btn btn-primary"
    };
    let title = title.clone();
    let message = message.clone();
    let confirm_text = confirm_text.clone();
    let cancel_text = cancel_text.clone();

    view! {
        <Show
            when=move || show.get()
            fallback=|| ()
        >
            <div
                class="modal-backdrop"
                on:click=handle_backdrop_click
                on:keydown=handle_keydown
                role="dialog"
                aria-modal="true"
                aria-labelledby="modal-title"
                tabindex="-1"
            >
                <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                    <div class="modal-header">
                        <h2 id="modal-title" class="modal-title">{title.clone()}</h2>
                    </div>
                    <div class="modal-body">
                        <p>{message.clone()}</p>
                    </div>
                    <div class="modal-footer">
                        <button
                            class="btn btn-secondary"
                            on:click=move |_| on_cancel.with_value(|f| f())
                            type="button"
                        >
                            {cancel_text.clone()}
                        </button>
                        <button
                            class={button_class}
                            on:click=move |_| on_confirm.with_value(|f| f())
                            type="button"
                        >
                            {confirm_text.clone()}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn AlertModal(
    /// Whether the modal is visible
    show: Signal<bool>,
    /// Callback when user closes
    on_close: impl Fn() + 'static + Clone + Send + Sync,
    /// Modal title
    #[prop(default = "Alert".to_string())]
    title: String,
    /// Modal message/body
    message: String,
    /// Text for close button
    #[prop(default = "OK".to_string())]
    close_text: String,
) -> impl IntoView {
    let on_close = StoredValue::new(on_close);
    let title = title.clone();
    let message = message.clone();
    let close_text = close_text.clone();

    // Handle ESC key press
    let handle_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Escape" {
            on_close.with_value(|f| f());
        }
    };

    view! {
        <Show
            when=move || show.get()
            fallback=|| ()
        >
            <div
                class="modal-backdrop"
                on:click=move |_| on_close.with_value(|f| f())
                on:keydown=handle_keydown
                role="alertdialog"
                aria-modal="true"
                aria-labelledby="alert-modal-title"
                tabindex="-1"
            >
                <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                    <div class="modal-header">
                        <h2 id="alert-modal-title" class="modal-title">{title.clone()}</h2>
                    </div>
                    <div class="modal-body">
                        <p>{message.clone()}</p>
                    </div>
                    <div class="modal-footer">
                        <button
                            class="btn btn-primary"
                            on:click=move |_| on_close.with_value(|f| f())
                            type="button"
                        >
                            {close_text.clone()}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
