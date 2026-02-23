use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::{SearchableSelect, LoadingSpinner, toast_success, toast_error};
use crate::server_functions::{
    get_camps, get_camp, generate_shopping_list, generate_meal_schedule, generate_attendance_summary
};
use crate::models::{Camp, MealType, ShoppingListItem, MealScheduleItem, AttendanceSummary};

#[derive(Clone, Copy, PartialEq)]
enum ReportType {
    ShoppingList,
    MealSchedule,
    AttendanceSummary,
}

impl ReportType {
    fn as_str(&self) -> &'static str {
        match self {
            ReportType::ShoppingList => "shopping_list",
            ReportType::MealSchedule => "meal_schedule",
            ReportType::AttendanceSummary => "attendance_summary",
        }
    }

    fn display(&self) -> &'static str {
        match self {
            ReportType::ShoppingList => "Shopping List",
            ReportType::MealSchedule => "Meal Schedule",
            ReportType::AttendanceSummary => "Attendance Summary",
        }
    }
}

#[component]
pub fn ReportGenerator() -> impl IntoView {
    // State
    let (camps, set_camps) = signal(Vec::<Camp>::new());
    let (selected_camp_id, set_selected_camp_id) = signal(0i64);
    let (selected_camp, set_selected_camp) = signal(Option::<Camp>::None);
    let (report_type, set_report_type) = signal(ReportType::ShoppingList);
    let (start_date, set_start_date) = signal(String::new());
    let (end_date, set_end_date) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);

    // Report data
    let (shopping_list, set_shopping_list) = signal(Vec::<ShoppingListItem>::new());
    let (meal_schedule, set_meal_schedule) = signal(Vec::<MealScheduleItem>::new());
    let (attendance_summary, set_attendance_summary) = signal(Vec::<AttendanceSummary>::new());
    let (report_generated, set_report_generated) = signal(false);

    // Load camps on mount
    let load_camps = Action::new(move |_: &()| async move {
        match get_camps().await {
            Ok(camps_list) => {
                set_camps.set(camps_list);
            }
            Err(e) => {
                toast_error(&format!("Failed to load camps: {}", e));
            }
        }
    });

    load_camps.dispatch(());

    // Watch for camp selection changes
    Effect::new(move || {
        let camp_id = selected_camp_id.get();
        if camp_id > 0 {
            spawn_local(async move {
                match get_camp(camp_id).await {
                    Ok(camp) => {
                        // Set default date range to camp dates
                        set_start_date.set(camp.start_date.format("%Y-%m-%d").to_string());
                        set_end_date.set(camp.end_date.format("%Y-%m-%d").to_string());
                        set_selected_camp.set(Some(camp));
                    }
                    Err(e) => {
                        toast_error(&format!("Failed to load camp details: {}", e));
                    }
                }
            });
        } else {
            set_selected_camp.set(None);
            set_start_date.set(String::new());
            set_end_date.set(String::new());
        }
    });

    let handle_generate = move |_| {
        if selected_camp_id.get() == 0 {
            toast_error("Please select a camp");
            return;
        }

        set_is_loading.set(true);
        set_report_generated.set(false);

        let camp_id = selected_camp_id.get();
        let current_report_type = report_type.get();
        let start = start_date.get();
        let end = end_date.get();

        spawn_local(async move {
            match current_report_type {
                ReportType::ShoppingList => {
                    if start.is_empty() || end.is_empty() {
                        toast_error("Please select start and end dates");
                        set_is_loading.set(false);
                        return;
                    }

                    match generate_shopping_list(camp_id, start, end).await {
                        Ok(items) => {
                            set_shopping_list.set(items);
                            set_report_generated.set(true);
                            toast_success("Shopping list generated successfully!");
                        }
                        Err(e) => {
                            toast_error(&format!("Failed to generate shopping list: {}", e));
                        }
                    }
                }
                ReportType::MealSchedule => {
                    match generate_meal_schedule(camp_id).await {
                        Ok(items) => {
                            set_meal_schedule.set(items);
                            set_report_generated.set(true);
                            toast_success("Meal schedule generated successfully!");
                        }
                        Err(e) => {
                            toast_error(&format!("Failed to generate meal schedule: {}", e));
                        }
                    }
                }
                ReportType::AttendanceSummary => {
                    match generate_attendance_summary(camp_id).await {
                        Ok(items) => {
                            set_attendance_summary.set(items);
                            set_report_generated.set(true);
                            toast_success("Attendance summary generated successfully!");
                        }
                        Err(e) => {
                            toast_error(&format!("Failed to generate attendance summary: {}", e));
                        }
                    }
                }
            }
            set_is_loading.set(false);
        });
    };

    let handle_print = move |_| {
        let _ = window().print();
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h2 class="text-3xl font-bold text-slate-800">"Reports"</h2>
            </div>

            // Report Configuration Card
            <div class="card no-print">
                <h3 class="text-xl font-bold text-slate-800 mb-6">"Generate Report"</h3>

                <div class="space-y-6">
                    // Camp Selector
                    <div>
                        <SearchableSelect
                            options=camps.into()
                            selected_value=selected_camp_id.into()
                            on_change=move |id| set_selected_camp_id.set(id)
                            get_id=|camp: &Camp| camp.id.to_string()
                            get_display=|camp: &Camp| camp.name.clone()
                            placeholder="Select a camp..."
                        />
                    </div>

                    // Report Type Selector
                    <div>
                        <label class="form-label">"Report Type" <span class="text-red-500">"*"</span></label>
                        <select
                            class="form-input"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let new_type = match value.as_str() {
                                    "meal_schedule" => ReportType::MealSchedule,
                                    "attendance_summary" => ReportType::AttendanceSummary,
                                    _ => ReportType::ShoppingList,
                                };
                                set_report_type.set(new_type);
                            }
                        >
                            <option value="shopping_list">"Shopping List"</option>
                            <option value="meal_schedule">"Meal Schedule"</option>
                            <option value="attendance_summary">"Attendance Summary"</option>
                        </select>
                    </div>

                    // Date Range (only for shopping list)
                    <Show
                        when=move || report_type.get() == ReportType::ShoppingList
                        fallback=|| ()
                    >
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="form-label">"Start Date" <span class="text-red-500">"*"</span></label>
                                <input
                                    type="date"
                                    class="form-input"
                                    value=move || start_date.get()
                                    on:input=move |ev| set_start_date.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="form-label">"End Date" <span class="text-red-500">"*"</span></label>
                                <input
                                    type="date"
                                    class="form-input"
                                    value=move || end_date.get()
                                    on:input=move |ev| set_end_date.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </Show>

                    // Generate Button
                    <div class="flex gap-3">
                        <button
                            class="btn btn-primary"
                            on:click=handle_generate
                            disabled=move || is_loading.get() || selected_camp_id.get() == 0
                        >
                            {move || if is_loading.get() { "Generating..." } else { "Generate Report" }}
                        </button>

                        <Show
                            when=move || report_generated.get()
                            fallback=|| ()
                        >
                            <button
                                class="btn btn-secondary"
                                on:click=handle_print
                            >
                                "🖨️ Print"
                            </button>
                        </Show>
                    </div>
                </div>
            </div>

            // Loading State
            <LoadingSpinner show=is_loading.get() />

            // Report Display
            <Show
                when=move || report_generated.get() && !is_loading.get()
                fallback=|| ()
            >
                <div class="card print-section">
                    {move || match report_type.get() {
                        ReportType::ShoppingList => view! {
                            <ShoppingListReport
                                camp=selected_camp.get()
                                items=shopping_list.get()
                                start_date=start_date.get()
                                end_date=end_date.get()
                            />
                        }.into_any(),
                        ReportType::MealSchedule => view! {
                            <MealScheduleReport
                                camp=selected_camp.get()
                                items=meal_schedule.get()
                            />
                        }.into_any(),
                        ReportType::AttendanceSummary => view! {
                            <AttendanceSummaryReport
                                camp=selected_camp.get()
                                items=attendance_summary.get()
                            />
                        }.into_any(),
                    }}
                </div>
            </Show>
        </div>

        // Print Styles
        <style>
            "@media print {
                .no-print { display: none; }
                .print-section { page-break-inside: avoid; }
                body { font-size: 12pt; }
                h2 { font-size: 18pt; }
                h3 { font-size: 16pt; }
                table { width: 100%; border-collapse: collapse; }
                th, td { border: 1px solid #000; padding: 8px; text-align: left; }
            }"
        </style>
    }
}

