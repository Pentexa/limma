# Basit Makefile - geliştirici iş akışını hızlandırmak için
.PHONY: help build-backend run-backend build-frontend run-frontend docker-build

help:
	@echo "Targets:"
	@echo "  make build-backend   - Rust backend'i release olarak derle"
	@echo "  make run-backend     - Backend'i doğrudan cargo run ile başlat"
	@echo "  make build-frontend  - Frontend'i build et"
	@echo "  make run-frontend    - Frontend development server'ı başlat"
	@echo "  make docker-build    - Backend için docker image oluştur (local)"

build-backend:
	cargo build --manifest-path backend/Cargo.toml --release

run-backend:
	cd backend && cargo run --bin limma

build-frontend:
	cd frontend && npm install && npm run build

run-frontend:
	cd frontend && npm install && npm run dev

docker-build:
	docker build -t pentexa/limma-backend:latest -f backend/Dockerfile .
