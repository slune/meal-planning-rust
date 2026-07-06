use leptos::prelude::*;

pub fn icon(name: &str) -> AnyView {
    match name {
        "kitchen" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 18h14"/>
                <path d="M7 18V9a5 5 0 0 1 10 0v9"/>
                <path d="M9 6V3"/>
                <path d="M12 5V2"/>
                <path d="M15 6V3"/>
            </svg>
        }
        .into_any(),
        "camp" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 19 10.5 5 18 19"/>
                <path d="M10.5 5 21 19"/>
                <path d="M8 19l4-7 4 7"/>
            </svg>
        }
        .into_any(),
        "ingredients" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 8h16"/>
                <path d="M6 8l1.4 11h9.2L18 8"/>
                <path d="M9 8V6a3 3 0 0 1 6 0v2"/>
                <path d="M10 13h4"/>
            </svg>
        }
        .into_any(),
        "recipes" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 4h9a4 4 0 0 1 4 4v16H7a2 2 0 0 1-2-2V4z"/>
                <path d="M9 8h5"/>
                <path d="M9 12h6"/>
                <path d="M9 16h4"/>
            </svg>
        }
        .into_any(),
        "planner" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 5h14a2 2 0 0 1 2 2v14H3V7a2 2 0 0 1 2-2z"/>
                <path d="M8 3v4"/>
                <path d="M16 3v4"/>
                <path d="M3 10h18"/>
                <path d="M8 15h3"/>
                <path d="M14 15h3"/>
            </svg>
        }
        .into_any(),
        "reports" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 20V4h14v16z"/>
                <path d="M8 16v-4"/>
                <path d="M12 16V8"/>
                <path d="M16 16v-6"/>
            </svg>
        }
        .into_any(),
        "settings" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"/>
                <path d="M12 3v3"/>
                <path d="M12 18v3"/>
                <path d="M3 12h3"/>
                <path d="M18 12h3"/>
                <path d="m5.6 5.6 2.1 2.1"/>
                <path d="m16.3 16.3 2.1 2.1"/>
                <path d="m18.4 5.6-2.1 2.1"/>
                <path d="m7.7 16.3-2.1 2.1"/>
            </svg>
        }
        .into_any(),
        "database" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <ellipse cx="12" cy="5" rx="7" ry="3"/>
                <path d="M5 5v6c0 1.7 3.1 3 7 3s7-1.3 7-3V5"/>
                <path d="M5 11v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6"/>
            </svg>
        }
        .into_any(),
        "download" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 3v12"/>
                <path d="m7 10 5 5 5-5"/>
                <path d="M5 21h14"/>
            </svg>
        }
        .into_any(),
        "category" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 6h7l2 2h7v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z"/>
                <path d="M4 10h16"/>
            </svg>
        }
        .into_any(),
        "calendar" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 5h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z"/>
                <path d="M8 3v4"/>
                <path d="M16 3v4"/>
                <path d="M3 10h18"/>
            </svg>
        }
        .into_any(),
        "users" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M16 19v-1a4 4 0 0 0-8 0v1"/>
                <circle cx="12" cy="8" r="3"/>
                <path d="M4 19v-1a3 3 0 0 1 3-3"/>
                <path d="M20 19v-1a3 3 0 0 0-3-3"/>
            </svg>
        }
        .into_any(),
        "child" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="7" r="3"/>
                <path d="M9 13h6"/>
                <path d="M10 13 8 20"/>
                <path d="M14 13l2 7"/>
                <path d="M10 16h4"/>
            </svg>
        }
        .into_any(),
        "teen" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="6.5" r="2.5"/>
                <path d="M8 21v-6a4 4 0 0 1 8 0v6"/>
                <path d="M9 14h6"/>
                <path d="M17 15h2v4h-2"/>
            </svg>
        }
        .into_any(),
        "adult" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="6" r="3"/>
                <path d="M6 21v-4a6 6 0 0 1 12 0v4"/>
                <path d="M8 17h8"/>
            </svg>
        }
        .into_any(),
        "search" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="11" cy="11" r="6"/>
                <path d="m16 16 4 4"/>
            </svg>
        }
        .into_any(),
        "plus" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 5v14"/>
                <path d="M5 12h14"/>
            </svg>
        }
        .into_any(),
        "edit" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 20h4l10.5-10.5a2.1 2.1 0 0 0-3-3L5 17z"/>
                <path d="m14 7 3 3"/>
            </svg>
        }
        .into_any(),
        "trash" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16"/>
                <path d="M10 11v6"/>
                <path d="M14 11v6"/>
                <path d="M6 7l1 14h10l1-14"/>
                <path d="M9 7V4h6v3"/>
            </svg>
        }
        .into_any(),
        "copy" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 8h10v12H8z"/>
                <path d="M6 16H4V4h10v2"/>
            </svg>
        }
        .into_any(),
        "pdf" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 3h8l4 4v14H6z"/>
                <path d="M14 3v5h5"/>
                <path d="M8 16h8"/>
                <path d="M8 12h3"/>
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 6h12v12H6z"/>
            </svg>
        }
        .into_any(),
    }
}
