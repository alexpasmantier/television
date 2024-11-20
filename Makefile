default: build

setup:
	@echo "Setting up $(NAME)"
	@echo "Installing git hooks"
	@mkdir -p .git/hooks
	@echo "make fix" > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Done"

clean:
	@echo "Cleaning build dir"
	@rm -rf target/*
	@echo "Cleaning using cargo"
	@cargo clean

check:
	@echo "Checking $(NAME)"
	@cargo check

release:
	@echo "Building release: $(VERSION)"
	@cargo build --release

build:
	@echo "Building debug"
	@cargo build

format:
	@echo "Formatting $(NAME)"
	@cargo fmt --all

lint:
	@echo "Linting $(NAME)"
	@cargo clippy

fix: format
	@echo "Fixing $(NAME)"
	@cargo fix --allow-staged
	@make lint

run:
	@echo "Running debug"
	@RUST_LOG=debug cargo run

test:
	@echo "Testing $(NAME)"
	@cargo test --all

bump-workspace:
	@echo "Bumping workspace"
	@./scripts/bump.sh

publish:
	@echo "Publishing $(NAME)"
	@./scripts/publish.sh

update-changelog:
	@echo "Updating changelog"
	@git cliff -o CHANGELOG.md
