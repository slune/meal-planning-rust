# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A full-stack Rust web application for planning meals at boy scout camps, built with Leptos framework. Features recipe management, meal planning calendar, PDF report generation, and multilingual support (English/Czech).

## Technology Stack

- **Framework**: Leptos 0.8 (full-stack with SSR and hydration)
- **Backend**: Axum web server
- **Database**: SQLite with sqlx (compile-time checked queries)
- **PDF Generation**: printpdf
- **Styling**: TailwindCSS
- **i18n**: Fluent localization (locales/en.ftl, locales/cz.ftl)

## Development Commands

```bash
# Development server (default port 3000)
cargo leptos watch

# Production build
cargo leptos build --release

# Run tests
cargo test

# Format code
cargo fmt

# Check all features
cargo check --all-features

# Docker deployment
docker-compose up -d
```

Makefile targets: `make dev`, `make build`, `make test`, `make fmt`, `make check`

## Architecture

Leptos full-stack architecture with three main layers:

1. **Models** (`src/models/`): Core data structures shared between client and server
   - `category.rs`, `ingredient.rs`, `recipe.rs`, `camp.rs`, `meal_plan.rs`
   - `MealType` enum: breakfast, morning_snack, lunch, afternoon_snack, dinner

2. **API Layer** (`src/api/`): Database operations using sqlx
   - Direct SQLite queries with compile-time verification
   - One module per domain (categories, ingredients, recipes, camps, meal_plans)
   - Always accept `&SqlitePool` as first parameter

3. **Server Functions** (`src/server_functions/`): Leptos server functions
   - Annotated with `#[server(FunctionName, "/api")]`
   - Bridge between frontend and API layer
   - Use `expect_context::<sqlx::SqlitePool>()` to get database connection
   - Return `Result<T, ServerFnError>`

4. **Components** (`src/components/`): Leptos UI components
   - Reusable UI elements annotated with `#[component]`

5. **Pages** (`src/pages/`): Route-level components
   - Configured in `src/app.rs` router

6. **Reports** (`src/reports/`): PDF generation logic
   - `generate_daily_report()`: Ingredients for a specific day
   - `generate_camp_report()`: Complete shopping list for camp

## Database

- SQLite database at `data/meal_planning.db` (configurable via `DATABASE_URL`)
- Migrations run automatically on startup from `src/db.rs`
- Migration files in `migrations/` directory (numbered 001-005)
- Schema: categories, ingredients, recipes, recipe_ingredients, camps, meal_plans, planned_meals, meal_attendance

## Feature Flags

- `ssr`: Server-side rendering (enables api, db, reports modules)
- `hydrate`: Client-side hydration (WASM)

## Key Patterns

- Server functions are the ONLY way frontend communicates with backend
- Database queries use sqlx macros for compile-time verification
- All database operations go through the API layer
- Context propagation: `SqlitePool` is provided via Leptos context in `main.rs`
- Components use reactive signals for state management

## Configuration

Environment variables:
- `DATABASE_URL`: SQLite database path (default: `sqlite://data/meal_planning.db`)
- `AUTH_PASSWORD`: Application password (default: `admin123`)
- `RUST_LOG`: Logging level (default: `info`)

## Adding New Features

When adding a new domain entity:
1. Create model in `src/models/`
2. Create API functions in `src/api/`
3. Create server functions in `src/server_functions/`
4. Create UI components in `src/components/`
5. Add page component in `src/pages/` if needed
6. Register route in `src/app.rs`
