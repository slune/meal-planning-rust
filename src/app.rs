use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::nav::NavBar;
use crate::components::{ToastProvider, icon};
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <ToastProvider>
            <Router>
                <NavBar/>
                <main class="container mx-auto px-4 py-6 md:px-6 lg:px-8 mb-16">
                <Routes fallback=|| view! {
                    <div class="empty-state">
                        <span class="icon-badge icon-badge-sky mx-auto mb-4" aria-hidden="true">{icon("search")}</span>
                        <p class="text-2xl font-semibold text-slate-950 mb-2">"Page not found"</p>
                        <p class="text-sm text-slate-600 mb-6">"The page you're looking for doesn't exist."</p>
                        <a href="/" class="btn btn-primary">"Go Home"</a>
                    </div>
                }>
                    <Route path=path!("login") view=LoginPage/>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("camps") view=CampsPage/>
                    <Route path=path!("recipes") view=RecipesPage/>
                    <Route path=path!("ingredients") view=IngredientsPage/>
                    <Route path=path!("planner") view=MealPlannerPage/>
                    <Route path=path!("planner/:camp_id") view=MealPlannerPage/>
                    <Route path=path!("reports") view=ReportsPage/>
                    <Route path=path!("settings") view=SettingsPage/>
                </Routes>
            </main>
            </Router>
        </ToastProvider>
    }
}

#[component]
pub fn Shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="stylesheet" href="/style/main.css"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
