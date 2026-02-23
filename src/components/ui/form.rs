use leptos::prelude::*;

#[component]
pub fn FormField(
    /// Label text
    label: String,
    /// Input field ID
    #[prop(optional)]
    id: Option<String>,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
    /// Error message to display
    #[prop(optional)]
    error: Option<String>,
    /// Help text to display below the input
    #[prop(optional)]
    help_text: Option<String>,
    /// Additional CSS classes for the container
    #[prop(default = String::new())]
    class: String,
    /// Input element
    children: Children,
) -> impl IntoView {
    let field_id = id.unwrap_or_else(|| {
        label.to_lowercase().replace(" ", "-")
    });
    let error_id = format!("{}-error", field_id);
    let help_id = format!("{}-help", field_id);

    view! {
        <div class=format!("form-field {}", class)>
            <label for=field_id.clone() class="form-label">
                {label.clone()}
                {required.then(|| view! {
                    <span class="text-red-500 ml-1" aria-label="required">"*"</span>
                })}
            </label>
            {children()}
            {help_text.clone().map(|text| view! {
                <p id=help_id.clone() class="form-help-text text-sm text-slate-600 mt-1">
                    {text}
                </p>
            })}
            {error.clone().map(|err| view! {
                <div id=error_id.clone() class="form-error text-red-600 text-sm mt-1" role="alert">
                    {err}
                </div>
            })}
        </div>
    }
}

#[component]
pub fn Input(
    /// Input type (text, number, email, etc.)
    #[prop(default = "text".to_string())]
    input_type: String,
    /// Input value signal
    value: Signal<String>,
    /// Input change handler
    on_input: impl Fn(String) + 'static + Clone + Send + Sync,
    /// Input ID
    #[prop(optional)]
    id: Option<String>,
    /// Placeholder text
    #[prop(default = String::new())]
    placeholder: String,
    /// Whether the input is required
    #[prop(default = false)]
    required: bool,
    /// Whether the input is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the input has an error
    #[prop(default = false)]
    has_error: bool,
    /// Minimum value (for number inputs)
    #[prop(optional)]
    min: Option<String>,
    /// Maximum value (for number inputs)
    #[prop(optional)]
    max: Option<String>,
    /// Step value (for number inputs)
    #[prop(optional)]
    step: Option<String>,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
    /// ARIA described by (for error messages)
    #[prop(optional)]
    aria_describedby: Option<String>,
) -> impl IntoView {
    let on_input = StoredValue::new(on_input);

    view! {
        <input
            type=input_type
            id=id
            class=format!("form-input {}", class)
            value=move || value.get()
            on:input=move |ev| on_input.with_value(|f| f(event_target_value(&ev)))
            placeholder=placeholder
            required=required
            disabled=disabled
            aria-required=move || required
            aria-invalid=move || has_error
            aria-describedby=aria_describedby
            min=min
            max=max
            step=step
        />
    }
}

#[component]
pub fn TextArea(
    /// Textarea value signal
    value: Signal<String>,
    /// Textarea change handler
    on_input: impl Fn(String) + 'static + Clone + Send + Sync,
    /// Textarea ID
    #[prop(optional)]
    id: Option<String>,
    /// Placeholder text
    #[prop(default = String::new())]
    placeholder: String,
    /// Number of rows
    #[prop(default = 4)]
    rows: usize,
    /// Whether the textarea is required
    #[prop(default = false)]
    required: bool,
    /// Whether the textarea is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the textarea has an error
    #[prop(default = false)]
    has_error: bool,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
    /// ARIA described by (for error messages)
    #[prop(optional)]
    aria_describedby: Option<String>,
) -> impl IntoView {
    let on_input = StoredValue::new(on_input);

    view! {
        <textarea
            id=id
            class=format!("form-input {}", class)
            rows=rows
            on:input=move |ev| on_input.with_value(|f| f(event_target_value(&ev)))
            placeholder=placeholder
            required=required
            disabled=disabled
            aria-required=move || required
            aria-invalid=move || has_error
            aria-describedby=aria_describedby
        >
            {move || value.get()}
        </textarea>
    }
}

#[component]
pub fn Select(
    /// Select value signal
    value: Signal<String>,
    /// Select change handler
    on_change: impl Fn(String) + 'static + Clone + Send + Sync,
    /// Select ID
    #[prop(optional)]
    id: Option<String>,
    /// Whether the select is required
    #[prop(default = false)]
    required: bool,
    /// Whether the select is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the select has an error
    #[prop(default = false)]
    has_error: bool,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
    /// ARIA described by (for error messages)
    #[prop(optional)]
    aria_describedby: Option<String>,
    /// Select options
    children: Children,
) -> impl IntoView {
    let on_change = StoredValue::new(on_change);

    view! {
        <select
            id=id
            class=format!("form-input {}", class)
            on:change=move |ev| on_change.with_value(|f| f(event_target_value(&ev)))
            required=required
            disabled=disabled
            aria-required=move || required
            aria-invalid=move || has_error
            aria-describedby=aria_describedby
        >
            {children()}
        </select>
    }
}

#[component]
pub fn Checkbox(
    /// Checkbox checked signal
    checked: Signal<bool>,
    /// Checkbox change handler
    on_change: impl Fn(bool) + 'static + Clone + Send + Sync,
    /// Checkbox ID
    #[prop(optional)]
    id: Option<String>,
    /// Label text
    label: String,
    /// Whether the checkbox is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
) -> impl IntoView {
    let on_change = StoredValue::new(on_change);
    let checkbox_id = id.unwrap_or_else(|| {
        label.to_lowercase().replace(" ", "-")
    });

    view! {
        <div class=format!("flex items-center gap-2 {}", class)>
            <input
                type="checkbox"
                id=checkbox_id.clone()
                checked=move || checked.get()
                on:change=move |ev| on_change.with_value(|f| f(event_target_checked(&ev)))
                disabled=disabled
                class="form-checkbox"
            />
            <label for=checkbox_id class="text-slate-700 cursor-pointer select-none">
                {label}
            </label>
        </div>
    }
}

// Validation helpers
pub fn validate_required(value: &str, field_name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} is required", field_name))
    } else {
        Ok(())
    }
}

pub fn validate_email(email: &str) -> Result<(), String> {
    if email.contains('@') && email.contains('.') {
        Ok(())
    } else {
        Err("Please enter a valid email address".to_string())
    }
}

pub fn validate_min_length(value: &str, min: usize, field_name: &str) -> Result<(), String> {
    if value.len() < min {
        Err(format!("{} must be at least {} characters", field_name, min))
    } else {
        Ok(())
    }
}

pub fn validate_number_range(value: i32, min: i32, max: i32, field_name: &str) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{} must be between {} and {}", field_name, min, max))
    } else {
        Ok(())
    }
}

pub fn validate_positive_number(value: i32, field_name: &str) -> Result<(), String> {
    if value < 0 {
        Err(format!("{} must be a positive number", field_name))
    } else {
        Ok(())
    }
}
