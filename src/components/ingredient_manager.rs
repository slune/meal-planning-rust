use crate::components::{ConfirmModal, SearchableSelect, icon, toast_error, toast_success};
use crate::models::{Category, Ingredient, IngredientUsage};
use crate::server_functions::categories::get_categories;
use crate::server_functions::ingredients::{
    create_ingredient, delete_ingredient, get_ingredient_usage, get_ingredients, merge_ingredients,
    update_ingredient,
};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::query_signal;

#[component]
pub fn IngredientManager() -> impl IntoView {
    let (ingredients, set_ingredients) = signal(Vec::<Ingredient>::new());
    let (ingredient_usage, set_ingredient_usage) = signal(Vec::<IngredientUsage>::new());
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

    // Merge state
    let (selected_ids, set_selected_ids) = signal(Vec::<i64>::new());
    let (show_merge_form, set_show_merge_form) = signal(false);
    let (expanded_usage_id, set_expanded_usage_id) = signal(None::<i64>);
    let (opened_query_ingredient_id, set_opened_query_ingredient_id) = signal(None::<i64>);
    let (merge_name, set_merge_name) = signal(String::new());
    let (merge_category_id, set_merge_category_id) = signal(0i64);
    let (merge_primary_unit, set_merge_primary_unit) = signal(String::new());
    let (merge_secondary_unit, set_merge_secondary_unit) = signal(String::new());

    // Search
    let (search_query, set_search_query) = signal(String::new());
    let (ingredient_id_query, set_ingredient_id_query) = query_signal::<i64>("ingredient_id");

    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match get_ingredients().await {
                Ok(data) => set_ingredients.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load ingredients: {}", e))),
            }

            match get_ingredient_usage().await {
                Ok(data) => set_ingredient_usage.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load ingredient usage: {}", e))),
            }

            match get_categories().await {
                Ok(data) => {
                    if category_id.get() == 0
                        && let Some(first) = data.first()
                    {
                        set_category_id.set(first.id);
                    }
                    set_categories.set(data);
                }
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
        set_ingredient_id_query.set(None);
        if let Some(first) = categories.get().first() {
            set_category_id.set(first.id);
        }
        set_error.set(None);
    };

    let cancel_form = move |_| {
        set_show_form.set(false);
        set_error.set(None);
    };

    let cancel_merge = move |_| {
        set_show_merge_form.set(false);
        set_error.set(None);
    };

    let usage_recipes_for = move |ingredient_id: i64| {
        ingredient_usage
            .get()
            .into_iter()
            .find(|usage| usage.ingredient_id == ingredient_id)
            .map(|usage| usage.recipes)
            .unwrap_or_default()
    };

    let start_edit = move |ingredient: Ingredient| {
        set_edit_name.set(ingredient.name);
        set_edit_category_id.set(ingredient.category_id);
        set_edit_primary_unit.set(ingredient.primary_unit);
        set_edit_secondary_unit.set(ingredient.secondary_unit.unwrap_or_default());
        set_editing_id.set(Some(ingredient.id));
        set_show_form.set(false);
        set_show_merge_form.set(false);
    };

    Effect::new(move |_| {
        if let Some(ingredient_id) = ingredient_id_query.get()
            && opened_query_ingredient_id.get_untracked() != Some(ingredient_id)
            && let Some(ingredient) = ingredients
                .get()
                .into_iter()
                .find(|ingredient| ingredient.id == ingredient_id)
        {
            set_opened_query_ingredient_id.set(Some(ingredient_id));
            set_search_query.set(ingredient.name.clone());
            start_edit(ingredient);
        }
    });

    let toggle_selected = move |id: i64, checked: bool| {
        let mut current = selected_ids.get();
        if checked {
            if !current.contains(&id) {
                current.push(id);
            }
        } else {
            current.retain(|selected| *selected != id);
        }
        set_selected_ids.set(current);
    };

    let open_merge_form = move |_| {
        let ids = selected_ids.get();
        if ids.len() < 2 {
            toast_error("Select at least two ingredients to merge");
            return;
        }

        let selected = ingredients
            .get()
            .into_iter()
            .filter(|ingredient| ids.contains(&ingredient.id))
            .collect::<Vec<_>>();

        if selected.len() != ids.len() {
            toast_error("One or more selected ingredients are no longer available");
            return;
        }

        let Some(first) = selected.first() else {
            toast_error("Select at least two ingredients to merge");
            return;
        };

        if selected.iter().any(|ingredient| {
            ingredient.primary_unit != first.primary_unit
                || ingredient.secondary_unit != first.secondary_unit
        }) {
            toast_error("Selected ingredients must have the same primary and secondary units");
            return;
        }

        if let Some(target) = selected.iter().min_by_key(|ingredient| ingredient.id) {
            set_merge_name.set(target.name.clone());
            set_merge_category_id.set(target.category_id);
            set_merge_primary_unit.set(target.primary_unit.clone());
            set_merge_secondary_unit.set(target.secondary_unit.clone().unwrap_or_default());
            set_show_form.set(false);
            set_editing_id.set(None);
            set_show_merge_form.set(true);
            set_error.set(None);
        }
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

            let secondary = if secondary_unit_val.is_empty() {
                None
            } else {
                Some(secondary_unit_val)
            };

            match create_ingredient(name_val, category_id_val, primary_unit_val, secondary).await {
                Ok(_) => {
                    toast_success("Ingredient created successfully!");
                    reset_form();
                    set_show_form.set(false);
                    load_data();
                }
                Err(e) => toast_error(format!("Failed to create ingredient: {}", e)),
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
            let secondary = if secondary_val.is_empty() {
                None
            } else {
                Some(secondary_val)
            };
            match update_ingredient(id, name_val, cat_val, primary_val, secondary).await {
                Ok(_) => {
                    toast_success("Ingredient updated!");
                    set_editing_id.set(None);
                    load_data();
                }
                Err(e) => toast_error(format!("Failed to update: {}", e)),
            }
            set_loading.set(false);
        });
    };

    let submit_merge = move |ev: SubmitEvent| {
        ev.prevent_default();

        let mut ids = selected_ids.get();
        ids.sort_unstable();
        ids.dedup();

        if ids.len() < 2 {
            toast_error("Select at least two ingredients to merge");
            return;
        }

        let target_id = ids[0];
        let name_val = merge_name.get();
        let cat_val = merge_category_id.get();
        let primary_val = merge_primary_unit.get();
        let secondary_val = merge_secondary_unit.get();

        if name_val.is_empty() || primary_val.is_empty() {
            toast_error("Name and primary unit are required");
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            let secondary = if secondary_val.is_empty() {
                None
            } else {
                Some(secondary_val)
            };

            match merge_ingredients(target_id, ids, name_val, cat_val, primary_val, secondary).await
            {
                Ok(result) => {
                    toast_success(format!(
                        "Merged {} ingredients and updated {} recipe rows",
                        result.merged_ingredient_count, result.updated_recipe_rows
                    ));
                    set_selected_ids.set(Vec::new());
                    set_expanded_usage_id.set(Some(result.ingredient.id));
                    set_show_merge_form.set(false);
                    load_data();
                }
                Err(e) => toast_error(format!("Failed to merge ingredients: {}", e)),
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
                    let mut current = selected_ids.get();
                    current.retain(|selected| *selected != id);
                    set_selected_ids.set(current);
                    if expanded_usage_id.get() == Some(id) {
                        set_expanded_usage_id.set(None);
                    }
                    load_data();
                }
                Err(e) => toast_error(format!("Failed to delete: {}", e)),
            }
            set_loading.set(false);
        });
    };

    let cancel_delete = move || {
        set_show_delete_modal.set(false);
    };

    view! {
        <div class="space-y-4">
            <div class="page-header">
                <div class="page-heading">
                    <span class="icon-badge icon-badge-emerald" aria-hidden="true">{icon("ingredients")}</span>
                    <div>
                        <p class="page-kicker">"Catalog"</p>
                        <h3 class="page-title">"Ingredients"</h3>
                        <p class="page-subtitle">"Maintain the ingredient units used by recipes and shopping reports."</p>
                        <div class="mt-3">
                            <span class="status-chip status-chip-emerald">{move || format!("{} ingredients", ingredients.get().len())}</span>
                        </div>
                    </div>
                </div>
                <div class="flex flex-wrap gap-2">
                    <button
                        type="button"
                        class="btn btn-secondary"
                        on:click=open_merge_form
                        disabled=move || loading.get() || selected_ids.get().len() < 2
                    >
                        {icon("copy")}
                        {move || {
                            let count = selected_ids.get().len();
                            if count == 0 {
                                "Merge".to_string()
                            } else {
                                format!("Merge ({})", count)
                            }
                        }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-primary"
                        on:click=move |_| {
                            reset_form();
                            set_editing_id.set(None);
                            set_show_merge_form.set(false);
                            set_show_form.set(true);
                        }
                        disabled=move || loading.get()
                    >
                        {icon("plus")}
                        "Add ingredient"
                    </button>
                </div>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert-error">
                    <span class="font-semibold mr-2">"Error:"</span>
                    {err}
                </div>
            })}

            {move || show_form.get().then(|| view! {
                <div class="card panel-accent panel-accent-emerald">
                    <h4 class="section-title mb-4 flex items-center gap-2">
                        <span class="inline-icon text-emerald-700" aria-hidden="true">{icon("plus")}</span>
                        "New Ingredient"
                    </h4>
                    <form on:submit=handle_submit>
                        <div class="grid grid-cols-1 gap-3 items-end lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)_7rem_7rem_auto]">
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
                                    {icon("plus")}
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

            {move || show_merge_form.get().then(|| {
                let ids = selected_ids.get();
                let selected = ingredients
                    .get()
                    .into_iter()
                    .filter(|ingredient| ids.contains(&ingredient.id))
                    .collect::<Vec<_>>();

                view! {
                    <div class="card panel-accent panel-accent-emerald">
                        <h4 class="section-title mb-4 flex items-center gap-2">
                            <span class="inline-icon text-emerald-700" aria-hidden="true">{icon("copy")}</span>
                            "Merge Ingredients"
                        </h4>
                        <div class="mb-4 flex flex-wrap gap-2">
                            {selected.into_iter().map(|ingredient| {
                                view! {
                                    <span class="status-chip status-chip-emerald">{ingredient.name}</span>
                                }
                            }).collect_view()}
                        </div>
                        <form on:submit=submit_merge>
                            <div class="grid grid-cols-1 gap-3 items-end lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)_7rem_7rem_auto]">
                                <div>
                                    <label class="form-label text-xs">"Name *"</label>
                                    <input
                                        type="text"
                                        class="form-input text-sm"
                                        prop:value=move || merge_name.get()
                                        on:input=move |ev| set_merge_name.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div>
                                    <SearchableSelect
                                        options=categories.into()
                                        selected_value=merge_category_id.into()
                                        on_change=move |id| set_merge_category_id.set(id)
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
                                        prop:value=move || merge_primary_unit.get()
                                        on:input=move |ev| set_merge_primary_unit.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div>
                                    <label class="form-label text-xs">"Secondary Unit"</label>
                                    <input
                                        type="text"
                                        class="form-input text-sm"
                                        prop:value=move || merge_secondary_unit.get()
                                        on:input=move |ev| set_merge_secondary_unit.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="flex gap-2">
                                    <button type="submit" class="btn btn-primary text-sm" disabled=move || loading.get()>
                                        {icon("copy")}
                                        {move || if loading.get() { "Merging..." } else { "Merge" }}
                                    </button>
                                    <button type="button" class="btn btn-secondary text-sm" on:click=cancel_merge disabled=move || loading.get()>
                                        "Cancel"
                                    </button>
                                </div>
                            </div>
                        </form>
                    </div>
                }
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
                    <div class="empty-state">
                        <span class="icon-badge icon-badge-emerald mx-auto mb-4" aria-hidden="true">{icon("ingredients")}</span>
                        <h3 class="text-xl font-semibold text-slate-950 mb-2">"No ingredients yet"</h3>
                        <p class="text-sm text-slate-600">"Add ingredients with units before building recipes."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="card p-0 overflow-x-auto">
                        // Search bar
                        <div class="px-4 py-2.5 border-b border-slate-200 flex items-center gap-3 bg-slate-50">
                            <span class="inline-icon text-emerald-700" aria-hidden="true">{icon("search")}</span>
                            <input
                                type="text"
                                class="form-input flex-1 text-sm py-1.5"
                                placeholder="Search ingredients..."
                                prop:value=move || search_query.get()
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            />
                            {move || (!selected_ids.get().is_empty()).then(|| view! {
                                <span class="status-chip status-chip-emerald">{move || format!("{} selected", selected_ids.get().len())}</span>
                            })}
                        </div>
                        // Header row
                        <div class="grid gap-3 px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wide border-b border-slate-200"
                             style="grid-template-columns: 2rem minmax(0,3fr) minmax(0,2fr) 6rem 6rem 7rem 6rem">
                            <span></span>
                            <span>"Name"</span>
                            <span>"Category"</span>
                            <span>"Primary Unit"</span>
                            <span>"Secondary Unit"</span>
                            <span>"Recipes"</span>
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
                                        let ingredient_for_edit = Ingredient {
                                            id,
                                            name: name_s.clone(),
                                            category_id: cat_id,
                                            primary_unit: primary_s.clone(),
                                            secondary_unit: secondary_s.clone(),
                                            created_at: None,
                                            updated_at: None,
                                        };

                                        view! {
                                            <div>
                                                {move || if editing_id.get() == Some(id) {
                                                    view! {
                                                        <div class="grid gap-3 px-4 py-2 items-center border-b border-slate-100 bg-sky-50/60"
                                                             style="grid-template-columns: 2rem minmax(0,3fr) minmax(0,2fr) 6rem 6rem 7rem 6rem">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox"
                                                                prop:checked=move || selected_ids.get().contains(&id)
                                                                on:change=move |ev| toggle_selected(id, event_target_checked(&ev))
                                                            />
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
                                                            <div class="text-sm">
                                                                {move || {
                                                                    let count = usage_recipes_for(id).len();
                                                                    if count == 0 {
                                                                        view! { <span class="text-slate-400">"Unused"</span> }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <button
                                                                                type="button"
                                                                                class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold text-sky-700 hover:bg-slate-100 hover:text-sky-800"
                                                                                on:click=move |_| {
                                                                                    if expanded_usage_id.get() == Some(id) {
                                                                                        set_expanded_usage_id.set(None);
                                                                                    } else {
                                                                                        set_expanded_usage_id.set(Some(id));
                                                                                    }
                                                                                }
                                                                            >
                                                                                {format!("{} recipes", count)}
                                                                            </button>
                                                                        }.into_any()
                                                                    }
                                                                }}
                                                            </div>
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
                                                    view! {
                                                        <div class="grid gap-3 px-4 py-2.5 items-center border-b border-slate-100 hover:bg-slate-50 transition-colors"
                                                             style="grid-template-columns: 2rem minmax(0,3fr) minmax(0,2fr) 6rem 6rem 7rem 6rem">
                                                            <input
                                                                type="checkbox"
                                                                class="form-checkbox"
                                                                prop:checked=move || selected_ids.get().contains(&id)
                                                                on:change=move |ev| toggle_selected(id, event_target_checked(&ev))
                                                            />
                                                            <span class="font-medium text-slate-800 text-sm truncate">{name_s.clone()}</span>
                                                            <span class="text-sm text-slate-600 truncate">{cat_label}</span>
                                                            <span class="w-fit rounded-full bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-800">{primary_s.clone()}</span>
                                                            <span class="text-sm text-slate-500">{secondary_disp.clone()}</span>
                                                            <div class="text-sm">
                                                                {move || {
                                                                    let count = usage_recipes_for(id).len();
                                                                    if count == 0 {
                                                                        view! { <span class="text-slate-400">"Unused"</span> }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <button
                                                                                type="button"
                                                                                class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold text-sky-700 hover:bg-slate-100 hover:text-sky-800"
                                                                                on:click=move |_| {
                                                                                    if expanded_usage_id.get() == Some(id) {
                                                                                        set_expanded_usage_id.set(None);
                                                                                    } else {
                                                                                        set_expanded_usage_id.set(Some(id));
                                                                                    }
                                                                                }
                                                                            >
                                                                                {format!("{} recipes", count)}
                                                                            </button>
                                                                        }.into_any()
                                                                    }
                                                                }}
                                                            </div>
                                                            <div class="flex gap-1">
                                                                <button type="button"
                                                                    class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold text-slate-500 hover:bg-slate-100 hover:text-slate-900"
                                                                    title="Edit"
                                                                    on:click={
                                                                        let ingredient = ingredient_for_edit.clone();
                                                                        move |_| {
                                                                            start_edit(ingredient.clone());
                                                                        }
                                                                    }
                                                                    disabled=move || loading.get()
                                                                >
                                                                    {icon("edit")}
                                                                    "Edit"
                                                                </button>
                                                                <button type="button"
                                                                    class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold text-rose-600 hover:bg-rose-50 hover:text-rose-700"
                                                                    title="Delete"
                                                                    on:click=move |_| handle_delete_click(id)
                                                                    disabled=move || loading.get()
                                                                >
                                                                    {icon("trash")}
                                                                    "Del"
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                }}
                                                {move || (expanded_usage_id.get() == Some(id)).then(|| {
                                                    let recipes = usage_recipes_for(id);
                                                    view! {
                                                        <div class="px-4 py-3 border-b border-slate-100 bg-slate-50">
                                                            <div class="grid gap-3 text-sm"
                                                                 style="grid-template-columns: 2rem minmax(0,3fr) minmax(0,2fr) 6rem 6rem 7rem 6rem">
                                                                <span></span>
                                                                <div class="space-y-2" style="grid-column: 2 / -1">
                                                                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-500">"Used in recipes"</p>
                                                                    <div class="flex flex-wrap gap-2">
                                                                        {recipes.into_iter().map(|recipe| view! {
                                                                            <a
                                                                                href=format!("/recipes?recipe_id={}", recipe.id)
                                                                                class="status-chip status-chip-emerald text-emerald-800 no-underline hover:text-emerald-800"
                                                                            >
                                                                                {recipe.name}
                                                                            </a>
                                                                        }).collect_view()}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                })}
                                            </div>
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
