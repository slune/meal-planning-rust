use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

use crate::components::icon;
use crate::server_functions::auth::logout;

fn nav_link_class(active: bool) -> &'static str {
    if active {
        "inline-flex items-center gap-2 rounded-md border border-sky-200 bg-sky-50 px-3 py-2 text-sm font-semibold text-sky-800 no-underline"
    } else {
        "inline-flex items-center gap-2 rounded-md border border-transparent px-3 py-2 text-sm font-semibold text-slate-600 no-underline hover:bg-slate-100 hover:text-slate-950"
    }
}

#[component]
pub fn NavBar() -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();
    let navigate = use_navigate();

    let logout_action = Action::new(|_: &()| async { logout().await });

    Effect::new(move |_| {
        if logout_action.value().get().is_some() {
            navigate("/login", Default::default());
        }
    });

    // Helper to check if path matches
    let is_active = move |path: &str| {
        let current = pathname();
        current == path || (path != "/" && current.starts_with(path))
    };

    view! {
        <nav class="mb-6 border-b border-slate-200 bg-white">
            <div class="container mx-auto px-4 py-3 md:px-6 lg:px-8">
                <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                    <div>
                        <a href="/" class="inline-flex items-center gap-3 text-lg font-semibold tracking-normal text-slate-950 no-underline hover:text-sky-800">
                            <span class="app-mark" aria-hidden="true">
                                {icon("kitchen")}
                            </span>
                            "Boy Scout Meal Planner"
                        </a>
                    </div>
                    <div class="flex flex-wrap gap-1" role="navigation" aria-label="Main navigation">
                        <a
                            href="/camps"
                            class=move || nav_link_class(is_active("/camps"))
                            aria-current=move || if is_active("/camps") { Some("page") } else { None }
                        >
                            <span class="nav-icon" aria-hidden="true">{icon("camp")}</span>
                            "Camps"
                        </a>
                        <a
                            href="/planner"
                            class=move || nav_link_class(is_active("/planner"))
                            aria-current=move || if is_active("/planner") { Some("page") } else { None }
                        >
                            <span class="nav-icon" aria-hidden="true">{icon("planner")}</span>
                            "Planner"
                        </a>
                        <a
                            href="/recipes"
                            class=move || nav_link_class(is_active("/recipes"))
                            aria-current=move || if is_active("/recipes") { Some("page") } else { None }
                        >
                            <span class="nav-icon" aria-hidden="true">{icon("recipes")}</span>
                            "Recipes"
                        </a>
                        <a
                            href="/ingredients"
                            class=move || nav_link_class(is_active("/ingredients"))
                            aria-current=move || if is_active("/ingredients") { Some("page") } else { None }
                        >
                            <span class="nav-icon" aria-hidden="true">{icon("ingredients")}</span>
                            "Ingredients"
                        </a>
                        <a
                            href="/reports"
                            class=move || nav_link_class(is_active("/reports"))
                            aria-current=move || if is_active("/reports") { Some("page") } else { None }
                        >
                            <span class="nav-icon" aria-hidden="true">{icon("reports")}</span>
                            "Reports"
                        </a>
                        <button
                            class="rounded-md px-3 py-2 text-sm font-semibold text-slate-500 hover:bg-slate-100 hover:text-slate-950"
                            on:click=move |_| { logout_action.dispatch(()); }
                        >
                            "Logout"
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    }
}
