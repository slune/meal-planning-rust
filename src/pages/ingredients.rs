use leptos::prelude::*;
use crate::components::{IngredientManager, CategoryManager};

#[component]
pub fn IngredientsPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("ingredients");

    view! {
        <div class="space-y-6">
            <div class="card p-0 overflow-hidden">
                <div class="flex gap-1 bg-slate-50 p-2">
                    <button
                        class=move || if active_tab.get() == "ingredients" {
                            "flex-1 px-6 py-3 bg-white rounded-lg shadow-md border-2 border-blue-500 font-bold text-blue-700 transition-all duration-200"
                        } else {
                            "flex-1 px-6 py-3 text-slate-600 hover:bg-white/50 rounded-lg hover:shadow transition-all duration-200"
                        }
                        on:click=move |_| set_active_tab.set("ingredients")
                    >
                        <span class="mr-2">"🥕"</span>
                        "Ingredients"
                    </button>
                    <button
                        class=move || if active_tab.get() == "categories" {
                            "flex-1 px-6 py-3 bg-white rounded-lg shadow-md border-2 border-blue-500 font-bold text-blue-700 transition-all duration-200"
                        } else {
                            "flex-1 px-6 py-3 text-slate-600 hover:bg-white/50 rounded-lg hover:shadow transition-all duration-200"
                        }
                        on:click=move |_| set_active_tab.set("categories")
                    >
                        <span class="mr-2">"📁"</span>
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
