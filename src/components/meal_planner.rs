use crate::models::{Recipe, PlannedMealWithDetails, MealType, Camp};
use crate::server_functions::meal_plans::{get_planned_meals_for_date, get_planned_meals_for_camp, get_planned_meals_for_date_range, create_planned_meal, update_planned_meal, delete_planned_meal};
use crate::server_functions::recipes::get_recipes;
use crate::server_functions::camps::{get_camp, get_camps};
use crate::components::{SearchableSelect, ConfirmModal, toast_success, toast_error};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_params_map, use_navigate};
use leptos::ev::SubmitEvent;
use chrono::{NaiveDate, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    SingleDay,
    AllDays,
}

#[component]
pub fn MealPlanner() -> impl IntoView {
    let params = use_params_map();
    let camp_id = Memo::new(move |_| {
        params.with(|p| {
            p.get("camp_id")
                .and_then(|id| id.parse::<i64>().ok())
                .unwrap_or(0)
        })
    });

    let navigate = use_navigate();

    let (planned_meals, set_planned_meals) = signal(Vec::<PlannedMealWithDetails>::new());
    let (multi_day_meals, set_multi_day_meals) = signal(HashMap::<String, Vec<PlannedMealWithDetails>>::new());
    let (recipes, set_recipes) = signal(Vec::<Recipe>::new());
    let (camps, set_camps) = signal(Vec::<Camp>::new());
    let (camp, set_camp) = signal(None::<Camp>);
    let (show_form, set_show_form) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (selected_date, set_selected_date) = signal(String::new());
    let (view_mode, set_view_mode) = signal(ViewMode::SingleDay);

    // Form fields
    let (editing_meal_id, set_editing_meal_id) = signal(None::<i64>);
    let (meal_type, set_meal_type) = signal(MealType::Breakfast);
    let (recipe_id, set_recipe_id) = signal(0i64);
    let (children, set_children) = signal(0);
    let (teens, set_teens) = signal(0);
    let (adults, set_adults) = signal(0);

    // Modal state
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (delete_id, set_delete_id) = signal(0i64);

    // Load camps, recipes on mount
    Effect::new(move |_| {
        spawn_local(async move {
            // Load all camps
            match get_camps().await {
                Ok(data) => set_camps.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load camps: {}", e))),
            }
            
            // Load recipes
            match get_recipes().await {
                Ok(data) => {
                    set_recipes.set(data.clone());
                    if let Some(first) = data.first() {
                        set_recipe_id.set(first.id);
                    }
                },
                Err(e) => set_error.set(Some(format!("Failed to load recipes: {}", e))),
            }
        });
    });

    // Load camp when camp_id changes
    Effect::new(move |_| {
        let current_camp_id = camp_id.get();
        
        spawn_local(async move {
            // Load camp if camp_id is valid
            if current_camp_id > 0 {
                match get_camp(current_camp_id).await {
                    Ok(camp_data) => {
                        // Set selected date to camp start date if it's not set or outside camp range
                        let current_selected = selected_date.get_untracked();
                        if current_selected.is_empty() {
                            set_selected_date.set(camp_data.start_date.format("%Y-%m-%d").to_string());
                        }
                        set_camp.set(Some(camp_data));
                    },
                    Err(e) => set_error.set(Some(format!("Failed to load camp: {}", e))),
                }
            } else {
                set_camp.set(None);
            }
        });
    });

    // Load planned meals based on view mode
    let load_meals = move || {
        let current_camp_id = camp_id.get_untracked();
        let current_date = selected_date.get_untracked();
        let current_view_mode = view_mode.get_untracked();

        if current_camp_id == 0 {
            set_error.set(Some("No camp selected".to_string()));
            return;
        }

        if current_date.is_empty() {
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match current_view_mode {
                ViewMode::SingleDay => {
                    match get_planned_meals_for_date(current_camp_id, current_date).await {
                        Ok(mut data) => {
                            data.sort_by_key(|m| {
                                MealType::from_str(&m.planned_meal.meal_type)
                                    .map(|mt| mt.sort_order())
                                    .unwrap_or(99)
                            });
                            set_planned_meals.set(data);
                        },
                        Err(e) => set_error.set(Some(format!("Failed to load meals: {}", e))),
                    }
                },
                ViewMode::AllDays => {
                    match get_planned_meals_for_camp(current_camp_id).await {
                        Ok(data) => {
                            let mut map = HashMap::new();
                            for (date, mut meals) in data {
                                meals.sort_by_key(|m| {
                                    MealType::from_str(&m.planned_meal.meal_type)
                                        .map(|mt| mt.sort_order())
                                        .unwrap_or(99)
                                });
                                map.insert(date, meals);
                            }
                            set_multi_day_meals.set(map);
                        },
                        Err(e) => set_error.set(Some(format!("Failed to load meals: {}", e))),
                    }
                },
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        let _ = selected_date.get();
        let _ = view_mode.get();
        load_meals();
    });

    let reset_form = move || {
        set_editing_meal_id.set(None);
        set_meal_type.set(MealType::Breakfast);
        if let Some(first) = recipes.get().first() {
            set_recipe_id.set(first.id);
        }
        if let Some(c) = camp.get() {
            set_children.set(c.default_children);
            set_teens.set(c.default_teens);
            set_adults.set(c.default_adults);
        } else {
            set_children.set(0);
            set_teens.set(0);
            set_adults.set(0);
        }
        set_error.set(None);
    };

    let cancel_form = move |_| {
        set_show_form.set(false);
        set_error.set(None);
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let current_camp_id = camp_id.get();
        let date_val = selected_date.get();
        let meal_type_val = meal_type.get();
        let recipe_id_val = recipe_id.get();
        let children_val = children.get();
        let teens_val = teens.get();
        let adults_val = adults.get();
        let editing_id = editing_meal_id.get();

        if current_camp_id == 0 {
            toast_error("No camp selected");
            return;
        }

        // Validate attendance counts are non-negative
        if children_val < 0 {
            toast_error("Number of children cannot be negative");
            return;
        }
        if teens_val < 0 {
            toast_error("Number of teens cannot be negative");
            return;
        }
        if adults_val < 0 {
            toast_error("Number of adults cannot be negative");
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let attendance = if children_val > 0 || teens_val > 0 || adults_val > 0 {
                (Some(children_val), Some(teens_val), Some(adults_val))
            } else {
                (None, None, None)
            };

            let result: Result<(), _> = if let Some(id) = editing_id {
                // Update existing meal
                update_planned_meal(
                    id,
                    recipe_id_val,
                    attendance.0,
                    attendance.1,
                    attendance.2,
                ).await.map_err(|e| e.to_string())
            } else {
                // Create new meal
                let meal_type_str = meal_type_val.as_str().to_string();
                create_planned_meal(
                    current_camp_id,
                    date_val,
                    meal_type_str,
                    recipe_id_val,
                    attendance.0,
                    attendance.1,
                    attendance.2,
                ).await.map(|_| ()).map_err(|e| e.to_string())
            };

            match result {
                Ok(_) => {
                    toast_success(if editing_id.is_some() {
                        "Meal updated successfully!"
                    } else {
                        "Meal added successfully!"
                    });
                    reset_form();
                    set_show_form.set(false);
                    load_meals();
                },
                Err(error_msg) => {
                    toast_error(&format!("Failed to save meal: {}", error_msg));
                },
            }

            set_loading.set(false);
        });
    };

    // Handle edit click
    let handle_edit_click = move |meal: PlannedMealWithDetails| {
        set_editing_meal_id.set(Some(meal.planned_meal.id));
        set_recipe_id.set(meal.planned_meal.recipe_id);

        if let Some(mt) = MealType::from_str(&meal.planned_meal.meal_type) {
            set_meal_type.set(mt);
        }

        if let Some(att) = meal.attendance {
            set_children.set(att.children);
            set_teens.set(att.teens);
            set_adults.set(att.adults);
        } else if let Some(c) = camp.get() {
            set_children.set(c.default_children);
            set_teens.set(c.default_teens);
            set_adults.set(c.default_adults);
        }

        set_show_form.set(true);
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

            match delete_planned_meal(id).await {
                Ok(_) => {
                    toast_success("Meal deleted successfully!");
                    load_meals();
                },
                Err(e) => toast_error(&format!("Failed to delete meal: {}", e)),
            }

            set_loading.set(false);
        });
    };

    let cancel_delete = move || {
        set_show_delete_modal.set(false);
    };

    let format_meal_type = |meal_type_str: &str| -> &'static str {
        match meal_type_str {
            "breakfast" => "Breakfast",
            "morning_snack" => "Morning Snack",
            "lunch" => "Lunch",
            "afternoon_snack" => "Afternoon Snack",
            "dinner" => "Dinner",
            _ => "Unknown",
        }
    };

    let go_to_previous_day = move |_| {
        let current_date = selected_date.get();
        if let Ok(date) = NaiveDate::parse_from_str(&current_date, "%Y-%m-%d") {
            if let Some(prev) = date.checked_sub_signed(Duration::days(1)) {
                if let Some(c) = camp.get() {
                    if prev >= c.start_date {
                        set_selected_date.set(prev.format("%Y-%m-%d").to_string());
                    }
                } else {
                    set_selected_date.set(prev.format("%Y-%m-%d").to_string());
                }
            }
        }
    };

    let go_to_next_day = move |_| {
        let current_date = selected_date.get();
        if let Ok(date) = NaiveDate::parse_from_str(&current_date, "%Y-%m-%d") {
            if let Some(next) = date.checked_add_signed(Duration::days(1)) {
                if let Some(c) = camp.get() {
                    if next <= c.end_date {
                        set_selected_date.set(next.format("%Y-%m-%d").to_string());
                    }
                } else {
                    set_selected_date.set(next.format("%Y-%m-%d").to_string());
                }
            }
        }
    };

    let get_camp_day_info = move || -> Option<String> {
        if let Some(c) = camp.get() {
            let current_date = selected_date.get();
            if let Ok(date) = NaiveDate::parse_from_str(&current_date, "%Y-%m-%d") {
                if date >= c.start_date && date <= c.end_date {
                    let day_num = (date - c.start_date).num_days() + 1;
                    return Some(format!("Day {} of {}", day_num, (c.end_date - c.start_date).num_days() + 1));
                }
            }
        }
        None
    };

    let can_go_previous = move || -> bool {
        if let Some(c) = camp.get() {
            let current_date = selected_date.get();
            if let Ok(date) = NaiveDate::parse_from_str(&current_date, "%Y-%m-%d") {
                return date > c.start_date;
            }
        }
        false
    };

    let can_go_next = move || -> bool {
        if let Some(c) = camp.get() {
            let current_date = selected_date.get();
            if let Ok(date) = NaiveDate::parse_from_str(&current_date, "%Y-%m-%d") {
                return date < c.end_date;
            }
        }
        false
    };

    let nav_for_selector = navigate.clone();
    let nav_for_empty_state = navigate.clone();

    view! {
        <div class="space-y-6">
            // Camp Selector
            <div class="card">
                <SearchableSelect
                    options=camps.into()
                    selected_value=Signal::derive(move || camp_id.get())
                    on_change=move |id| {
                        if id > 0 {
                            nav_for_selector(&format!("/planner/{}", id), Default::default());
                        }
                    }
                    get_id=|camp: &Camp| camp.id.to_string()
                    get_display=|camp: &Camp| format!("{} ({} to {})",
                        camp.name,
                        camp.start_date.format("%Y-%m-%d"),
                        camp.end_date.format("%Y-%m-%d")
                    )
                    placeholder="Search camps..."
                    label="Select Camp"
                />
            </div>

            <div class="flex justify-between items-center flex-wrap gap-4">
                <div>
                    <h2 class="text-2xl font-bold">"Meal Planner"</h2>
                    {move || camp.get().map(|c| view! {
                        <p class="text-slate-600 mt-1">
                            {c.name.clone()} " ("
                            {c.start_date.format("%Y-%m-%d").to_string()}
                            " to "
                            {c.end_date.format("%Y-%m-%d").to_string()}
                            ")"
                        </p>
                    })}
                </div>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=move |_| {
                        reset_form();
                        set_show_form.set(true);
                    }
                    disabled=move || loading.get() || camp_id.get() == 0
                >
                    "+ Add Meal"
                </button>
            </div>

            // View Mode Selector
            <div class="card">
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-sm font-medium text-slate-700">"View:"</span>
                    <div class="flex gap-2">
                        <button
                            type="button"
                            class=move || if view_mode.get() == ViewMode::SingleDay {
                                "px-4 py-2 rounded-lg font-medium bg-indigo-600 text-white shadow-md"
                            } else {
                                "px-4 py-2 rounded-lg font-medium bg-slate-200 text-slate-700 hover:bg-slate-300"
                            }
                            on:click=move |_| set_view_mode.set(ViewMode::SingleDay)
                            disabled=move || loading.get() || camp_id.get() == 0
                        >
                            "Single Day"
                        </button>
                        <button
                            type="button"
                            class=move || if view_mode.get() == ViewMode::AllDays {
                                "px-4 py-2 rounded-lg font-medium bg-indigo-600 text-white shadow-md"
                            } else {
                                "px-4 py-2 rounded-lg font-medium bg-slate-200 text-slate-700 hover:bg-slate-300"
                            }
                            on:click=move |_| set_view_mode.set(ViewMode::AllDays)
                            disabled=move || loading.get() || camp_id.get() == 0
                        >
                            "All Days"
                        </button>
                    </div>
                </div>
            </div>

            // Date navigation - only show in single day mode
            {move || if view_mode.get() == ViewMode::SingleDay {
                Some(view! {
                    <div class="card">
                        <div class="flex items-center justify-between gap-4">
                            <button
                                type="button"
                                class="btn btn-secondary"
                                on:click=go_to_previous_day
                                disabled=move || !can_go_previous() || loading.get()
                            >
                                "← Previous Day"
                            </button>

                            <div class="flex flex-col items-center gap-2">
                                <input
                                    type="date"
                                    class="form-input text-center"
                                    prop:value=move || selected_date.get()
                                    on:input=move |ev| set_selected_date.set(event_target_value(&ev))
                                    prop:min=move || camp.get().map(|c| c.start_date.format("%Y-%m-%d").to_string())
                                    prop:max=move || camp.get().map(|c| c.end_date.format("%Y-%m-%d").to_string())
                                />
                                {move || get_camp_day_info().map(|info| view! {
                                    <span class="text-sm text-blue-600 font-medium">{info}</span>
                                })}
                            </div>

                            <button
                                type="button"
                                class="btn btn-secondary"
                                on:click=go_to_next_day
                                disabled=move || !can_go_next() || loading.get()
                            >
                                "Next Day →"
                            </button>
                        </div>
                    </div>
                })
            } else {
                None
            }}

            // Show friendly empty state when no camp is selected
            {move || if camp_id.get() == 0 {
                let nav = nav_for_empty_state.clone();
                Some(view! {
                    <div class="card text-center py-16 bg-gradient-to-br from-slate-50 to-blue-50 border-2 border-dashed border-slate-300">
                        <div class="text-7xl mb-6">"📅"</div>
                        <h3 class="text-2xl font-bold text-slate-800 mb-3">"No Camp Selected"</h3>
                        <p class="text-lg text-slate-600 mb-8">
                            "Please select a camp to start planning meals"
                        </p>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| nav("/camps", Default::default())
                        >
                            "Go to Camps"
                        </button>
                    </div>
                })
            } else {
                None
            }}

            {move || error.get().map(|err| view! {
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                    {err}
                </div>
            })}

            {move || show_form.get().then(|| {
                view! {
                <div class="card">
                    <h3 class="text-xl font-semibold mb-4">
                        {move || if editing_meal_id.get().is_some() {
                            "Edit Meal"
                        } else {
                            "Plan a Meal"
                        }}
                    </h3>

                    <form on:submit=handle_submit class="space-y-4">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="form-label">"Meal Type" <span class="text-red-500">"*"</span></label>
                                <select
                                    class="form-input"
                                    prop:value=move || meal_type.get().as_str()
                                    on:change=move |ev| {
                                        if let Some(mt) = MealType::from_str(&event_target_value(&ev)) {
                                            set_meal_type.set(mt);
                                        }
                                    }
                                    disabled=move || editing_meal_id.get().is_some()
                                >
                                    <option value="breakfast">"Breakfast"</option>
                                    <option value="morning_snack">"Morning Snack"</option>
                                    <option value="lunch">"Lunch"</option>
                                    <option value="afternoon_snack">"Afternoon Snack"</option>
                                    <option value="dinner">"Dinner"</option>
                                </select>
                            </div>
                            <div>
                                <SearchableSelect
                                    options=recipes.into()
                                    selected_value=recipe_id.into()
                                    on_change=move |id| set_recipe_id.set(id)
                                    get_id=|recipe: &Recipe| recipe.id.to_string()
                                    get_display=|recipe: &Recipe| recipe.name.clone()
                                    placeholder="Search recipes..."
                                    label="Recipe"
                                    required=true
                                />
                            </div>
                        </div>
                        
                        <div class="border-t pt-4">
                            <h4 class="font-semibold mb-3">"Attendance (optional)"</h4>
                            <div class="grid grid-cols-3 gap-4">
                                <div>
                                    <label class="form-label">"Children"</label>
                                    <input
                                        type="number"
                                        class="form-input"
                                        prop:value=move || children.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                                set_children.set(val);
                                            }
                                        }
                                        min="0"
                                    />
                                </div>
                                <div>
                                    <label class="form-label">"Teens"</label>
                                    <input
                                        type="number"
                                        class="form-input"
                                        prop:value=move || teens.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                                set_teens.set(val);
                                            }
                                        }
                                        min="0"
                                    />
                                </div>
                                <div>
                                    <label class="form-label">"Adults"</label>
                                    <input
                                        type="number"
                                        class="form-input"
                                        prop:value=move || adults.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                                set_adults.set(val);
                                            }
                                        }
                                        min="0"
                                    />
                                </div>
                            </div>
                        </div>

                        <div class="flex gap-2">
                            <button
                                type="submit"
                                class="btn btn-primary"
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() {
                                    "Saving..."
                                } else if editing_meal_id.get().is_some() {
                                    "Update Meal"
                                } else {
                                    "Add Meal"
                                }}
                            </button>
                            <button type="button" class="btn btn-secondary" on:click=cancel_form disabled=move || loading.get()>
                                "Cancel"
                            </button>
                        </div>
                    </form>
                </div>
                }
            })}

            {move || if loading.get() && !show_form.get() {
                view! { <div class="text-center py-8">"Loading..."</div> }.into_any()
            } else if view_mode.get() == ViewMode::SingleDay {
                // Single day view
                if planned_meals.get().is_empty() {
                    view! {
                        <div class="card text-center text-slate-600">
                            "No meals planned for this date. Click 'Add Meal' to plan one."
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-3">
                            <For
                                each=move || planned_meals.get()
                                key=|meal| meal.planned_meal.id
                                let:meal
                            >
                                <div class="card">
                                    <div class="flex justify-between items-start gap-4">
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm font-semibold text-blue-600">
                                                    {format_meal_type(&meal.planned_meal.meal_type)}
                                                </span>
                                            </div>
                                            <h3 class="text-lg font-semibold mt-1">{meal.recipe_name.clone()}</h3>
                                            {meal.attendance.clone().map(|att| view! {
                                                <p class="text-sm text-slate-500 mt-2">
                                                    "Attendance: "
                                                    {att.children} " children, "
                                                    {att.teens} " teens, "
                                                    {att.adults} " adults"
                                                </p>
                                            })}
                                        </div>
                                        <div class="flex flex-col gap-2 shrink-0">
                                            <button
                                                class="btn btn-secondary text-sm whitespace-nowrap"
                                                on:click={
                                                    let meal_clone = meal.clone();
                                                    move |_| handle_edit_click(meal_clone.clone())
                                                }
                                                disabled=move || loading.get()
                                            >
                                                "✏️ Edit"
                                            </button>
                                            <button
                                                class="btn btn-danger text-sm whitespace-nowrap"
                                                on:click={
                                                    let id = meal.planned_meal.id;
                                                    move |_| handle_delete_click(id)
                                                }
                                                disabled=move || loading.get()
                                            >
                                                "🗑️ Delete"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </For>
                        </div>
                    }.into_any()
                }
            } else {
                // All days view
                let meals_map = multi_day_meals.get();
                if meals_map.is_empty() {
                    view! {
                        <div class="card text-center text-slate-600">
                            "No meals planned yet. Select 'Single Day' view to add meals."
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-6">
                            <For
                                each=move || {
                                    let meals_map = multi_day_meals.get();
                                    let mut dates: Vec<String> = meals_map.keys().cloned().collect();
                                    dates.sort();
                                    dates
                                }
                                key=|date| date.clone()
                                let:date
                            >
                                {
                                    let meals_for_date = multi_day_meals.get().get(&date).cloned().unwrap_or_default();
                                    let date_str = date.clone();
                                    let camp_data = camp.get();

                                    // Calculate day number
                                    let day_info = if let Some(c) = camp_data {
                                        if let Ok(parsed_date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                                            if parsed_date >= c.start_date && parsed_date <= c.end_date {
                                                let day_num = (parsed_date - c.start_date).num_days() + 1;
                                                Some(format!("Day {}", day_num))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    view! {
                                        <div class="card border-2 border-indigo-200">
                                            <div class="flex justify-between items-center mb-4">
                                                <div>
                                                    <h3 class="text-xl font-bold text-indigo-700">{date_str}</h3>
                                                    {day_info.map(|info| view! {
                                                        <span class="text-sm text-indigo-600 font-medium">{info}</span>
                                                    })}
                                                </div>
                                            </div>
                                            {if meals_for_date.is_empty() {
                                                view! {
                                                    <p class="text-slate-500 italic">"No meals planned for this day"</p>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-2">
                                                        <For
                                                            each=move || meals_for_date.clone()
                                                            key=|meal| meal.planned_meal.id
                                                            let:meal
                                                        >
                                                            <div class="bg-slate-50 rounded-lg p-4">
                                                                <div class="flex justify-between items-start gap-3">
                                                                    <div class="flex-1 min-w-0">
                                                                        <div class="flex items-center gap-2">
                                                                            <span class="text-sm font-semibold text-blue-600">
                                                                                {format_meal_type(&meal.planned_meal.meal_type)}
                                                                            </span>
                                                                        </div>
                                                                        <h4 class="font-semibold mt-1">{meal.recipe_name.clone()}</h4>
                                                                        {meal.attendance.clone().map(|att| view! {
                                                                            <p class="text-xs text-slate-500 mt-1">
                                                                                "Attendance: "
                                                                                {att.children} " children, "
                                                                                {att.teens} " teens, "
                                                                                {att.adults} " adults"
                                                                            </p>
                                                                        })}
                                                                    </div>
                                                                    <div class="flex flex-col gap-2 shrink-0">
                                                                        <button
                                                                            class="btn btn-secondary text-xs px-3 py-1 whitespace-nowrap"
                                                                            on:click={
                                                                                let meal_clone = meal.clone();
                                                                                move |_| {
                                                                                    // Switch to single day view and edit
                                                                                    set_view_mode.set(ViewMode::SingleDay);
                                                                                    handle_edit_click(meal_clone.clone());
                                                                                }
                                                                            }
                                                                            disabled=move || loading.get()
                                                                        >
                                                                            "✏️ Edit"
                                                                        </button>
                                                                        <button
                                                                            class="btn btn-danger text-xs px-3 py-1 whitespace-nowrap"
                                                                            on:click={
                                                                                let id = meal.planned_meal.id;
                                                                                move |_| handle_delete_click(id)
                                                                            }
                                                                            disabled=move || loading.get()
                                                                        >
                                                                            "🗑️ Delete"
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </For>
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>
                                    }
                                }
                            </For>
                        </div>
                    }.into_any()
                }
            }}

            // Delete Confirmation Modal
            <ConfirmModal
                show=show_delete_modal.into()
                on_confirm=confirm_delete
                on_cancel=cancel_delete
                title="Delete Meal".to_string()
                message="Are you sure you want to delete this meal? This action cannot be undone.".to_string()
                confirm_text="Delete".to_string()
                cancel_text="Cancel".to_string()
                variant="danger".to_string()
            />
        </div>
    }
}
