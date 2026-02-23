use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::nav::NavBar;
use crate::components::ToastProvider;
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <ToastProvider>
            <Router>
                <NavBar/>
                <main class="container mx-auto px-8 py-8 mb-20">
                <Routes fallback=|| view! {
                    <div class="card text-center py-16">
                        <div class="text-8xl mb-6">"🔍"</div>
                        <p class="text-3xl font-bold text-slate-800 mb-4">"Page not found"</p>
                        <p class="text-xl text-slate-600 mb-8">"The page you're looking for doesn't exist."</p>
                        <a href="/" class="btn btn-primary">"Go Home"</a>
                    </div>
                }>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("camps") view=CampsPage/>
                    <Route path=path!("recipes") view=RecipesPage/>
                    <Route path=path!("ingredients") view=IngredientsPage/>
                    <Route path=path!("planner") view=MealPlannerPage/>
                    <Route path=path!("planner/:camp_id") view=MealPlannerPage/>
                    <Route path=path!("reports") view=ReportsPage/>
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
