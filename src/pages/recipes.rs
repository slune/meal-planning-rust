use crate::components::RecipeEditor;
use leptos::prelude::*;

#[component]
pub fn RecipesPage() -> impl IntoView {
    view! {
        <RecipeEditor/>
    }
}
