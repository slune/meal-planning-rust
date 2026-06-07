use crate::components::icon;
use leptos::prelude::*;

#[component]
fn IconBadge(name: &'static str, tone: &'static str) -> impl IntoView {
    let class = format!("icon-badge icon-badge-{}", tone);

    view! {
        <span class=class aria-hidden="true">
            {icon(name)}
        </span>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="space-y-8">
            <section class="workspace-hero">
                <div class="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_20rem] lg:items-center">
                    <div class="flex gap-4">
                        <IconBadge name="kitchen" tone="sky"/>
                        <div>
                            <p class="page-kicker">"Meal operations"</p>
                            <h1 class="mt-1 text-3xl font-semibold text-slate-950">"Camp kitchen workspace"</h1>
                            <p class="mt-3 max-w-3xl text-sm text-slate-700">
                                "Plan camp menus from recipe setup through shopping, kitchen prep, and printable reports."
                            </p>
                            <div class="mt-5 flex flex-wrap gap-2">
                                <span class="status-chip status-chip-sky">"Meal schedule"</span>
                                <span class="status-chip status-chip-emerald">"Ingredient totals"</span>
                                <span class="status-chip status-chip-amber">"PDF exports"</span>
                            </div>
                        </div>
                    </div>

                    <div class="workspace-side-panel">
                        <p class="page-kicker">"Fast path"</p>
                        <h2 class="section-title mt-1">"Start with a camp"</h2>
                        <p class="mt-2 text-sm text-slate-600">
                            "Select a camp, fill the meal schedule, then generate the kitchen packet."
                        </p>
                        <div class="mt-4 flex flex-col gap-2">
                            <a href="/planner" class="btn btn-primary">"Open planner"</a>
                            <a href="/reports" class="btn btn-secondary">"Generate reports"</a>
                        </div>
                    </div>
                </div>
            </section>

            <section class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-5">
                <a href="/camps" class="home-action home-action-sky">
                    <IconBadge name="camp" tone="sky"/>
                    <p class="page-kicker mt-4">"Set up"</p>
                    <h2 class="section-title mt-2">"Camps"</h2>
                    <p class="mt-2 text-sm text-slate-600">"Dates, locations, notes, and default attendance."</p>
                </a>

                <a href="/ingredients" class="home-action home-action-emerald">
                    <IconBadge name="ingredients" tone="emerald"/>
                    <p class="page-kicker mt-4">"Catalog"</p>
                    <h2 class="section-title mt-2">"Ingredients"</h2>
                    <p class="mt-2 text-sm text-slate-600">"Ingredient names, units, and categories."</p>
                </a>

                <a href="/recipes" class="home-action home-action-amber">
                    <IconBadge name="recipes" tone="amber"/>
                    <p class="page-kicker mt-4">"Library"</p>
                    <h2 class="section-title mt-2">"Recipes"</h2>
                    <p class="mt-2 text-sm text-slate-600">"Base servings, instructions, and scaling rules."</p>
                </a>

                <a href="/planner" class="home-action home-action-rose">
                    <IconBadge name="planner" tone="rose"/>
                    <p class="page-kicker mt-4">"Schedule"</p>
                    <h2 class="section-title mt-2">"Meal planner"</h2>
                    <p class="mt-2 text-sm text-slate-600">"Assign recipes to each camp day and meal."</p>
                </a>

                <a href="/reports" class="home-action home-action-violet">
                    <IconBadge name="reports" tone="violet"/>
                    <p class="page-kicker mt-4">"Export"</p>
                    <h2 class="section-title mt-2">"Reports"</h2>
                    <p class="mt-2 text-sm text-slate-600">"Shopping lists, attendance, schedules, and PDFs."</p>
                </a>
            </section>

            <section class="grid grid-cols-1 gap-4 lg:grid-cols-[2fr_1fr]">
                <div class="card">
                    <div class="mb-5">
                        <p class="page-kicker">"Workflow"</p>
                        <h2 class="section-title mt-1">"Recommended setup order"</h2>
                    </div>
                    <ol class="grid gap-3 text-sm text-slate-700 md:grid-cols-2">
                        <li class="rounded-md border-l-4 border-emerald-500 bg-emerald-50/60 p-3">
                            <span class="font-semibold text-slate-950">"1. Categories and ingredients"</span>
                            <p class="mt-1 text-sm">"Create the units and categories recipes will use."</p>
                        </li>
                        <li class="rounded-md border-l-4 border-amber-500 bg-amber-50/60 p-3">
                            <span class="font-semibold text-slate-950">"2. Recipes"</span>
                            <p class="mt-1 text-sm">"Add base servings and per-person multipliers."</p>
                        </li>
                        <li class="rounded-md border-l-4 border-sky-500 bg-sky-50/60 p-3">
                            <span class="font-semibold text-slate-950">"3. Camps"</span>
                            <p class="mt-1 text-sm">"Define camp dates and attendance defaults."</p>
                        </li>
                        <li class="rounded-md border-l-4 border-rose-500 bg-rose-50/60 p-3">
                            <span class="font-semibold text-slate-950">"4. Planner and reports"</span>
                            <p class="mt-1 text-sm">"Schedule meals, then export kitchen and shopping PDFs."</p>
                        </li>
                    </ol>
                </div>

                <div class="card">
                    <p class="page-kicker">"Kitchen packet"</p>
                    <h2 class="section-title mt-1">"Reports now export clean PDFs"</h2>
                    <p class="mt-2 text-sm text-slate-600">
                        "Use the reports page for paged PDFs that exclude the app chrome."
                    </p>
                    <div class="mt-5 flex flex-col gap-2">
                        <a href="/reports" class="btn btn-secondary">"Open reports"</a>
                        <a href="/ingredients" class="btn btn-secondary">"Review ingredients"</a>
                    </div>
                </div>
            </section>
        </div>
    }
}
