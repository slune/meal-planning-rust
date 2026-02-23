use leptos::prelude::*;
use crate::components::MealPlanner as MealPlannerComponent;

#[component]
pub fn MealPlannerPage() -> impl IntoView {
    view! {
        <div>
            <MealPlannerComponent/>
        </div>
    }
}
