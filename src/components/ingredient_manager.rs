use crate::models::{Ingredient, Category};
use crate::server_functions::ingredients::{get_ingredients, create_ingredient, update_ingredient, delete_ingredient};
use crate::server_functions::categories::get_categories;
use crate::components::{SearchableSelect, ConfirmModal, toast_success, toast_error};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::ev::SubmitEvent;

#[component]
pub fn IngredientManager() -> impl IntoView {
    let (ingredients, set_ingredients) = signal(Vec::<Ingredient>::new());
    let (categories, set_categories) = signal(Vec::<Category>::new());
    let (show_form, set_show_form) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Modal state
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (delete_id, set_delete_id) = signal(0i64);

    // New ingredient form fields
    let (name, set_name) = signal(String::new());
    let (category_id, set_category_id) = signal(0i64);
    let (primary_unit, set_primary_unit) = signal(String::new());
    let (secondary_unit, set_secondary_unit) = signal(String::new());

    // Inline edit state
    let (editing_id, set_editing_id) = signal(None::<i64>);
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_category_id, set_edit_category_id) = signal(0i64);
    let (edit_primary_unit, set_edit_primary_unit) = signal(String::new());
    let (edit_secondary_unit, set_edit_secondary_unit) = signal(String::new());

    // Search
    let (search_query, set_search_query) = signal(String::new());

    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match get_ingredients().await {
                Ok(data) => set_ingredients.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load ingredients: {}", e))),
            }

            match get_categories().await {
                Ok(data) => {
                    if category_id.get() == 0 {
                        if let Some(first) = data.first() {
                            set_category_id.set(first.id);
                        }
                    }
                    set_categories.set(data);
                },
                Err(e) => set_error.set(Some(format!("Failed to load categories: {}", e))),
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_data();
    });

    let reset_form = move || {
        set_name.set(String::new());
        set_primary_unit.set(String::new());
        set_secondary_unit.set(String::new());
        if let Some(first) = categories.get().first() {
            set_category_id.set(first.id);
        }
        set_error.set(None);
    };

    let cancel_form = move |_| {
        set_show_form.set(false);
        set_error.set(None);
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let name_val = name.get();
        let category_id_val = category_id.get();
        let primary_unit_val = primary_unit.get();
        let secondary_unit_val = secondary_unit.get();

        if name_val.is_empty() || primary_unit_val.is_empty() {
            toast_error("Please fill in all required fields");
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let secondary = if secondary_unit_val.is_empty() { None } else { Some(secondary_unit_val) };

            match create_ingredient(name_val, category_id_val, primary_unit_val, secondary).await {
                Ok(_) => {
                    toast_success("Ingredient created successfully!");
                    reset_form();
                    set_show_form.set(false);
                    load_data();
                },
                Err(e) => toast_error(&format!("Failed to create ingredient: {}", e)),
            }

            set_loading.set(false);
        });
    };

    let save_edit = move |id: i64| {
        let name_val = edit_name.get();
        let cat_val = edit_category_id.get();
        let primary_val = edit_primary_unit.get();
        let secondary_val = edit_secondary_unit.get();

        if name_val.is_empty() || primary_val.is_empty() {
            toast_error("Name and primary unit are required");
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            let secondary = if secondary_val.is_empty() { None } else { Some(secondary_val) };
            match update_ingredient(id, name_val, cat_val, primary_val, secondary).await {
                Ok(_) => {
                    toast_success("Ingredient updated!");
                    set_editing_id.set(None);
                    load_data();
                },
                Err(e) => toast_error(&format!("Failed to update: {}", e)),
            }
            set_loading.set(false);
        });
    };

    let handle_delete_click = move |id: i64| {
        set_delete_id.set(id);
        set_show_delete_modal.set(true);
    };

    let confirm_delete = move || {
        let id = delete_id.get();
        set_show_delete_modal.set(false);

        spawn_local(async move {
            set_loading.set(true);
            match delete_ingredient(id).await {
                Ok(_) => {
                    toast_success("Ingredient deleted successfully!");
                    load_data();
                },
                Err(e) => toast_error(&format!("Failed to delete: {}", e)),
            }
            set_loading.set(false);
        });
    };

    let cancel_delete = move || {
        set_show_delete_modal.set(false);
    };

    view! {
        <div class="space-y-4">
            <div class="flex justify-between items-center">
                <h3 class="text-2xl font-bold text-gradient flex items-center gap-2">
                    <span class="text-3xl">"🥕"</span>
                    "Ingredients"
                </h3>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=move |_| {
                        reset_form();
                        set_editing_id.set(None);
                        set_show_form.set(true);
                    }
                    disabled=move || loading.get()
                >
                    <span class="mr-1">"+"</span>
                    " Add Ingredient"
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
                    <h4 class="text-lg font-bold mb-3 text-gradient flex items-center gap-2">
                        <span>"✨"</span>
                        "New Ingredient"
                    </h4>
                    <form on:submit=handle_submit>
                        <div class="grid gap-3 items-end"
                             style="grid-template-columns: minmax(0,3fr) minmax(0,2fr) 7rem 7rem auto">
                            <div>
                                <label class="form-label text-xs">"Name *"</label>
                                <input
                                    type="text"
                                    class="form-input text-sm"
                                    prop:value=move || name.get()
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div>
                                <SearchableSelect
                                    options=categories.into()
                                    selected_value=category_id.into()
                                    on_change=move |id| set_category_id.set(id)
                                    get_id=|c: &Category| c.id.to_string()
                                    get_display=|c: &Category| c.name.clone()
                                    placeholder="Category..."
                                    label="Category *"
                                    required=true
                                />
                            </div>
                            <div>
                                <label class="form-label text-xs">"Primary Unit *"</label>
                                <input
                                    type="text"
                                    class="form-input text-sm"
                                    prop:value=move || primary_unit.get()
                                    on:input=move |ev| set_primary_unit.set(event_target_value(&ev))
                                    placeholder="kg, l, ks..."
                                    required
                                />
                            </div>
                            <div>
                                <label class="form-label text-xs">"Secondary Unit"</label>
                                <input
                                    type="text"
                                    class="form-input text-sm"
                                    prop:value=move || secondary_unit.get()
                                    on:input=move |ev| set_secondary_unit.set(event_target_value(&ev))
                                    placeholder="pcs, cans..."
                                />
                            </div>
                            <div class="flex gap-2">
                                <button type="submit" class="btn btn-primary text-sm" disabled=move || loading.get()>
                                    {move || if loading.get() { "Saving..." } else { "Save" }}
                                </button>
                                <button type="button" class="btn btn-secondary text-sm" on:click=cancel_form disabled=move || loading.get()>
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
            })}

            {move || if loading.get() && ingredients.get().is_empty() {
                view! {
                    <div class="card text-center py-12">
                        <div class="spinner mx-auto mb-4"></div>
                        <p class="text-slate-600">"Loading ingredients..."</p>
                    </div>
                }.into_any()
            } else if ingredients.get().is_empty() {
                view! {
                    <div class="card text-center py-16 bg-gradient-to-br from-slate-50 to-blue-50 border-2 border-dashed border-slate-300">
                        <div class="text-7xl mb-6">"🥕"</div>
                        <h3 class="text-2xl font-bold text-slate-800 mb-3">"No ingredients yet"</h3>
                        <p class="text-lg text-slate-600 mb-8">"Get started by creating your first ingredient"</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="card p-0 overflow-hidden">
                        // Search bar
                        <div class="px-4 py-2.5 border-b border-slate-200 flex items-center gap-3 bg-slate-50">
                            <span class="text-base">"🔍"</span>
                            <input
                                type="text"
                                class="form-input flex-1 text-sm py-1.5"
                                placeholder="Search ingredients..."
                                prop:value=move || search_query.get()
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            />
                        </div>
                        // Header row
                        <div class="grid gap-3 px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wide border-b border-slate-200"
                             style="grid-template-columns: minmax(0,3fr) minmax(0,2fr) 6rem 6rem 5rem">
                            <span>"Name"</span>
                            <span>"Category"</span>
                            <span>"Primary Unit"</span>
                            <span>"Secondary Unit"</span>
                            <span></span>
                        </div>
                        // Rows
                        <div>
                            {move || {
                                let query = search_query.get().to_lowercase();
                                ingredients.get()
                                    .into_iter()
                                    .filter(|i| query.is_empty() || i.name.to_lowercase().contains(&query))
                                    .map(|ing| {
                                        let id = ing.id;
                                        let cat_id = ing.category_id;
                                        let name_s = ing.name.clone();
                                        let primary_s = ing.primary_unit.clone();
                                        let secondary_s = ing.secondary_unit.clone();
                                        let secondary_disp = ing.secondary_unit.clone().unwrap_or_default();

                                        view! {
                                            {move || if editing_id.get() == Some(id) {
                                                view! {
                                                    <div class="grid gap-3 px-4 py-2 items-center border-b border-slate-100 bg-blue-50/60"
                                                         style="grid-template-columns: minmax(0,3fr) minmax(0,2fr) 6rem 6rem 5rem">
                                                        <input type="text" class="form-input text-sm"
                                                            prop:value=move || edit_name.get()
                                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                                        />
                                                        <SearchableSelect
                                                            options=categories.into()
                                                            selected_value=edit_category_id.into()
                                                            on_change=move |cid| set_edit_category_id.set(cid)
                                                            get_id=|c: &Category| c.id.to_string()
                                                            get_display=|c: &Category| c.name.clone()
                                                            placeholder="Category..."
                                                        />
                                                        <input type="text" class="form-input text-sm"
                                                            prop:value=move || edit_primary_unit.get()
                                                            on:input=move |ev| set_edit_primary_unit.set(event_target_value(&ev))
                                                        />
                                                        <input type="text" class="form-input text-sm"
                                                            prop:value=move || edit_secondary_unit.get()
                                                            on:input=move |ev| set_edit_secondary_unit.set(event_target_value(&ev))
                                                        />
                                                        <div class="flex gap-1">
                                                            <button type="button"
                                                                class="text-emerald-600 hover:text-emerald-800 hover:bg-emerald-50 rounded-lg p-1.5 transition-colors font-bold text-base leading-none"
                                                                on:click=move |_| save_edit(id)
                                                                disabled=move || loading.get()
                                                            >"✓"</button>
                                                            <button type="button"
                                                                class="text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg p-1.5 transition-colors text-base leading-none"
                                                                on:click=move |_| set_editing_id.set(None)
                                                            >"✕"</button>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let cat_label = categories.get()
                                                    .iter()
                                                    .find(|c| c.id == cat_id)
                                                    .map(|c| c.name.clone())
                                                    .unwrap_or_default();
                                                let name_c = name_s.clone();
                                                let primary_c = primary_s.clone();
                                                let secondary_c = secondary_s.clone();
                                                view! {
                                                    <div class="grid gap-3 px-4 py-2.5 items-center border-b border-slate-100 hover:bg-slate-50 transition-colors"
                                                         style="grid-template-columns: minmax(0,3fr) minmax(0,2fr) 6rem 6rem 5rem">
                                                        <span class="font-medium text-slate-800 text-sm truncate">{name_s.clone()}</span>
                                                        <span class="text-sm text-slate-600 truncate">{cat_label}</span>
                                                        <span class="text-xs font-medium text-blue-700 bg-blue-100 px-2 py-0.5 rounded-full w-fit">{primary_s.clone()}</span>
                                                        <span class="text-sm text-slate-500">{secondary_disp.clone()}</span>
                                                        <div class="flex gap-1">
                                                            <button type="button"
                                                                class="text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg p-1.5 transition-colors"
                                                                title="Edit"
                                                                on:click=move |_| {
                                                                    set_edit_name.set(name_c.clone());
                                                                    set_edit_category_id.set(cat_id);
                                                                    set_edit_primary_unit.set(primary_c.clone());
                                                                    set_edit_secondary_unit.set(secondary_c.clone().unwrap_or_default());
                                                                    set_editing_id.set(Some(id));
                                                                    set_show_form.set(false);
                                                                }
                                                                disabled=move || loading.get()
                                                            >"✏️"</button>
                                                            <button type="button"
                                                                class="text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg p-1.5 transition-colors"
                                                                title="Delete"
                                                                on:click=move |_| handle_delete_click(id)
                                                                disabled=move || loading.get()
                                                            >"🗑️"</button>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            }}
                                        }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </div>
                }.into_any()
            }}

            <ConfirmModal
                show=show_delete_modal.into()
                on_confirm=confirm_delete
                on_cancel=cancel_delete
                title="Delete Ingredient".to_string()
                message="Are you sure? This action cannot be undone.".to_string()
                confirm_text="Delete".to_string()
                cancel_text="Cancel".to_string()
                variant="danger".to_string()
            />
        </div>
    }
}
