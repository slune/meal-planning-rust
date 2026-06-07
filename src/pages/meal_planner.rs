use crate::components::MealPlanner as MealPlannerComponent;
use leptos::prelude::*;

#[component]
pub fn MealPlannerPage() -> impl IntoView {
    view! {
        <div>
            <MealPlannerComponent/>
        </div>
    }
}
