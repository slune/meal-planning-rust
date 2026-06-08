use crate::components::{ConfirmModal, SearchableSelect, icon, toast_error, toast_success};
use crate::models::{CreateRecipeIngredient, Ingredient, Recipe};
use crate::server_functions::ingredients::get_ingredients;
use crate::server_functions::recipes::{
    create_recipe, delete_recipe, get_recipe_with_ingredients, get_recipes, update_recipe,
};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::query_signal;

#[derive(Clone, Debug)]
struct RecipeIngredientForm {
    ingredient_id: i64,
    base_quantity: f64,
    unit: String,
    child_multiplier: Option<f64>,
    teen_multiplier: Option<f64>,
    adult_multiplier: Option<f64>,
    notes: Option<String>,
}

#[component]
pub fn RecipeEditor() -> impl IntoView {
    let (recipes, set_recipes) = signal(Vec::<Recipe>::new());
    let (ingredients, set_ingredients) = signal(Vec::<Ingredient>::new());
    let (show_form, set_show_form) = signal(false);
    let (editing_recipe_id, set_editing_recipe_id) = signal(None::<i64>);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Modal state
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (delete_id, set_delete_id) = signal(0i64);

    // Form fields
    let (name, set_name) = signal(String::new());
    let (instructions, set_instructions) = signal(String::new());
    let (portions, set_portions) = signal(10);
    let (recipe_ingredients, set_recipe_ingredients) = signal(Vec::<RecipeIngredientForm>::new());

    // Search
    let (search_query, set_search_query) = signal(String::new());
    let (recipe_id_query, set_recipe_id_query) = query_signal::<i64>("recipe_id");
    let (opened_query_recipe_id, set_opened_query_recipe_id) = signal(None::<i64>);

    // Load data on mount
    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match get_recipes().await {
                Ok(data) => set_recipes.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load recipes: {}", e))),
            }

            match get_ingredients().await {
                Ok(data) => set_ingredients.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load ingredients: {}", e))),
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_data();
    });

    let reset_form = move || {
        set_name.set(String::new());
        set_instructions.set(String::new());
        set_portions.set(10);
        set_recipe_ingredients.set(Vec::new());
        set_editing_recipe_id.set(None);
        set_recipe_id_query.set(None);
        set_error.set(None);
    };

    let cancel_form = move |_| {
        set_show_form.set(false);
        set_editing_recipe_id.set(None);
        set_error.set(None);
    };

    let add_ingredient = move |_| {
        let mut current = recipe_ingredients.get();
        if let Some(first_ing) = ingredients.get().first() {
            current.push(RecipeIngredientForm {
                ingredient_id: first_ing.id,
                base_quantity: 1.0,
                unit: first_ing.primary_unit.clone(),
                child_multiplier: Some(0.5),
                teen_multiplier: Some(0.75),
                adult_multiplier: Some(1.0),
                notes: None,
            });
            set_recipe_ingredients.set(current);
        }
    };

    let remove_ingredient = move |index: usize| {
        let mut current = recipe_ingredients.get();
        if index < current.len() {
            current.remove(index);
            set_recipe_ingredients.set(current);
        }
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let name_val = name.get();
        let instructions_val = instructions.get();
        let portions_val = portions.get();
        let recipe_ingredients_val = recipe_ingredients.get();
        let editing_id = editing_recipe_id.get();

        if name_val.is_empty() {
            toast_error("Please fill in recipe name");
            return;
        }

        if recipe_ingredients_val.is_empty() {
            toast_error("Please add at least one ingredient");
            return;
        }

        // Validate portions is positive
        if portions_val <= 0 {
            toast_error("Portions must be greater than 0");
            return;
        }

        // Validate ingredient quantities are positive
        for ingredient in &recipe_ingredients_val {
            if ingredient.base_quantity <= 0.0 {
                toast_error("All ingredient quantities must be greater than 0");
                return;
            }

            if [
                ingredient.child_multiplier,
                ingredient.teen_multiplier,
                ingredient.adult_multiplier,
            ]
            .into_iter()
            .flatten()
            .any(|mult| mult < 0.0)
            {
                toast_error("Multipliers cannot be negative");
                return;
            }
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let ingredients_to_create = recipe_ingredients_val
                .into_iter()
                .map(|ri| CreateRecipeIngredient {
                    ingredient_id: ri.ingredient_id,
                    base_quantity: ri.base_quantity,
                    unit: ri.unit,
                    child_multiplier: ri.child_multiplier,
                    teen_multiplier: ri.teen_multiplier,
                    adult_multiplier: ri.adult_multiplier,
                    notes: ri.notes,
                })
                .collect();

            let instructions_opt = if instructions_val.is_empty() {
                None
            } else {
                Some(instructions_val)
            };

            let result = if let Some(id) = editing_id {
                update_recipe(
                    id,
                    name_val,
                    instructions_opt,
                    portions_val,
                    ingredients_to_create,
                )
                .await
                .map_err(|e| format!("Failed to update recipe: {}", e))
            } else {
                create_recipe(
                    name_val,
                    instructions_opt,
                    portions_val,
                    ingredients_to_create,
                )
                .await
                .map_err(|e| format!("Failed to create recipe: {}", e))
            };

            match result {
                Ok(_) => {
                    if editing_id.is_some() {
                        toast_success("Recipe updated successfully!");
                    } else {
                        toast_success("Recipe created successfully!");
                    }
                    reset_form();
                    set_show_form.set(false);
                    load_data();
                }
                Err(e) => toast_error(&e),
            }

            set_loading.set(false);
        });
    };

    let handle_edit = move |id: i64| {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match get_recipe_with_ingredients(id).await {
                Ok(recipe_data) => {
                    set_name.set(recipe_data.recipe.name.clone());
                    set_instructions
                        .set(recipe_data.recipe.instructions.clone().unwrap_or_default());
                    set_portions.set(recipe_data.recipe.portions);

                    let form_ingredients = recipe_data
                        .ingredients
                        .into_iter()
                        .map(|ing| RecipeIngredientForm {
                            ingredient_id: ing.recipe_ingredient.ingredient_id,
                            base_quantity: ing.recipe_ingredient.base_quantity,
                            unit: ing.recipe_ingredient.unit,
                            child_multiplier: ing.recipe_ingredient.child_multiplier,
                            teen_multiplier: ing.recipe_ingredient.teen_multiplier,
                            adult_multiplier: ing.recipe_ingredient.adult_multiplier,
                            notes: ing.recipe_ingredient.notes,
                        })
                        .collect();

                    set_recipe_ingredients.set(form_ingredients);
                    set_editing_recipe_id.set(Some(id));
                    set_show_form.set(true);
                }
                Err(e) => set_error.set(Some(format!("Failed to load recipe: {}", e))),
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        if let Some(recipe_id) = recipe_id_query.get()
            && opened_query_recipe_id.get_untracked() != Some(recipe_id)
        {
            set_opened_query_recipe_id.set(Some(recipe_id));
            set_search_query.set(String::new());
            handle_edit(recipe_id);
        }
    });

    let handle_delete_click = move |id: i64| {
        set_delete_id.set(id);
        set_show_delete_modal.set(true);
    };

    let confirm_delete = move || {
        let id = delete_id.get();
        set_show_delete_modal.set(false);

        spawn_local(async move {
            set_loading.set(true);
            match delete_recipe(id).await {
                Ok(_) => {
                    toast_success("Recipe deleted successfully!");
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
        <div class="space-y-6">
            <div class="page-header">
                <div class="page-heading">
                    <span class="icon-badge icon-badge-amber" aria-hidden="true">{icon("recipes")}</span>
                    <div>
                        <p class="page-kicker">"Library"</p>
                        <h2 class="page-title">"Recipes"</h2>
                        <p class="page-subtitle">"Maintain recipe portions, preparation notes, ingredients, and attendance multipliers."</p>
                        <div class="mt-3">
                            <span class="status-chip status-chip-amber">{move || format!("{} recipes", recipes.get().len())}</span>
                        </div>
                    </div>
                </div>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=move |_| {
                        reset_form();
                        set_show_form.set(true);
                    }
                    disabled=move || loading.get()
                >
                    {icon("plus")}
                    "Add recipe"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert-error">
                    <span class="font-semibold mr-2">"Error:"</span>
                    {err}
                </div>
            })}

            // Search bar
            {move || (!show_form.get() && !recipes.get().is_empty()).then(|| view! {
                <div class="card p-4">
                    <div class="flex items-center gap-3">
                        <span class="inline-icon text-amber-700" aria-hidden="true">{icon("search")}</span>
                        <input
                            type="text"
                            class="form-input flex-1"
                            placeholder="Search recipes by name..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                    </div>
                </div>
            })}

            {move || show_form.get().then(|| view! {
                <div class="card panel-accent panel-accent-amber">
                    <h3 class="section-title mb-6 flex items-center gap-2">
                        <span class="inline-icon text-amber-700" aria-hidden="true">
                            {move || if editing_recipe_id.get().is_some() { icon("edit") } else { icon("plus") }}
                        </span>
                        {move || if editing_recipe_id.get().is_some() { "Edit Recipe" } else { "New Recipe" }}
                    </h3>
                    <form on:submit=handle_submit class="space-y-4">
                        <div>
                            <label class="form-label">"Recipe Name" <span class="text-red-500">"*"</span></label>
                            <input
                                type="text"
                                class="form-input"
                                prop:value=move || name.get()
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div>
                            <label class="form-label">"Portions"</label>
                            <input
                                type="number"
                                class="form-input"
                                prop:value=move || portions.get()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                        set_portions.set(val);
                                    }
                                }
                                min="1"
                            />
                        </div>

                        <div>
                            <label class="form-label">"Instructions"</label>
                            <textarea
                                class="form-input"
                                rows="6"
                                prop:value=move || instructions.get()
                                on:input=move |ev| set_instructions.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="border-t-2 border-slate-200 pt-6">
                            <div class="flex justify-between items-center mb-4">
                                <h4 class="text-lg font-bold text-slate-800 flex items-center gap-2">
                                    <span class="inline-icon text-emerald-700" aria-hidden="true">{icon("ingredients")}</span>
                                    "Ingredients"
                                    <span class="text-red-500">"*"</span>
                                </h4>
                                <button
                                    type="button"
                                    class="btn btn-secondary text-sm"
                                    on:click=add_ingredient
                                >
                                    {icon("plus")}
                                    "Add ingredient"
                                </button>
                            </div>

                            <div class="overflow-x-auto">
                            {move || (!recipe_ingredients.get().is_empty()).then(|| view! {
                                <div class="grid min-w-[42rem] gap-2 px-2 mb-1 text-xs font-semibold text-slate-500 uppercase tracking-wide"
                                     style="grid-template-columns: minmax(0,3fr) 5rem 4.5rem 4rem 4rem 4rem 2rem">
                                    <span>"Ingredient"</span>
                                    <span>"Qty"</span>
                                    <span>"Unit"</span>
                                    <span class="text-center">"Child×"</span>
                                    <span class="text-center">"Teen×"</span>
                                    <span class="text-center">"Adult×"</span>
                                    <span></span>
                                </div>
                            })}
                            <div class="space-y-1">
                                {move || {
                                    recipe_ingredients.get().into_iter().enumerate().map(|(idx, ing)| {
                                        let ingredients_clone = ingredients;
                                        view! {
                                            <div class="grid min-w-[42rem] gap-2 items-center bg-slate-50 rounded-md px-2 py-1.5"
                                                 style="grid-template-columns: minmax(0,3fr) 5rem 4.5rem 4rem 4rem 4rem 2rem">
                                                <SearchableSelect
                                                    options=ingredients.into()
                                                    selected_value=Signal::derive(move || ing.ingredient_id)
                                                    on_change=move |val| {
                                                        let mut current = recipe_ingredients.get();
                                                        if let Some(item) = current.get_mut(idx) {
                                                            item.ingredient_id = val;
                                                            if let Some(selected_ing) = ingredients_clone.get().iter().find(|i| i.id == val) {
                                                                item.unit = selected_ing.primary_unit.clone();
                                                            }
                                                            set_recipe_ingredients.set(current);
                                                        }
                                                    }
                                                    get_id=|ingredient: &Ingredient| ingredient.id.to_string()
                                                    get_display=|ingredient: &Ingredient| ingredient.name.clone()
                                                    placeholder="Search ingredients..."
                                                />
                                                <input
                                                    type="number"
                                                    step="any"
                                                    class="form-input text-sm"
                                                    prop:value=ing.base_quantity.to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                                            let mut current = recipe_ingredients.get();
                                                            if let Some(item) = current.get_mut(idx) {
                                                                item.base_quantity = val;
                                                                set_recipe_ingredients.set(current);
                                                            }
                                                        }
                                                    }
                                                />
                                                <input
                                                    type="text"
                                                    class="form-input text-sm"
                                                    prop:value=ing.unit.clone()
                                                    on:input=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        let mut current = recipe_ingredients.get();
                                                        if let Some(item) = current.get_mut(idx) {
                                                            item.unit = val;
                                                            set_recipe_ingredients.set(current);
                                                        }
                                                    }
                                                />
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    min="0"
                                                    max="2"
                                                    class="form-input text-sm text-center"
                                                    prop:value=ing.child_multiplier.map(|v| v.to_string()).unwrap_or_default()
                                                    on:input=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        let mut current = recipe_ingredients.get();
                                                        if let Some(item) = current.get_mut(idx) {
                                                            item.child_multiplier = val.parse().ok();
                                                            set_recipe_ingredients.set(current);
                                                        }
                                                    }
                                                />
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    min="0"
                                                    max="2"
                                                    class="form-input text-sm text-center"
                                                    prop:value=ing.teen_multiplier.map(|v| v.to_string()).unwrap_or_default()
                                                    on:input=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        let mut current = recipe_ingredients.get();
                                                        if let Some(item) = current.get_mut(idx) {
                                                            item.teen_multiplier = val.parse().ok();
                                                            set_recipe_ingredients.set(current);
                                                        }
                                                    }
                                                />
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    min="0"
                                                    max="2"
                                                    class="form-input text-sm text-center"
                                                    prop:value=ing.adult_multiplier.map(|v| v.to_string()).unwrap_or_default()
                                                    on:input=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        let mut current = recipe_ingredients.get();
                                                        if let Some(item) = current.get_mut(idx) {
                                                            item.adult_multiplier = val.parse().ok();
                                                            set_recipe_ingredients.set(current);
                                                        }
                                                    }
                                                />
                                                <button
                                                    type="button"
                                                    class="text-red-400 hover:text-red-600 hover:bg-red-50 rounded-lg p-1 transition-colors text-lg leading-none w-full flex items-center justify-center"
                                                    on:click=move |_| remove_ingredient(idx)
                                                >
                                                    "×"
                                                </button>
                                            </div>
                                        }
                                    }).collect_view()
                                }}
                            </div>
                            </div>
                        </div>

                        <div class="flex gap-2">
                            <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                                {icon("recipes")}
                                {move || {
                                    if loading.get() {
                                        "Saving..."
                                    } else if editing_recipe_id.get().is_some() {
                                        "Update Recipe"
                                    } else {
                                        "Save Recipe"
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
                        <div class="spinner mx-auto mb-4"></div>
                        <p class="text-slate-600">"Loading recipes..."</p>
                    </div>
                }.into_any()
            } else if recipes.get().is_empty() {
                view! {
                    <div class="empty-state">
                        <span class="icon-badge icon-badge-amber mx-auto mb-4" aria-hidden="true">{icon("recipes")}</span>
                        <h3 class="text-xl font-semibold text-slate-950 mb-2">"No recipes yet"</h3>
                        <p class="text-sm text-slate-600">"Create recipes after ingredients are available."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For
                            each=move || {
                                let query = search_query.get().to_lowercase();
                                recipes.get()
                                    .into_iter()
                                    .filter(|r| query.is_empty() || r.name.to_lowercase().contains(&query))
                                    .collect::<Vec<_>>()
                            }
                            key=|recipe| recipe.id
                            let:recipe
                        >
                            <div class="record-card record-card-amber">
                                <div class="flex items-start justify-between mb-3">
                                    <span class="icon-badge icon-badge-amber" aria-hidden="true">{icon("recipes")}</span>
                                    <span class="badge badge-primary">{recipe.portions} " portions"</span>
                                </div>
                                <h3 class="text-xl font-bold text-slate-800 mb-4">{recipe.name.clone()}</h3>
                                <div class="mt-auto flex gap-2">
                                    <button
                                        class="btn btn-secondary text-sm flex-1"
                                        on:click={
                                            let id = recipe.id;
                                            move |_| handle_edit(id)
                                        }
                                        disabled=move || loading.get()
                                    >
                                        {icon("edit")}
                                        "Edit"
                                    </button>
                                    <button
                                        class="btn btn-danger text-sm"
                                        on:click={
                                            let id = recipe.id;
                                            move |_| handle_delete_click(id)
                                        }
                                        disabled=move || loading.get()
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

            <ConfirmModal
                show=show_delete_modal.into()
                on_confirm=confirm_delete
                on_cancel=cancel_delete
                title="Delete Recipe".to_string()
                message="Are you sure? This action cannot be undone.".to_string()
                confirm_text="Delete".to_string()
                cancel_text="Cancel".to_string()
                variant="danger".to_string()
            />
        </div>
    }
}
