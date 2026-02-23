use leptos::prelude::*;

#[component]
pub fn Spinner(
    /// Size of the spinner (small, medium, large)
    #[prop(default = "medium".to_string())]
    size: String,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
    /// ARIA label for accessibility
    #[prop(default = "Loading".to_string())]
    aria_label: String,
) -> impl IntoView {
    let size_class = match size.as_str() {
        "small" => "spinner-sm",
        "large" => "spinner-lg",
        _ => "",
    };

    view! {
        <div
            class=format!("spinner {} {}", size_class, class)
            role="status"
            aria-label=aria_label
        >
            <span class="sr-only">{aria_label.clone()}</span>
        </div>
    }
}

#[component]
pub fn LoadingOverlay(
    /// Whether the overlay is visible
    show: Signal<bool>,
    /// Loading message
    #[prop(default = "Loading...".to_string())]
    message: String,
) -> impl IntoView {
    let message = message.clone();
    view! {
        <Show
            when=move || show.get()
            fallback=|| ()
        >
            <div class="loading-overlay">
                <div class="loading-overlay-content">
                    <Spinner size="large".to_string() />
                    <p class="loading-overlay-message">{message.clone()}</p>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn SkeletonLoader(
    /// Number of skeleton rows
    #[prop(default = 3)]
    rows: usize,
    /// Height of each row
    #[prop(default = "h-4".to_string())]
    height: String,
    /// Additional CSS classes
    #[prop(default = String::new())]
    class: String,
) -> impl IntoView {
    let height = height.clone();
    view! {
        <div class=format!("skeleton-loader {}", class) aria-busy="true" aria-label="Loading content">
            {(0..rows).map(move |i| {
                let width = if i == rows - 1 { "w-3/4" } else { "w-full" };
                view! {
                    <div class=format!("skeleton-row {} {}", height.clone(), width)></div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn CardSkeleton(
    /// Number of skeleton cards
    #[prop(default = 3)]
    count: usize,
) -> impl IntoView {
    view! {
        <div class="card-grid">
            {(0..count).map(|_| {
                view! {
                    <div class="card skeleton-card">
                        <div class="skeleton-row h-6 w-3/4 mb-4"></div>
                        <div class="skeleton-row h-4 w-full mb-2"></div>
                        <div class="skeleton-row h-4 w-5/6 mb-2"></div>
                        <div class="skeleton-row h-4 w-2/3"></div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn LoadingSpinner(
    /// Whether to show the spinner
    #[prop(default = true)]
    show: bool,
    /// Loading message
    #[prop(optional)]
    message: Option<String>,
) -> impl IntoView {
    let message = message.clone();
    view! {
        <Show
            when=move || show
            fallback=|| ()
        >
            <div class="flex flex-col items-center justify-center py-12">
                <Spinner />
                {message.clone().map(|msg| view! {
                    <p class="mt-4 text-slate-600">{msg}</p>
                })}
            </div>
        </Show>
    }
}
