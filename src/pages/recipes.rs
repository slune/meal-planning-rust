use leptos::prelude::*;
use crate::components::RecipeEditor;

#[component]
pub fn RecipesPage() -> impl IntoView {
    view! {
        <RecipeEditor/>
    }
}
