use crate::models::Camp;
use crate::server_functions::camps::{get_camps, create_camp, delete_camp};
use crate::components::{ConfirmModal, toast_success, toast_error};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::ev::SubmitEvent;
use leptos_router::hooks::use_navigate;

#[component]
pub fn CampManager() -> impl IntoView {
    let navigate = use_navigate();
    let nav_stored = StoredValue::new(navigate);
    
    let (camps, set_camps) = signal(Vec::<Camp>::new());
    let (show_form, set_show_form) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Modal state
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (delete_id, set_delete_id) = signal(0i64);

    // Form fields
    let (name, set_name) = signal(String::new());
    let (start_date, set_start_date) = signal(String::new());
    let (end_date, set_end_date) = signal(String::new());
    let (default_children, set_default_children) = signal(String::from("0"));
    let (default_teens, set_default_teens) = signal(String::from("0"));
    let (default_adults, set_default_adults) = signal(String::from("0"));
    let (notes, set_notes) = signal(String::new());

    // Load data on mount
    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            match get_camps().await {
                Ok(data) => set_camps.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load camps: {}", e))),
            }
            
            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_data();
    });

    let cancel_form = move |_| {
        set_show_form.set(false);
        set_error.set(None);
    };

    // Reset form fields
    let reset_form = move || {
        set_name.set(String::new());
        set_start_date.set(String::new());
        set_end_date.set(String::new());
        set_default_children.set(String::from("0"));
        set_default_teens.set(String::from("0"));
        set_default_adults.set(String::from("0"));
        set_notes.set(String::new());
        set_error.set(None);
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        
        let name_val = name.get();
        let start_date_val = start_date.get();
        let end_date_val = end_date.get();
        let default_children_val = default_children.get();
        let default_teens_val = default_teens.get();
        let default_adults_val = default_adults.get();
        let notes_val = notes.get();
        
        if name_val.is_empty() || start_date_val.is_empty() || end_date_val.is_empty() {
            toast_error("Please fill in all required fields");
            return;
        }

        let children_count = match default_children_val.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                toast_error("Invalid number for children");
                return;
            }
        };

        let teens_count = match default_teens_val.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                toast_error("Invalid number for teens");
                return;
            }
        };

        let adults_count = match default_adults_val.parse::<i32>() {
            Ok(val) => val,
            Err(_) => {
                toast_error("Invalid number for adults");
                return;
            }
        };

        // Validate date range
        if start_date_val >= end_date_val {
            toast_error("End date must be after start date");
            return;
        }

        // Validate counts are non-negative
        if children_count < 0 {
            toast_error("Number of children cannot be negative");
            return;
        }
        if teens_count < 0 {
            toast_error("Number of teens cannot be negative");
            return;
        }
        if adults_count < 0 {
            toast_error("Number of adults cannot be negative");
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            let notes_opt = if notes_val.is_empty() { None } else { Some(notes_val) };
            
            match create_camp(
                name_val,
                start_date_val,
                end_date_val,
                children_count,
                teens_count,
                adults_count,
                notes_opt,
            ).await {
                Ok(_) => {
                    toast_success("Camp created successfully!");
                    reset_form();
                    set_show_form.set(false);
                    load_data();
                },
                Err(e) => toast_error(&format!("Failed to create camp: {}", e)),
            }
            
            set_loading.set(false);
        });
    };

    // Trigger delete modal
    let handle_delete_click = move |id: i64| {
        set_delete_id.set(id);
        set_show_delete_modal.set(true);
    };

    // Confirm delete action
    let confirm_delete = move || {
        let id = delete_id.get();
        set_show_delete_modal.set(false);

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match delete_camp(id).await {
                Ok(_) => {
                    toast_success("Camp deleted successfully!");
                    load_data();
                },
                Err(e) => {
                    toast_error(&format!("Failed to delete camp: {}", e));
                },
            }

            set_loading.set(false);
        });
    };

    let cancel_delete = move || {
        set_show_delete_modal.set(false);
    };

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h2 class="text-3xl font-bold text-gradient flex items-center gap-3">
                    <span class="text-4xl">"🏕️"</span>
                    "Camps"
                </h2>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=move |_| {
                        set_name.set(String::new());
                        set_start_date.set(String::new());
                        set_end_date.set(String::new());
                        set_default_children.set(String::from("0"));
                        set_default_teens.set(String::from("0"));
                        set_default_adults.set(String::from("0"));
                        set_notes.set(String::new());
                        set_show_form.set(true);
                        set_error.set(None);
                    }
                    disabled=move || loading.get()
                >
                    <span class="mr-1">"+"</span>
                    " Add Camp"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert-error">
                    <span class="font-semibold mr-2">"⚠️ Error:"</span>
                    {err}
                </div>
            })}

            {move || show_form.get().then(|| view! {
                <div class="card border-2 border-blue-200">
                    <h3 class="text-2xl font-bold mb-6 text-gradient flex items-center gap-2">
                        <span>"✨"</span>
                        "New Camp"
                    </h3>
                    <form on:submit=handle_submit class="space-y-4">
                        <div>
                            <label for="camp-name" class="form-label">"Camp Name" <span class="text-red-500">"*"</span></label>
                            <input
                                id="camp-name"
                                type="text"
                                class="form-input"
                                prop:value=move || name.get()
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                placeholder="e.g., Summer Camp 2026"
                                required
                                aria-required="true"
                            />
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label for="camp-start-date" class="form-label">"Start Date" <span class="text-red-500">"*"</span></label>
                                <input
                                    id="camp-start-date"
                                    type="date"
                                    class="form-input"
                                    prop:value=move || start_date.get()
                                    on:input=move |ev| set_start_date.set(event_target_value(&ev))
                                    required
                                    aria-required="true"
                                />
                            </div>
                            <div>
                                <label for="camp-end-date" class="form-label">"End Date" <span class="text-red-500">"*"</span></label>
                                <input
                                    id="camp-end-date"
                                    type="date"
                                    class="form-input"
                                    prop:value=move || end_date.get()
                                    on:input=move |ev| set_end_date.set(event_target_value(&ev))
                                    required
                                    aria-required="true"
                                />
                            </div>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div>
                                <label for="camp-children" class="form-label">"Default Children"</label>
                                <input
                                    id="camp-children"
                                    type="number"
                                    class="form-input"
                                    prop:value=move || default_children.get()
                                    on:input=move |ev| set_default_children.set(event_target_value(&ev))
                                    min="0"
                                    aria-label="Default number of children"
                                />
                            </div>
                            <div>
                                <label for="camp-teens" class="form-label">"Default Teens"</label>
                                <input
                                    id="camp-teens"
                                    type="number"
                                    class="form-input"
                                    prop:value=move || default_teens.get()
                                    on:input=move |ev| set_default_teens.set(event_target_value(&ev))
                                    min="0"
                                    aria-label="Default number of teens"
                                />
                            </div>
                            <div>
                                <label for="camp-adults" class="form-label">"Default Adults"</label>
                                <input
                                    id="camp-adults"
                                    type="number"
                                    class="form-input"
                                    prop:value=move || default_adults.get()
                                    on:input=move |ev| set_default_adults.set(event_target_value(&ev))
                                    min="0"
                                    aria-label="Default number of adults"
                                />
                            </div>
                        </div>
                        <div>
                            <label for="camp-notes" class="form-label">"Notes (optional)"</label>
                            <textarea
                                id="camp-notes"
                                class="form-input"
                                prop:value=move || notes.get()
                                on:input=move |ev| set_notes.set(event_target_value(&ev))
                                placeholder="Any additional notes about this camp"
                                rows="3"
                                aria-label="Camp notes"
                            />
                        </div>
                        <div class="flex gap-2">
                            <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                                {move || if loading.get() { "Saving..." } else { "Save" }}
                            </button>
                            <button type="button" class="btn btn-secondary" on:click=cancel_form disabled=move || loading.get()>
                                "Cancel"
                            </button>
                        </div>
                    </form>
                </div>
            })}

            {move || if loading.get() && !show_form.get() {
                view! {
                    <div class="card text-center py-12">
                        <div class="spinner mx-auto mb-4" role="status" aria-label="Loading camps"></div>
                        <p class="text-slate-600" aria-live="polite">"Loading camps..."</p>
                    </div>
                }.into_any()
            } else if camps.get().is_empty() {
                view! { 
                    <div class="card text-center py-16 bg-gradient-to-br from-slate-50 to-blue-50 border-2 border-dashed border-slate-300">
                        <div class="text-7xl mb-6">"🏕️"</div>
                        <h3 class="text-2xl font-bold text-slate-800 mb-3">"No camps yet"</h3>
                        <p class="text-lg text-slate-600 mb-8">"Get started by creating your first camp"</p>
                    </div> 
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <For
                            each=move || camps.get()
                            key=|camp| camp.id
                            let:camp
                        >
                            <div class="card group">
                                <div class="flex items-start justify-between mb-3">
                                    <div class="text-4xl group-hover:scale-110 transition-transform duration-200">"🏕️"</div>
                                </div>
                                <h3 class="text-xl font-bold text-slate-800 mb-2">{camp.name.clone()}</h3>
                                <div class="flex items-center gap-2 text-sm text-slate-600 mb-3">
                                    <span>"📅"</span>
                                    <span>{format!("{} to {}", camp.start_date, camp.end_date)}</span>
                                </div>
                                <div class="flex gap-2 flex-wrap mb-3">
                                    <span class="badge badge-primary">
                                        "👶 " {camp.default_children} " children"
                                    </span>
                                    <span class="badge badge-primary">
                                        "🧒 " {camp.default_teens} " teens"
                                    </span>
                                    <span class="badge badge-primary">
                                        "👨 " {camp.default_adults} " adults"
                                    </span>
                                </div>
                                {camp.notes.clone().map(|n| view! {
                                    <p class="text-sm text-slate-600 mb-4 italic bg-slate-50 p-2 rounded">{n}</p>
                                })}
                                <div class="mt-auto flex gap-2">
                                    <button
                                        class="btn btn-primary text-sm flex-1"
                                        on:click={
                                            let id = camp.id;
                                            move |_| {
                                                nav_stored.with_value(|nav| {
                                                    nav(&format!("/planner/{}", id), Default::default());
                                                });
                                            }
                                        }
                                        disabled=move || loading.get()
                                    >
                                        "📅 Plan Meals"
                                    </button>
                                    <button
                                        class="btn btn-danger text-sm"
                                        on:click={
                                            let id = camp.id;
                                            move |_| handle_delete_click(id)
                                        }
                                        disabled=move || loading.get()
                                        aria-label="Delete camp"
                                    >
                                        "🗑️"
                                    </button>
                                </div>
                            </div>
                        </For>
                    </div>
                }.into_any()
            }}

            // Delete Confirmation Modal
            <ConfirmModal
                show=show_delete_modal.into()
                on_confirm=confirm_delete
                on_cancel=cancel_delete
                title="Delete Camp".to_string()
                message="Are you sure you want to delete this camp? This action cannot be undone.".to_string()
                confirm_text="Delete".to_string()
                cancel_text="Cancel".to_string()
                variant="danger".to_string()
            />
        </div>
    }
}
