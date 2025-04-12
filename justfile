NAME := 'television'

# List all available commands
default:
	just --list

alias r := run

# Run the program in debug mode with logs enabled
@run:
	echo "Running {{ NAME }}..."
	RUST_LOG=debug cargo run
	echo "Done"

# Setup the project environment for local development
@setup:
	echo "Setting up {{ NAME }}..."
	echo "Installing git hooks..."
	mkdir -p .git/hooks
	echo "just fix" > .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit
	echo "Installing dependencies..."
	cargo build
	echo "Done"

# Clean up cargo build artifacts
@clean:
	echo "Cleaning up {{ NAME }}..."
	echo "Removing git hooks..."
	rm -f .git/hooks/pre-commit
	echo "Done"

alias c := check

# Check the project for errors and warnings
@check:
	echo "Checking {{ NAME }}..."
	cargo check
	echo "Done"

# Format the code using cargo fmt
@format:
	echo "Formatting {{ NAME }}..."
	cargo fmt --all
	echo "Done"

# Lint the code using cargo clippy
@lint:
	echo "Linting {{ NAME }}..."
	cargo clippy --all-targets --all-features -- -D warnings
	echo "Done"

alias f := fix
# Fix linting and formatting errors
@fix:
	echo "Fixing {{ NAME }}..."
	cargo fix --allow-dirty --allow-staged
	just format
	just lint

alias t := test
# Run the tests for the project
@test:
	echo "Running {{ NAME }}'s test suite..."
	cargo test --all
	echo "Done"

alias b := build
# Build the project with the specified profile (dev by default)
@build profile='dev':
	echo "Building {{ NAME }} for profile: {{ profile }}..."
	cargo build --profile {{ profile }}
	echo "Done"

# Build the project in release mode
br: (build 'release')

# Update the project's changelog
@update-changelog:
	echo "Updating changelog..."
	git cliff -o CHANGELOG.md
	echo "Done"

alias m := update-man
# Update the project's manpages
update-man: build
	#!/usr/bin/env sh
	echo "Checking for manpages updates..."
	if ! diff ./man/tv.1 ./target/assets/tv.1 > /dev/null;
	then cp ./target/assets/tv.1 ./man/tv.1 && echo "Updated manpages"
	else echo "No changes to manpages"
	fi

alias rl := release
# Publish a new release (major, minor, or patch)
release kind='patch':
	#!/usr/bin/env sh
	echo "Releasing {{ NAME }} (kind: {{ kind }})..."
	# bump version (major, minor, patch)
	version=$(grep -E '^\s*version\s*=' Cargo.toml | cut -d '"' -f 2)
	kind="{{ kind }}"
	echo "Current version is: $version"
	if [ "$kind" = "major" ]; then
		new_version=$(echo $version | awk -F. -v OFS=. '{$1++; $2=0; $3=0} 1')
	elif [ "$kind" = "minor" ]; then
		new_version=$(echo $version | awk -F. -v OFS=. '{$2++; $3=0} 1')
	elif [ "$kind" = "patch" ]; then
		new_version=$(echo $version | awk -F. -v OFS=. '{$3++} 1')
	else
		echo "Invalid kind: $kind"
		exit 1
	fi
	echo "New version is: $new_version"
	sed -i "s/version = \"$version\"/version = \"$new_version\"/" Cargo.toml
	git add Cargo.toml
	echo "Done"

