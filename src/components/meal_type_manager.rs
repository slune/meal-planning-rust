use crate::components::{icon, toast_error, toast_success};
use crate::models::MealType;
use crate::server_functions::meal_plans::{create_meal_type, get_meal_types, update_meal_type};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn MealTypeManager() -> impl IntoView {
    let (meal_types, set_meal_types) = signal(Vec::<MealType>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let (new_name, set_new_name) = signal(String::new());
    let (new_order, set_new_order) = signal(String::new());
    let (editing_id, set_editing_id) = signal(None::<i64>);
    let (editing_name, set_editing_name) = signal(String::new());
    let (editing_order, set_editing_order) = signal(String::new());

    let load_meal_types = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match get_meal_types().await {
                Ok(data) => set_meal_types.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load meal types: {}", e))),
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_meal_types();
    });

    let clear_edit = move || {
        set_editing_id.set(None);
        set_editing_name.set(String::new());
        set_editing_order.set(String::new());
    };

    let add_meal_type = move |_| {
        let name = new_name.get();
        let order = new_order.get();
        let sort_order = if order.trim().is_empty() {
            None
        } else {
            match order.trim().parse::<i32>() {
                Ok(value) => Some(value),
                Err(_) => {
                    toast_error("Meal type order must be a whole number");
                    return;
                }
            }
        };

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match create_meal_type(name, sort_order).await {
                Ok(_) => {
                    toast_success("Meal type added successfully!");
                    set_new_name.set(String::new());
                    set_new_order.set(String::new());
                    load_meal_types();
                }
                Err(e) => toast_error(format!("Failed to add meal type: {}", e)),
            }

            set_loading.set(false);
        });
    };

    let start_edit = move |meal_type: MealType| {
        set_editing_id.set(Some(meal_type.id));
        set_editing_name.set(meal_type.name);
        set_editing_order.set(meal_type.sort_order.to_string());
    };

    let cancel_edit = move |_| {
        clear_edit();
    };

    let save_meal_type = move |_| {
        let Some(id) = editing_id.get() else {
            return;
        };
        let name = editing_name.get();
        let sort_order = match editing_order.get().trim().parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                toast_error("Meal type order must be a whole number");
                return;
            }
        };

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match update_meal_type(id, name, sort_order).await {
                Ok(_) => {
                    toast_success("Meal type updated successfully!");
                    clear_edit();
                    load_meal_types();
                }
                Err(e) => toast_error(format!("Failed to update meal type: {}", e)),
            }

            set_loading.set(false);
        });
    };

    view! {
        <div class="card panel-accent panel-accent-sky">
            <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between gap-3">
                    <h3 class="section-title flex items-center gap-2">
                        <span class="inline-icon text-sky-700" aria-hidden="true">{icon("planner")}</span>
                        "Meal Types"
                    </h3>
                    <span class="status-chip status-chip-sky">
                        {move || format!("{} types", meal_types.get().len())}
                    </span>
                </div>

                {move || error.get().map(|err| view! {
                    <div class="alert-error">{err}</div>
                })}

                <div class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,1fr)_7rem_auto] md:items-end">
                    <div>
                        <label class="form-label">"Name"</label>
                        <input
                            type="text"
                            class="form-input"
                            prop:value=move || new_name.get()
                            on:input=move |ev| set_new_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="form-label">"Order"</label>
                        <input
                            type="number"
                            class="form-input"
                            prop:value=move || new_order.get()
                            on:input=move |ev| set_new_order.set(event_target_value(&ev))
                            min="1"
                        />
                    </div>
                    <button
                        type="button"
                        class="btn btn-secondary"
                        on:click=add_meal_type
                        disabled=move || loading.get()
                    >
                        {icon("plus")}
                        "Add type"
                    </button>
                </div>

                <div class="overflow-x-auto">
                    <div class="grid min-w-[34rem] grid-cols-[5rem_minmax(0,1fr)_12rem] gap-2 px-2 pb-1 text-xs font-semibold uppercase tracking-wide text-slate-500">
                        <span>"Order"</span>
                        <span>"Name"</span>
                        <span></span>
                    </div>
                    <div class="space-y-1">
                        <For
                            each=move || meal_types.get()
                            key=|meal_type| meal_type.id
                            let:meal_type
                        >
                            {move || {
                                let is_editing = editing_id.get() == Some(meal_type.id);
                                if is_editing {
                                    view! {
                                        <div class="grid min-w-[34rem] grid-cols-[5rem_minmax(0,1fr)_12rem] items-center gap-2 rounded-md bg-slate-50 px-2 py-1.5">
                                            <input
                                                type="number"
                                                class="form-input text-sm"
                                                prop:value=move || editing_order.get()
                                                on:input=move |ev| set_editing_order.set(event_target_value(&ev))
                                                min="1"
                                            />
                                            <input
                                                type="text"
                                                class="form-input text-sm"
                                                prop:value=move || editing_name.get()
                                                on:input=move |ev| set_editing_name.set(event_target_value(&ev))
                                            />
                                            <div class="flex justify-end gap-2">
                                                <button
                                                    type="button"
                                                    class="btn btn-primary btn-sm"
                                                    on:click=save_meal_type
                                                    disabled=move || loading.get()
                                                >
                                                    "Save"
                                                </button>
                                                <button
                                                    type="button"
                                                    class="btn btn-secondary btn-sm"
                                                    on:click=cancel_edit
                                                    disabled=move || loading.get()
                                                >
                                                    "Cancel"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid min-w-[34rem] grid-cols-[5rem_minmax(0,1fr)_12rem] items-center gap-2 rounded-md bg-slate-50 px-2 py-1.5">
                                            <span class="text-sm font-semibold text-slate-700">{meal_type.sort_order}</span>
                                            <span class="truncate text-sm text-slate-900">{meal_type.name.clone()}</span>
                                            <div class="flex justify-end">
                                                <button
                                                    type="button"
                                                    class="btn btn-secondary btn-sm"
                                                    on:click={
                                                        let item = meal_type.clone();
                                                        move |_| start_edit(item.clone())
                                                    }
                                                    disabled=move || loading.get()
                                                >
                                                    {icon("edit")}
                                                    "Edit"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </For>
                    </div>
                </div>
            </div>
        </div>
    }
}
