use crate::components::{ConfirmModal, icon, toast_error, toast_success};
use crate::models::Camp;
use crate::server_functions::camps::{
    create_camp, delete_camp, duplicate_camp, get_camps, update_camp,
};
use chrono::NaiveDate;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

#[component]
pub fn CampManager() -> impl IntoView {
    let navigate = use_navigate();
    let nav_stored = StoredValue::new(navigate);

    let (camps, set_camps) = signal(Vec::<Camp>::new());
    let (show_form, set_show_form) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Edit state
    let (editing_camp_id, set_editing_camp_id) = signal(None::<i64>);
    let (copying_camp, set_copying_camp) = signal(false);
    let (copy_source_camp_id, set_copy_source_camp_id) = signal(None::<i64>);
    let (copy_source_name, set_copy_source_name) = signal(String::new());
    let (copy_source_day_count, set_copy_source_day_count) = signal(None::<i64>);

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
        set_editing_camp_id.set(None);
        set_copying_camp.set(false);
        set_copy_source_camp_id.set(None);
        set_copy_source_name.set(String::new());
        set_copy_source_day_count.set(None);
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
        set_copying_camp.set(false);
        set_copy_source_camp_id.set(None);
        set_copy_source_name.set(String::new());
        set_copy_source_day_count.set(None);
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
        let is_copying = copying_camp.get();
        let source_camp_id = copy_source_camp_id.get();
        let source_name = copy_source_name.get();
        let source_day_count = copy_source_day_count.get();

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
        if start_date_val > end_date_val {
            toast_error("End date must be on or after start date");
            return;
        }

        if is_copying {
            if name_val.trim() == source_name.trim() {
                toast_error("Copied camp must use a different name");
                return;
            }

            let Ok(start) = NaiveDate::parse_from_str(&start_date_val, "%Y-%m-%d") else {
                toast_error("Invalid start date");
                return;
            };
            let Ok(end) = NaiveDate::parse_from_str(&end_date_val, "%Y-%m-%d") else {
                toast_error("Invalid end date");
                return;
            };

            if let Some(source_days) = source_day_count {
                let target_days = (end - start).num_days() + 1;
                if target_days != source_days {
                    toast_error(format!(
                        "Copied camp must stay the same length: {} days",
                        source_days
                    ));
                    return;
                }
            }

            if source_camp_id.is_none() {
                toast_error("Missing source camp for copy");
                return;
            }
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

        let editing_id = editing_camp_id.get();
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let notes_opt = if notes_val.is_empty() {
                None
            } else {
                Some(notes_val)
            };

            let result = if let Some(id) = editing_id {
                update_camp(
                    id,
                    name_val,
                    start_date_val,
                    end_date_val,
                    children_count,
                    teens_count,
                    adults_count,
                    notes_opt,
                )
                .await
                .map(|_| ())
            } else if is_copying {
                duplicate_camp(
                    source_camp_id.unwrap_or_default(),
                    name_val,
                    start_date_val,
                    end_date_val,
                    children_count,
                    teens_count,
                    adults_count,
                    notes_opt,
                )
                .await
                .map(|_| ())
            } else {
                create_camp(
                    name_val,
                    start_date_val,
                    end_date_val,
                    children_count,
                    teens_count,
                    adults_count,
                    notes_opt,
                )
                .await
                .map(|_| ())
            };

            match result {
                Ok(_) => {
                    let msg = if editing_id.is_some() {
                        "Camp updated successfully!"
                    } else if is_copying {
                        "Camp copied successfully!"
                    } else {
                        "Camp created successfully!"
                    };
                    toast_success(msg);
                    reset_form();
                    set_editing_camp_id.set(None);
                    set_copying_camp.set(false);
                    set_copy_source_camp_id.set(None);
                    set_copy_source_name.set(String::new());
                    set_copy_source_day_count.set(None);
                    set_show_form.set(false);
                    load_data();
                }
                Err(e) => {
                    let action = if editing_id.is_some() {
                        "update"
                    } else {
                        "create"
                    };
                    toast_error(format!("Failed to {} camp: {}", action, e));
                }
            }

            set_loading.set(false);
        });
    };

    // Populate form for editing
    let handle_edit_click = move |camp: Camp| {
        set_editing_camp_id.set(Some(camp.id));
        set_name.set(camp.name);
        set_start_date.set(camp.start_date.format("%Y-%m-%d").to_string());
        set_end_date.set(camp.end_date.format("%Y-%m-%d").to_string());
        set_default_children.set(camp.default_children.to_string());
        set_default_teens.set(camp.default_teens.to_string());
        set_default_adults.set(camp.default_adults.to_string());
        set_notes.set(camp.notes.unwrap_or_default());
        set_show_form.set(true);
        set_copying_camp.set(false);
        set_copy_source_camp_id.set(None);
        set_copy_source_name.set(String::new());
        set_copy_source_day_count.set(None);
        set_error.set(None);
    };

    let handle_copy_click = move |camp: Camp| {
        set_editing_camp_id.set(None);
        set_copying_camp.set(true);
        set_copy_source_camp_id.set(Some(camp.id));
        set_copy_source_name.set(camp.name.clone());
        set_copy_source_day_count.set(Some((camp.end_date - camp.start_date).num_days() + 1));
        set_name.set(format!("Copy of {}", camp.name));
        set_start_date.set(camp.start_date.format("%Y-%m-%d").to_string());
        set_end_date.set(camp.end_date.format("%Y-%m-%d").to_string());
        set_default_children.set(camp.default_children.to_string());
        set_default_teens.set(camp.default_teens.to_string());
        set_default_adults.set(camp.default_adults.to_string());
        set_notes.set(camp.notes.unwrap_or_default());
        set_show_form.set(true);
        set_error.set(None);
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
                }
                Err(e) => {
                    toast_error(format!("Failed to delete camp: {}", e));
                }
            }

            set_loading.set(false);
        });
    };

    let cancel_delete = move || {
        set_show_delete_modal.set(false);
    };

    view! {
        <div class="space-y-6">
            <div class="page-header">
                <div class="page-heading">
                    <span class="icon-badge icon-badge-sky" aria-hidden="true">{icon("camp")}</span>
                    <div>
                        <p class="page-kicker">"Setup"</p>
                        <h2 class="page-title">"Camps"</h2>
                        <p class="page-subtitle">"Create camp date ranges and default attendance counts for planning."</p>
                        <div class="mt-3">
                            <span class="status-chip status-chip-sky">{move || format!("{} camps", camps.get().len())}</span>
                        </div>
                    </div>
                </div>
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
                        set_editing_camp_id.set(None);
                        set_copying_camp.set(false);
                        set_copy_source_camp_id.set(None);
                        set_copy_source_name.set(String::new());
                        set_copy_source_day_count.set(None);
                        set_show_form.set(true);
                        set_error.set(None);
                    }
                    disabled=move || loading.get()
                >
                    {icon("plus")}
                    "Add camp"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert-error">
                    <span class="font-semibold mr-2">"Error:"</span>
                    {err}
                </div>
            })}

            {move || show_form.get().then(|| view! {
                <div class="card panel-accent panel-accent-sky">
                    <h3 class="section-title mb-6 flex items-center gap-2">
                        <span class="inline-icon text-sky-700" aria-hidden="true">
                            {move || if editing_camp_id.get().is_some() {
                                icon("edit")
                            } else if copying_camp.get() {
                                icon("copy")
                            } else {
                                icon("plus")
                            }}
                        </span>
                        {move || if editing_camp_id.get().is_some() {
                            "Edit Camp"
                        } else if copying_camp.get() {
                            "Copy Camp"
                        } else {
                            "New Camp"
                        }}
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
                                {move || {
                                    if loading.get() {
                                        "Saving..."
                                    } else if editing_camp_id.get().is_some() {
                                        "Update"
                                    } else if copying_camp.get() {
                                        "Create copy"
                                    } else {
                                        "Save"
                                    }
                                }}
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
                    <div class="empty-state">
                        <span class="icon-badge icon-badge-sky mx-auto mb-4" aria-hidden="true">{icon("camp")}</span>
                        <h3 class="text-xl font-semibold text-slate-950 mb-2">"No camps yet"</h3>
                        <p class="text-sm text-slate-600">"Create the first camp to unlock meal planning and reports."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For
                            each=move || camps.get()
                            key=|camp| camp.id
                            let:camp
                        >
                            <div class="record-card record-card-sky">
                                {
                                    let day_count = (camp.end_date - camp.start_date).num_days() + 1;
                                    view! {
                                <div class="mb-4 flex items-start justify-between gap-3">
                                    <div>
                                        <h3 class="text-xl font-bold text-slate-800 mb-1">{camp.name.clone()}</h3>
                                        <div class="flex items-center gap-2 text-sm text-slate-600">
                                            <span class="inline-icon text-sky-700" aria-hidden="true">{icon("calendar")}</span>
                                            <span>{format!("{} to {}", camp.start_date, camp.end_date)}</span>
                                        </div>
                                    </div>
                                    <span class="status-chip status-chip-sky">{format!("{} days", day_count)}</span>
                                </div>
                                    }
                                }
                                <div class="flex gap-2 flex-wrap mb-3">
                                    <span class="badge badge-primary">
                                        <span class="inline-icon mr-1" aria-hidden="true">{icon("child")}</span>
                                        {camp.default_children} " children"
                                    </span>
                                    <span class="badge badge-primary">
                                        <span class="inline-icon mr-1" aria-hidden="true">{icon("teen")}</span>
                                        {camp.default_teens} " teens"
                                    </span>
                                    <span class="badge badge-primary">
                                        <span class="inline-icon mr-1" aria-hidden="true">{icon("adult")}</span>
                                        {camp.default_adults} " adults"
                                    </span>
                                </div>
                                {camp.notes.clone().map(|n| view! {
                                    <p class="text-sm text-slate-600 mb-4 italic bg-slate-50 p-2 rounded">{n}</p>
                                })}
                                <div class="mt-auto grid grid-cols-2 gap-2">
                                    <button
                                        class="btn btn-primary col-span-2 text-sm"
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
                                        {icon("planner")}
                                        "Plan meals"
                                    </button>
                                    <button
                                        class="btn btn-secondary text-sm"
                                        on:click={
                                            let camp = camp.clone();
                                            move |_| handle_edit_click(camp.clone())
                                        }
                                        disabled=move || loading.get()
                                        aria-label="Edit camp"
                                    >
                                        {icon("edit")}
                                        "Edit"
                                    </button>
                                    <button
                                        class="btn btn-secondary text-sm"
                                        on:click={
                                            let camp = camp.clone();
                                            move |_| handle_copy_click(camp.clone())
                                        }
                                        disabled=move || loading.get()
                                        aria-label="Copy camp"
                                    >
                                        {icon("copy")}
                                        "Copy"
                                    </button>
                                    <button
                                        class="btn btn-danger col-span-2 text-sm"
                                        on:click={
                                            let id = camp.id;
                                            move |_| handle_delete_click(id)
                                        }
                                        disabled=move || loading.get()
                                        aria-label="Delete camp"
                                    >
                                        {icon("trash")}
                                        "Delete"
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
