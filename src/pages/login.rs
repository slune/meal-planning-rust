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
        <div class="fixed inset-0 z-50 bg-gradient-to-br from-indigo-100 to-purple-100 flex items-center justify-center">
            <div class="card w-full max-w-md">
                <div class="text-center mb-8">
                    <div class="text-7xl mb-4">"🏕️"</div>
                    <h1 class="text-3xl font-bold text-slate-800">"Boy Scout Meal Planner"</h1>
                    <p class="text-slate-600 mt-2">"Sign in to continue"</p>
                </div>
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    login_action.dispatch(password.get());
                }>
                    <div class="mb-4">
                        <label class="block text-sm font-medium text-slate-700 mb-2">"Password"</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
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
                        class="w-full px-4 py-2 bg-indigo-600 text-white font-semibold rounded-lg hover:bg-indigo-700 transition-colors disabled:opacity-50"
                        disabled=move || login_action.pending().get()
                    >
                        {move || if login_action.pending().get() { "Signing in…" } else { "Sign In" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
