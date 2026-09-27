# Boy Scout Meal Planning Web Application

A full-stack Rust web application for planning meals at boy scout camps, built with Leptos framework.

## Features

- 📝 Recipe management with flexible portion sizes
- 🥕 Ingredient tracking with multiple units and categories
- 📅 Meal planning calendar for camps
- 👥 Support for different person types (child, teen, adult) with adjustable portions
- 📊 PDF report generation (daily ingredients, camp shopping lists)
- 🌍 Multilingual support (English/Czech)
- 🐳 Docker deployment ready

## Technology Stack

- **Framework**: Leptos (Rust full-stack with SSR)
- **Backend**: Axum web server
- **Database**: SQLite with sqlx
- **PDF Generation**: printpdf
- **Styling**: TailwindCSS
- **Deployment**: Docker

## Quick Start

### Prerequisites

- Rust (latest stable)
- cargo-leptos: `cargo install cargo-leptos`
- Docker (for containerized deployment)

### Development

1. Clone the repository
2. Copy `.env.example` to `.env` and configure as needed
3. Run the development server:

```bash
cargo leptos watch
```

The application will be available at `http://localhost:3000`

### Building for Production

```bash
cargo leptos build --release
```

### Docker Deployment

Build and run with Docker Compose:

```bash
docker-compose up -d
```

Or build the Docker image manually:

```bash
docker build -t meal-planner .
docker run -p 3000:3000 -v ./data:/app/data meal-planner
```

### GCP Deployment

The app runs on `homelab-vm` (zone `us-east1-b`, GCP project `homelab`), a
VM it shares with [wheel](https://github.com/slune/wheel). The VM has no
public IP: SSH goes through IAP, outbound traffic through Cloud NAT, and
you reach the app through an SSH tunnel.

Once, to create the project, VM and network and deploy the first time:

```bash
scripts/gcp-setup.sh
```

After that:

```bash
scripts/update.sh        # deploy
scripts/tunnel.sh        # then open https://localhost:3000 (Ctrl-C closes it)
scripts/backup-db.sh     # copy the database to backups/
scripts/vm.sh start      # or: stop, destroy
```

`vm.sh stop` and `destroy` also stop or delete wheel.

## Project Structure

```
ai_meal_planning/
├── src/
│   ├── main.rs           # Server entry point
│   ├── lib.rs            # Library root
│   ├── app.rs            # Leptos app component
│   ├── db.rs             # Database initialization
│   ├── models/           # Data models
│   ├── api/              # Backend API handlers
│   ├── reports/          # PDF report generation
│   ├── components/       # Leptos UI components
│   └── pages/            # Page components
├── migrations/           # SQL migration files
├── locales/              # i18n translation files
│   ├── en.ftl           # English translations
│   └── cz.ftl           # Czech translations
├── style/                # CSS styles
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Docker configuration
└── docker-compose.yml   # Docker Compose setup
```

## Database Schema

The application uses SQLite with the following main tables:

- **categories**: Ingredient categories (meat, vegetables, etc.)
- **ingredients**: Individual ingredients with units
- **recipes**: Recipe definitions with instructions
- **recipe_ingredients**: Junction table linking recipes to ingredients with portions
- **camps**: Camp events with dates and default attendance
- **meal_plans**: Daily meal plans for camps
- **planned_meals**: Individual meals (breakfast, lunch, etc.)
- **meal_attendance**: Attendance overrides per meal

## Configuration

Environment variables:

- `DATABASE_URL`: SQLite database path (default: `sqlite://data/meal_planning.db`)
- `AUTH_PASSWORD`: Application password (default: `admin123`)
- `RUST_LOG`: Logging level (default: `info`)

## Features in Detail

### Recipe Management

- Create recipes with multiple ingredients
- Set recipe portions
- Define portion multipliers for children, teens, and adults
- Support for multiple units per ingredient (kg, g, pieces, etc.)

### Meal Planning

- Plan meals for multiple days
- 5 meal types: breakfast, morning snack, lunch, afternoon snack, dinner
- Override attendance per meal or use camp defaults
- Support for partial meal days

### Report Generation

- **Daily Report**: Ingredients needed for a specific day, grouped by category
- **Camp Report**: Complete shopping list for entire camp duration
- Both reports available in English and Czech
- PDF format for easy printing

## Development Notes

- The application uses Leptos for both server-side and client-side rendering
- Server functions communicate between frontend and backend
- SQLite provides zero-configuration database storage
- All database queries are compile-time checked with sqlx macros

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
