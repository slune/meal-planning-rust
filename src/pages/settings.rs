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

            <div class="grid grid-cols-1 gap-6 lg:grid-cols-[2fr_1fr]">
                <section class="space-y-3">
                    <div>
                        <p class="page-kicker">"Planning"</p>
                        <h3 class="section-title mt-1">"Meal defaults"</h3>
                    </div>
                    <MealTypeManager/>
                </section>

                <section class="space-y-3">
                    <div>
                        <p class="page-kicker">"Data"</p>
                        <h3 class="section-title mt-1">"Exports"</h3>
                    </div>
                    <DatabaseExportPanel/>
                </section>
            </div>
        </div>
    }
}

#[component]
fn DatabaseExportPanel() -> impl IntoView {
    view! {
        <div class="card panel-accent panel-accent-emerald">
            <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between gap-3">
                    <h3 class="section-title flex items-center gap-2">
                        <span class="inline-icon text-emerald-700" aria-hidden="true">{icon("database")}</span>
                        "Database"
                    </h3>
                    <span class="status-chip status-chip-emerald">"SQLite"</span>
                </div>

                <p class="text-sm text-slate-600">
                    "Export a current snapshot of recipes, ingredients, camps, meal plans, and settings."
                </p>

                <a
                    class="btn btn-primary w-full"
                    href="/api/database/download"
                >
                    {icon("download")}
                    "Download database"
                </a>
            </div>
        </div>
    }
}