#[component]
fn ShoppingListReport(
    camp: Option<Camp>,
    items: Vec<ShoppingListItem>,
    start_date: String,
    end_date: String,
) -> impl IntoView {
    let camp_name = camp.as_ref().map(|c| c.name.clone()).unwrap_or_default();

    // Group items by category
    let mut grouped_items: Vec<(String, Vec<ShoppingListItem>)> = Vec::new();
    let mut current_category = String::new();
    let mut current_items = Vec::new();

    for item in items {
        if item.category_name != current_category {
            if !current_items.is_empty() {
                grouped_items.push((current_category.clone(), current_items.clone()));
                current_items.clear();
            }
            current_category = item.category_name.clone();
        }
        current_items.push(item);
    }

    if !current_items.is_empty() {
        grouped_items.push((current_category, current_items));
    }

    view! {
        <div>
            <h2 class="text-2xl font-bold text-slate-800 mb-2">"Shopping List"</h2>
            <p class="text-slate-600 mb-6">
                {camp_name} " • " {start_date.clone()} " to " {end_date}
            </p>

            {grouped_items.into_iter().map(|(category, items): (String, Vec<ShoppingListItem>)| {
                view! {
                    <div class="mb-8">
                        <h3 class="text-xl font-bold text-slate-700 mb-4 border-b-2 border-slate-300 pb-2">
                            {category}
                        </h3>
                        <table class="w-full">
                            <thead>
                                <tr class="bg-slate-100">
                                    <th class="text-left p-3">"Ingredient"</th>
                                    <th class="text-right p-3">"Quantity"</th>
                                    <th class="text-left p-3">"Unit"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {items.into_iter().map(|item| {
                                    view! {
                                        <tr class="border-t border-slate-200">
                                            <td class="p-3">{item.ingredient_name}</td>
                                            <td class="text-right p-3">{format!("{:.2}", item.total_quantity)}</td>
                                            <td class="p-3">{item.unit}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn MealScheduleReport(
    camp: Option<Camp>,
    items: Vec<MealScheduleItem>,
) -> impl IntoView {
    let camp_name = camp.as_ref().map(|c| c.name.clone()).unwrap_or_default();

    view! {
        <div>
            <h2 class="text-2xl font-bold text-slate-800 mb-2">"Meal Schedule"</h2>
            <p class="text-slate-600 mb-6">{camp_name}</p>

            <table class="w-full">
                <thead>
                    <tr class="bg-slate-100">
                        <th class="text-left p-3">"Date"</th>
                        <th class="text-left p-3">"Meal Type"</th>
                        <th class="text-left p-3">"Recipe"</th>
                        <th class="text-right p-3">"Children"</th>
                        <th class="text-right p-3">"Teens"</th>
                        <th class="text-right p-3">"Adults"</th>
                    </tr>
                </thead>
                <tbody>
                    {items.into_iter().map(|item| {
                        let meal_type_display = MealType::from_str(&item.meal_type)
                            .map(|mt| format_meal_type(&mt))
                            .unwrap_or(item.meal_type.clone());

                        view! {
                            <tr class="border-t border-slate-200">
                                <td class="p-3">{item.date.format("%Y-%m-%d").to_string()}</td>
                                <td class="p-3">{meal_type_display}</td>
                                <td class="p-3">{item.recipe_name}</td>
                                <td class="text-right p-3">{item.children}</td>
                                <td class="text-right p-3">{item.teens}</td>
                                <td class="text-right p-3">{item.adults}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn AttendanceSummaryReport(
    camp: Option<Camp>,
    items: Vec<AttendanceSummary>,
) -> impl IntoView {
    let camp_name = camp.as_ref().map(|c| c.name.clone()).unwrap_or_default();

    view! {
        <div>
            <h2 class="text-2xl font-bold text-slate-800 mb-2">"Attendance Summary"</h2>
            <p class="text-slate-600 mb-6">{camp_name}</p>

            <table class="w-full">
                <thead>
                    <tr class="bg-slate-100">
                        <th class="text-left p-3">"Date"</th>
                        <th class="text-left p-3">"Meal Type"</th>
                        <th class="text-right p-3">"Children"</th>
                        <th class="text-right p-3">"Teens"</th>
                        <th class="text-right p-3">"Adults"</th>
                        <th class="text-right p-3">"Total"</th>
                    </tr>
                </thead>
                <tbody>
                    {items.into_iter().map(|item| {
                        let meal_type_display = MealType::from_str(&item.meal_type)
                            .map(|mt| format_meal_type(&mt))
                            .unwrap_or(item.meal_type.clone());

                        view! {
                            <tr class="border-t border-slate-200">
                                <td class="p-3">{item.date.format("%Y-%m-%d").to_string()}</td>
                                <td class="p-3">{meal_type_display}</td>
                                <td class="text-right p-3">{item.children}</td>
                                <td class="text-right p-3">{item.teens}</td>
                                <td class="text-right p-3">{item.adults}</td>
                                <td class="text-right p-3 font-bold">{item.total_people}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}

// Helper function to format meal type
fn format_meal_type(meal_type: &MealType) -> String {
    match meal_type {
        MealType::Breakfast => "Breakfast",
        MealType::MorningSnack => "Morning Snack",
        MealType::Lunch => "Lunch",
        MealType::AfternoonSnack => "Afternoon Snack",
        MealType::Dinner => "Dinner",
    }.to_string()
}
