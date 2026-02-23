use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

use crate::server_functions::auth::logout;

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
        <nav class="bg-gradient-to-r from-indigo-600 via-indigo-700 to-purple-700 text-white shadow-2xl mb-12">
            <div class="container mx-auto">
                <div class="flex items-center justify-between">
                    <div class="flex flex-col py-4">
                        <a href="/" class="text-2xl font-bold hover:underline transition-all duration-200 flex items-center gap-3 no-underline text-white">
                            <span class="text-3xl">"🏕️"</span>
                            <span>"Boy Scout Meal Planner"</span>
                        </a>
                        <span class="text-xs text-blue-100 ml-12">"Build: 2026-01-19 01:15:00 UTC - v1.0.0-WORKING!"</span>
                    </div>
                    <div class="flex gap-1" role="navigation" aria-label="Main navigation">
                        <a
                            href="/camps"
                            class=move || if is_active("/camps") {
                                "px-5 py-3 rounded-xl bg-white/20 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline shadow-lg"
                            } else {
                                "px-5 py-3 rounded-xl hover:bg-white/10 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline hover:shadow-md"
                            }
                            aria-current=move || if is_active("/camps") { Some("page") } else { None }
                        >
                            <span class="mr-2">"🏕️"</span>
                            "Camps"
                        </a>
                        <a
                            href="/planner"
                            class=move || if is_active("/planner") {
                                "px-5 py-3 rounded-xl bg-white/20 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline shadow-lg"
                            } else {
                                "px-5 py-3 rounded-xl hover:bg-white/10 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline hover:shadow-md"
                            }
                            aria-current=move || if is_active("/planner") { Some("page") } else { None }
                        >
                            <span class="mr-2">"📅"</span>
                            "Planner"
                        </a>
                        <a
                            href="/recipes"
                            class=move || if is_active("/recipes") {
                                "px-5 py-3 rounded-xl bg-white/20 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline"
                            } else {
                                "px-5 py-3 rounded-xl hover:bg-white/10 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline"
                            }
                            aria-current=move || if is_active("/recipes") { Some("page") } else { None }
                        >
                            <span class="mr-2">"🍳"</span>
                            "Recipes"
                        </a>
                        <a
                            href="/ingredients"
                            class=move || if is_active("/ingredients") {
                                "px-5 py-3 rounded-xl bg-white/20 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline"
                            } else {
                                "px-5 py-3 rounded-xl hover:bg-white/10 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline"
                            }
                            aria-current=move || if is_active("/ingredients") { Some("page") } else { None }
                        >
                            <span class="mr-2">"🥕"</span>
                            "Ingredients"
                        </a>
                        <a
                            href="/reports"
                            class=move || if is_active("/reports") {
                                "px-5 py-3 rounded-xl bg-white/20 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline"
                            } else {
                                "px-5 py-3 rounded-xl hover:bg-white/10 backdrop-blur-sm transition-all duration-300 font-semibold text-white no-underline"
                            }
                            aria-current=move || if is_active("/reports") { Some("page") } else { None }
                        >
                            <span class="mr-2">"📊"</span>
                            "Reports"
                        </a>
                        <button
                            class="px-5 py-3 rounded-xl hover:bg-white/10 backdrop-blur-sm transition-all duration-300 font-semibold text-white"
                            on:click=move |_| { logout_action.dispatch(()); }
                        >
                            <span class="mr-2">"🔒"</span>
                            "Logout"
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    }
}
