use crate::components::icon;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::server_functions::auth::login;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();

    let login_action = Action::new(|password: &String| {
        let password = password.clone();
        async move { login(password).await }
    });

    Effect::new(move |_| {
        if let Some(result) = login_action.value().get() {
            match result {
                Ok(true) => {
                    navigate("/", Default::default());
                }
                Ok(false) => {
                    set_error.set(Some("Incorrect password.".to_string()));
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                }
            }
        }
    });

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-100 px-4">
            <div class="card w-full max-w-md">
                <div class="mb-8">
                    <span class="app-mark mb-4" aria-hidden="true">{icon("kitchen")}</span>
                    <p class="page-kicker">"Protected workspace"</p>
                    <h1 class="mt-1 text-2xl font-semibold text-slate-950">"Boy Scout Meal Planner"</h1>
                    <p class="mt-2 text-sm text-slate-600">"Sign in to manage meals, recipes, and reports."</p>
                </div>
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    login_action.dispatch(password.get());
                }>
                    <div class="mb-4">
                        <label class="form-label">"Password"</label>
                        <input
                            type="password"
                            class="form-input"
                            placeholder="Enter password"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>
                    {move || error.get().map(|msg| view! {
                        <p class="text-red-600 text-sm mb-4">{msg}</p>
                    })}
                    <button
                        type="submit"
                        class="btn btn-primary w-full"
                        disabled=move || login_action.pending().get()
                    >
                        {move || if login_action.pending().get() { "Signing in…" } else { "Sign In" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
