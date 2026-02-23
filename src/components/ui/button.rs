use leptos::prelude::*;
use leptos::ev::MouseEvent;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "btn-primary",
            ButtonVariant::Secondary => "btn-secondary",
            ButtonVariant::Danger => "btn-danger",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonSize::Small => "btn-sm",
            ButtonSize::Medium => "",
            ButtonSize::Large => "btn-lg",
        }
    }
}

#[component]
pub fn Button(
    /// Button click handler
    #[prop(optional)]
    on_click: Option<impl Fn(MouseEvent) + 'static + Clone + Send + Sync>,
    /// Button variant (primary, secondary, danger)
    #[prop(default = ButtonVariant::Primary)]
    variant: ButtonVariant,
    /// Button size
    #[prop(default = ButtonSize::Medium)]
    size: ButtonSize,
    /// Whether the button is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the button is in loading state
    #[prop(default = false)]
    loading: bool,
    /// Button type attribute
    #[prop(default = "button".to_string())]
    button_type: String,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
    /// ARIA label for accessibility
    #[prop(optional)]
    aria_label: Option<String>,
    /// Button content
    children: Children,
) -> impl IntoView {
    let on_click = StoredValue::new(on_click);

    let handle_click = move |ev: MouseEvent| {
        if !disabled && !loading {
            on_click.with_value(|handler_opt| {
                if let Some(handler) = handler_opt {
                    handler(ev);
                }
            });
        }
    };

    let button_classes = move || {
        format!(
            "btn {} {} {}",
            variant.class(),
            size.class(),
            class
        )
    };

    view! {
        <button
            type=button_type
            class=button_classes
            disabled=move || disabled || loading
            on:click=handle_click
            aria-busy=move || loading
            aria-label=aria_label
        >
            {children()}
        </button>
    }
}

// Note: If you want a loading spinner inside the button, pass it as children:
// <Button loading=true>
//     <Spinner size="small".to_string() /> "Loading..."
// </Button>

// IconButton is just a convenience wrapper around Button with btn-icon class
// Users can also directly use: <Button class="btn-icon" aria_label="...">...</Button>
