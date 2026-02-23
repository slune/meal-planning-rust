use leptos::prelude::*;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub variant: ToastVariant,
    pub duration: Option<Duration>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
    Warning,
}

impl ToastVariant {
    pub fn icon(&self) -> &'static str {
        match self {
            ToastVariant::Success => "✓",
            ToastVariant::Error => "✕",
            ToastVariant::Info => "ℹ",
            ToastVariant::Warning => "⚠",
        }
    }

    pub fn class(&self) -> &'static str {
        match self {
            ToastVariant::Success => "toast-success",
            ToastVariant::Error => "toast-error",
            ToastVariant::Info => "toast-info",
            ToastVariant::Warning => "toast-warning",
        }
    }

    pub fn role(&self) -> &'static str {
        match self {
            ToastVariant::Error => "alert",
            _ => "status",
        }
    }
}

#[derive(Clone, Copy)]
pub struct ToastContext {
    toasts: RwSignal<Vec<Toast>>,
    next_id: RwSignal<usize>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(0),
        }
    }

    pub fn show_toast(&self, message: String, variant: ToastVariant, duration: Option<Duration>) {
        let id = self.next_id.get();
        self.next_id.update(|n| *n += 1);

        let toast = Toast {
            id,
            message,
            variant,
            duration,
        };

        self.toasts.update(|toasts| toasts.push(toast.clone()));

        // Auto-dismiss after duration
        if let Some(duration) = duration {
            let toasts = self.toasts;
            set_timeout(
                move || {
                    toasts.update(|toasts| toasts.retain(|t| t.id != id));
                },
                duration,
            );
        }
    }

    pub fn success(&self, message: impl Into<String>) {
        self.show_toast(message.into(), ToastVariant::Success, Some(Duration::from_secs(4)));
    }

    pub fn error(&self, message: impl Into<String>) {
        self.show_toast(message.into(), ToastVariant::Error, Some(Duration::from_secs(6)));
    }

    pub fn info(&self, message: impl Into<String>) {
        self.show_toast(message.into(), ToastVariant::Info, Some(Duration::from_secs(4)));
    }

    pub fn warning(&self, message: impl Into<String>) {
        self.show_toast(message.into(), ToastVariant::Warning, Some(Duration::from_secs(5)));
    }

    pub fn dismiss(&self, id: usize) {
        self.toasts.update(|toasts| toasts.retain(|t| t.id != id));
    }

    pub fn dismiss_all(&self) {
        self.toasts.update(|toasts| toasts.clear());
    }
}

#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toast_context = ToastContext::new();
    provide_context(toast_context);

    view! {
        {children()}
        <ToastContainer />
    }
}

#[component]
fn ToastContainer() -> impl IntoView {
    let toast_context = expect_context::<ToastContext>();
    let toasts = toast_context.toasts;

    view! {
        <div class="toast-container" aria-live="polite" aria-atomic="false">
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    view! {
                        <ToastItem toast=toast />
                    }
                }
            />
        </div>
    }
}

#[component]
fn ToastItem(toast: Toast) -> impl IntoView {
    let toast_context = expect_context::<ToastContext>();
    let id = toast.id;
    let message = toast.message.clone();
    let icon = toast.variant.icon();
    let class_name = toast.variant.class();
    let role = toast.variant.role();

    view! {
        <div
            class=format!("toast {}", class_name)
            role=role
            aria-live=if role == "alert" { "assertive" } else { "polite" }
        >
            <div class="toast-icon">{icon}</div>
            <div class="toast-message">{message}</div>
            <button
                class="toast-close"
                on:click=move |_| toast_context.dismiss(id)
                aria-label="Close notification"
                type="button"
            >
                "×"
            </button>
        </div>
    }
}

// Helper functions for use in components
pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}

pub fn toast_success(message: impl Into<String>) {
    use_toast().success(message);
}

pub fn toast_error(message: impl Into<String>) {
    use_toast().error(message);
}

pub fn toast_info(message: impl Into<String>) {
    use_toast().info(message);
}

pub fn toast_warning(message: impl Into<String>) {
    use_toast().warning(message);
}
