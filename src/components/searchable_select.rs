use leptos::ev::{FocusEvent, KeyboardEvent};
use leptos::prelude::*;

#[component]
pub fn SearchableSelect<T, F, G, H>(
    /// List of all available options
    options: Signal<Vec<T>>,
    /// Currently selected value (ID)
    selected_value: Signal<i64>,
    /// Callback when selection changes
    on_change: F,
    /// Function to extract the ID from an option
    get_id: G,
    /// Function to extract the display text from an option
    get_display: H,
    /// Placeholder text
    #[prop(default = "Search...")]
    placeholder: &'static str,
    /// Label for accessibility
    #[prop(optional)]
    label: Option<&'static str>,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
) -> impl IntoView
where
    T: Clone + 'static + Send + Sync + PartialEq,
    F: Fn(i64) + 'static + Send + Sync + Clone,
    G: Fn(&T) -> String + 'static + Copy + Send + Sync,
    H: Fn(&T) -> String + 'static + Copy + Send + Sync,
{
    let (search_query, set_search_query) = signal(String::new());
    let (is_open, set_is_open) = signal(false);
    let (focused_index, set_focused_index) = signal(0usize);

    // Get the display text for the currently selected option
    let selected_display = Memo::new(move |_| {
        let sel_val = selected_value.get();
        options
            .get()
            .iter()
            .find(|opt| get_id(opt).parse::<i64>().unwrap_or(0) == sel_val)
            .map(|opt| get_display(opt))
            .unwrap_or_default()
    });

    // Filter options based on search query
    let filtered_options = Memo::new(move |_| {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            options.get()
        } else {
            options
                .get()
                .into_iter()
                .filter(|opt| get_display(opt).to_lowercase().contains(&query))
                .collect()
        }
    });

    let handle_select = StoredValue::new(move |item: T| {
        let id = get_id(&item).parse::<i64>().unwrap_or(0);
        on_change(id);
        set_search_query.set(String::new());
        set_is_open.set(false);
        set_focused_index.set(0);
    });

    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_search_query.set(value);
        set_is_open.set(true);
        set_focused_index.set(0);
    };

    let handle_focus = move |_: FocusEvent| {
        set_is_open.set(true);
    };

    let handle_blur = move |_: FocusEvent| {
        // Delay closing to allow click events on options
        set_timeout(
            move || set_is_open.set(false),
            std::time::Duration::from_millis(200),
        );
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        let key = ev.key();
        let filtered = filtered_options.get();

        match key.as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let current = focused_index.get();
                if current < filtered.len().saturating_sub(1) {
                    set_focused_index.set(current + 1);
                }
                set_is_open.set(true);
            }
            "ArrowUp" => {
                ev.prevent_default();
                let current = focused_index.get();
                if current > 0 {
                    set_focused_index.set(current - 1);
                }
                set_is_open.set(true);
            }
            "Enter" => {
                ev.prevent_default();
                if is_open.get() && !filtered.is_empty() {
                    let index = focused_index.get();
                    if let Some(item) = filtered.get(index) {
                        handle_select.with_value(|f| f(item.clone()));
                    }
                } else {
                    set_is_open.set(true);
                }
            }
            "Escape" => {
                ev.prevent_default();
                set_is_open.set(false);
                set_search_query.set(String::new());
            }
            _ => {}
        }
    };

    view! {
        <div class="relative">
            {label.map(|l| view! {
                <label class="form-label">
                    {l}
                    {required.then_some(view! { <span class="text-red-500">"*"</span> })}
                </label>
            })}
            <div class="relative">
                <input
                    type="text"
                    class="form-input pr-10"
                    placeholder=move || {
                        if search_query.get().is_empty() && !is_open.get() {
                            selected_display.get()
                        } else {
                            placeholder.to_string()
                        }
                    }
                    prop:value=move || search_query.get()
                    on:input=handle_input
                    on:focus=handle_focus
                    on:blur=handle_blur
                    on:keydown=handle_keydown
                />
                <div class="absolute right-3 top-1/2 transform -translate-y-1/2 pointer-events-none text-slate-400">
                    <span class="text-sm">{move || if is_open.get() { "▲" } else { "▼" }}</span>
                </div>
            </div>
            {move || is_open.get().then(|| {
                let filtered = filtered_options.get();
                view! {
                    <div class="dropdown-menu custom-scrollbar">
                        {if filtered.is_empty() {
                            view! {
                                <div class="px-4 py-3 text-slate-500 text-sm text-center">
                                    "🔍 No results found"
                                </div>
                            }.into_any()
                        } else {
                            filtered.into_iter().enumerate().map(|(idx, item)| {
                                let is_selected = get_id(&item).parse::<i64>().unwrap_or(0) == selected_value.get();
                                let is_focused = idx == focused_index.get();

                                view! {
                                    <div
                                        class=move || {
                                            let mut classes = "dropdown-item".to_string();
                                            if is_selected {
                                                classes.push_str(" dropdown-item-selected");
                                            }
                                            if is_focused && !is_selected {
                                                classes.push_str(" bg-slate-50");
                                            }
                                            classes
                                        }
                                        on:mousedown=move |ev| {
                                            ev.prevent_default();
                                            handle_select.with_value(|f| f(item.clone()));
                                        }
                                        on:mouseenter=move |_| set_focused_index.set(idx)
                                    >
                                        {if is_selected {
                                            view! { <span class="mr-2">"✓"</span> }.into_any()
                                        } else {
                                            view! { <span class="mr-2 invisible">"✓"</span> }.into_any()
                                        }}
                                        {get_display(&item)}
                                    </div>
                                }
                            }).collect_view().into_any()
                        }}
                    </div>
                }
            })}
        </div>
    }
}
