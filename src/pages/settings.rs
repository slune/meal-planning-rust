use crate::components::{MealTypeManager, icon};
use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="page-header">
                <div class="page-heading">
                    <span class="icon-badge icon-badge-sky" aria-hidden="true">{icon("settings")}</span>
                    <div>
                        <p class="page-kicker">"Configuration"</p>
                        <h2 class="page-title">"Settings"</h2>
                        <p class="page-subtitle">"Manage application-wide planning options."</p>
                    </div>
                </div>
            </div>

            <MealTypeManager/>
        </div>
    }
}
