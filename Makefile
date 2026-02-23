.PHONY: dev build clean docker-build docker-run

dev:
	npx tailwindcss -i style/main.css.source -o style/main.css --minify --watch & cargo leptos watch; kill %1 2>/dev/null

build:
	cargo leptos build --release

clean:
	cargo clean
	rm -rf target/
	rm -rf pkg/

docker-build:
	docker build -t meal-planner .

docker-run:
	docker run -p 3000:3000 -v ./data:/app/data meal-planner

docker-compose-up:
	docker-compose up -d

docker-compose-down:
	docker-compose down

setup-db:
	mkdir -p data
	touch data/.gitkeep

install-deps:
	cargo install cargo-leptos
	npm install -D tailwindcss

fmt:
	cargo fmt

check:
	cargo check --all-features

test:
	cargo test
