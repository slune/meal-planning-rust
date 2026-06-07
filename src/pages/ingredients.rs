use crate::components::{CategoryManager, IngredientManager, icon};
use leptos::prelude::*;

#[component]
pub fn IngredientsPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("ingredients");

    view! {
        <div class="space-y-6">
            <div class="border-b border-slate-200">
                <div class="flex gap-6">
                    <button
                        class=move || if active_tab.get() == "ingredients" {
                            "tab-active"
                        } else {
                            "tab-inactive"
                        }
                        on:click=move |_| set_active_tab.set("ingredients")
                    >
                        <span class="inline-icon mr-2" aria-hidden="true">{icon("ingredients")}</span>
                        "Ingredients"
                    </button>
                    <button
                        class=move || if active_tab.get() == "categories" {
                            "tab-active"
                        } else {
                            "tab-inactive"
                        }
                        on:click=move |_| set_active_tab.set("categories")
                    >
                        <span class="inline-icon mr-2" aria-hidden="true">{icon("category")}</span>
                        "Categories"
                    </button>
                </div>
            </div>

            {move || match active_tab.get() {
                "categories" => view! { <CategoryManager/> }.into_any(),
                _ => view! { <IngredientManager/> }.into_any(),
            }}
        </div>
    }
}
