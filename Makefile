VERSION=0.2.13
NAME=television
EXEC=tv
PREFIX=$(HOME)/.local

default: build_release

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

build_release:
	@echo "Building release: $(VERSION)"
	@cargo build --release

build_debug:
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

run:
	@echo "Running debug"
	@RUST_LOG=debug cargo run

test:
	@echo "Testing $(NAME)"
	@cargo test

install_debug: build_debug
	@echo "Installing debug"
	@cp target/debug/$(EXEC) $(PREFIX)/bin

install: build_release
	@echo "Installing release: $(VERSION)"
	@cp target/release/$(EXEC) $(PREFIX)/bin

dist: build_release
	@if [ ! -d ./pkg ]; \
	then \
		mkdir ./pkg; \
	fi

	@if [ -d ./pkg/$(NAME)-$(VERSION) ]; \
	then \
		echo "Current version number already exists! Removing old files!"; \
		rm -rf ./pkg/$(NAME)-$(VERSION); \
	fi

	@mkdir ./pkg/$(NAME)-$(VERSION)

	@cp ./dist-scripts/install.sh ./pkg/$(NAME)-$(VERSION)/

	@sed -i 's#{prefix}#$(PREFIX)#g' ./pkg/$(NAME)-$(VERSION)/install.sh
	@sed -i 's#{version}#$(VERSION)#g' ./pkg/$(NAME)-$(VERSION)/install.sh
	@sed -i 's#{name}#$(NAME)#g' ./pkg/$(NAME)-$(VERSION)/install.sh
	@sed -i 's#{exec}#$(EXEC)#g' ./pkg/$(NAME)-$(VERSION)/install.sh

	@mkdir ./pkg/$(NAME)-$(VERSION)/files
	@cp target/release/$(EXEC) ./pkg/$(NAME)-$(VERSION)/files/
	@strip ./pkg/$(NAME)-$(VERSION)/files/$(EXEC)

	@cp LICENSE ./pkg/$(NAME)-$(VERSION)/

	@cd ./pkg && tar -czf ./$(NAME)-$(VERSION).tar.gz ./$(NAME)-$(VERSION)
	@echo "Cleaning up"
	@rm -rf ./pkg/$(NAME)-$(VERSION)
