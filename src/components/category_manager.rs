use crate::models::Category;
use crate::server_functions::categories::{
    create_category, delete_category, get_categories, update_category,
};
use crate::components::{ConfirmModal, toast_success, toast_error};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn CategoryManager() -> impl IntoView {
    let (categories, set_categories) = signal(Vec::<Category>::new());
    let (show_form, set_show_form) = signal(false);
    let (editing_id, set_editing_id) = signal(None::<i64>);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Modal state
    let (show_delete_modal, set_show_delete_modal) = signal(false);
    let (delete_id, set_delete_id) = signal(0i64);

    // Form fields
    let (name, set_name) = signal(String::new());
    let (sort_order, set_sort_order) = signal(0);

    // Load categories on mount
    let load_data = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match get_categories().await {
                Ok(data) => set_categories.set(data),
                Err(e) => set_error.set(Some(format!("Failed to load categories: {}", e))),
            }

            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        load_data();
    });

    let reset_form = move || {
        set_name.set(String::new());
        set_sort_order.set(0);
        set_editing_id.set(None);
        set_error.set(None);
    };

    let start_add = move |_| {
        reset_form();
        set_show_form.set(true);
    };

    let start_edit = move |category: Category| {
        set_name.set(category.name);
        set_sort_order.set(category.sort_order);
        set_editing_id.set(Some(category.id));
        set_show_form.set(true);
    };

    let cancel_form = move |_| {
        set_show_form.set(false);
        set_error.set(None);
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let name_val = name.get();
        let sort_order_val = sort_order.get();
        let editing_id_val = editing_id.get();

        if name_val.is_empty() {
            toast_error("Please fill in all required fields");
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            let result = if let Some(id) = editing_id_val {
                update_category(id, Some(name_val), Some(sort_order_val)).await
            } else {
                create_category(name_val, sort_order_val).await
            };

            match result {
                Ok(_) => {
                    if editing_id_val.is_some() {
                        toast_success("Category updated successfully!");
                    } else {
                        toast_success("Category created successfully!");
                    }
                    reset_form();
                    set_show_form.set(false);
                    load_data();
                }
                Err(e) => toast_error(&format!("Failed to save category: {}", e)),
            }

            set_loading.set(false);
        });
    };

    let handle_delete_click = move |id: i64| {
        set_delete_id.set(id);
        set_show_delete_modal.set(true);
    };

    let confirm_delete = move || {
        let id = delete_id.get();
        set_show_delete_modal.set(false);

        spawn_local(async move {
            set_loading.set(true);
            match delete_category(id).await {
                Ok(_) => {
                    toast_success("Category deleted successfully!");
                    load_data();
                },
                Err(e) => toast_error(&format!("Failed to delete: {}", e)),
            }
            set_loading.set(false);
        });
    };

    let cancel_delete = move || {
        set_show_delete_modal.set(false);
    };

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h3 class="text-2xl font-bold text-gradient flex items-center gap-2">
                    <span class="text-3xl">"📁"</span>
                    "Categories"
                </h3>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=start_add
                    disabled=move || loading.get()
                >
                    <span class="mr-1">"+"</span>
                    " Add Category"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert-error">
                    <span class="font-semibold mr-2">"⚠️ Error:"</span>
                    {err}
                </div>
            })}

            {move || show_form.get().then(|| view! {
                <div class="card border-2 border-blue-200">
                    <h3 class="text-2xl font-bold mb-6 text-gradient flex items-center gap-2">
                        <span>{move || if editing_id.get().is_some() { "✏️" } else { "✨" }}</span>
                        {move || if editing_id.get().is_some() { "Edit Category" } else { "New Category" }}
                    </h3>
                    <form on:submit=handle_submit class="space-y-4">
                        <div>
                            <label class="form-label">"Name" <span class="text-red-500">"*"</span></label>
                            <input
                                type="text"
                                class="form-input"
                                prop:value=move || name.get()
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                required
                            />
                        </div>
                        <div>
                            <label class="form-label">"Sort Order"</label>
                            <input
                                type="number"
                                class="form-input"
                                prop:value=move || sort_order.get()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                        set_sort_order.set(val);
                                    }
                                }
                            />
                        </div>
                        <div class="flex gap-2">
                            <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                                {move || if loading.get() { "Saving..." } else { "Save" }}
                            </button>
                            <button type="button" class="btn btn-secondary" on:click=cancel_form disabled=move || loading.get()>
                                "Cancel"
                            </button>
                        </div>
                    </form>
                </div>
            })}

            {move || if loading.get() && !show_form.get() {
                view! {
                    <div class="card text-center py-12">
                        <div class="spinner mx-auto mb-4"></div>
                        <p class="text-slate-600">"Loading categories..."</p>
                    </div>
                }.into_any()
            } else if categories.get().is_empty() {
                view! {
                    <div class="card text-center py-16 bg-gradient-to-br from-slate-50 to-blue-50 border-2 border-dashed border-slate-300">
                        <div class="text-7xl mb-6">"📁"</div>
                        <h3 class="text-2xl font-bold text-slate-800 mb-3">"No categories yet"</h3>
                        <p class="text-lg text-slate-600 mb-8">"Get started by creating your first category"</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <For
                            each=move || categories.get()
                            key=|cat| cat.id
                            let:category
                        >
                            <div class="card group">
                                <div class="flex items-start justify-between mb-3">
                                    <div class="text-3xl group-hover:scale-110 transition-transform duration-200">"📁"</div>
                                    <span class="badge badge-secondary text-xs">"Sort: " {category.sort_order}</span>
                                </div>
                                <h3 class="text-xl font-bold text-slate-800 mb-4">{category.name.clone()}</h3>
                                <div class="mt-auto flex gap-2">
                                    <button
                                        class="btn btn-secondary text-sm flex-1"
                                        on:click={
                                            let cat = category.clone();
                                            move |_| start_edit(cat.clone())
                                        }
                                        disabled=move || loading.get()
                                    >
                                        "✏️ Edit"
                                    </button>
                                    <button
                                        class="btn btn-danger text-sm"
                                        on:click={
                                            let id = category.id;
                                            move |_| handle_delete_click(id)
                                        }
                                        disabled=move || loading.get()
                                    >
                                        "🗑️"
                                    </button>
                                </div>
                            </div>
                        </For>
                    </div>
                }.into_any()
            }}

            <ConfirmModal
                show=show_delete_modal.into()
                on_confirm=confirm_delete
                on_cancel=cancel_delete
                title="Delete Category".to_string()
                message="Are you sure? This action cannot be undone.".to_string()
                confirm_text="Delete".to_string()
                cancel_text="Cancel".to_string()
                variant="danger".to_string()
            />
        </div>
    }
}
