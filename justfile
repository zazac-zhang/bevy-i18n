# bevy_i18n development commands
# Install: brew install just || cargo install just
# Run: just <command>

# Default recipe - show available commands
default:
    @just --list

# ==============================================================================
# Development
# ==============================================================================

# Build the project
build:
    cargo build

# Build with all features
build-all:
    cargo build --all-features

# Build release binaries
build-release:
    cargo build --release --all-features

# Run tests
test:
    cargo test --all-features

# Run tests with output
test-verbose:
    cargo test --all-features -- --nocapture

# Run tests for specific crate
test-crate crate:
    cargo test -p {{crate}}

# Run example
run-example example:
    cargo run --example {{example}}

# Run the demo example
demo:
    cargo run --example demo

# Check code (fast build check)
check:
    cargo check --all-features

# Format code
fmt:
    cargo fmt --all

# Check formatting without changes
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --all-features -- -D warnings

# Fix clippy warnings automatically
clippy-fix:
    cargo clippy --all-features --fix --allow-dirty --allow-staged

# Run documentation generation
docs:
    cargo doc --no-deps --all-features

# Open documentation in browser
docs-open:
    cargo doc --no-deps --all-features --open

# ==============================================================================
# i18n Tools
# ==============================================================================

# Build i18n tools
build-i18n:
    cargo build --release --bin i18n-extract --bin i18n-validate

# Extract translation keys from source
i18n-extract:
    cargo run --release --bin i18n-extract -- src locales/template.yaml

# Extract keys to custom output
i18n-extract-to output:
    cargo run --release --bin i18n-extract -- src {{output}}

# Validate locale files
i18n-validate:
    cargo run --release --bin i18n-validate -- assets/locales

# Validate against template
i18n-validate-check:
    cargo run --release --bin i18n-validate -- -t locales/template.yaml assets/locales

# Full i18n workflow (extract + validate)
i18n-check: build-i18n
    ./target/release/i18n-extract src locales/template.yaml
    ./target/release/i18n-validate -t locales/template.yaml assets/locales

# Create new locale file from template
i18n-new-locale locale:
    #!/usr/bin/env bash
    if [ ! -f "locales/template.yaml" ]; then
        echo "Error: locales/template.yaml not found. Run 'just i18n-extract' first."
        exit 1
    fi
    mkdir -p assets/locales
    cp locales/template.yaml "assets/locales/{{locale}}.yaml"
    echo "Created assets/locales/{{locale}}.yaml"
    echo "Edit this file to add translations for {{locale}}"

# Show i18n statistics
i18n-stats:
    #!/usr/bin/env bash
    if [ ! -d "assets/locales" ]; then
        echo "No assets/locales directory found"
        exit 1
    fi
    echo "📊 Locale Statistics:"
    echo ""
    for file in assets/locales/*.yaml; do
        if [ -f "$file" ]; then
            locale=$(basename "$file" .yaml)
            count=$(grep -cE '^\s*\w+:' "$file" 2>/dev/null || echo "0")
            echo "  $locale: $count keys"
        fi
    done

# ==============================================================================
# CI/CD
# ==============================================================================

# Run all CI checks locally
ci: fmt-check clippy test build-i18n i18n-validate-check

# Quick check (format + clippy)
quick-check: fmt-check clippy

# Run pre-commit checks
pre-commit: fmt-check clippy i18n-validate-check

# Generate coverage report (requires tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Run security audit
audit:
    cargo audit

# Update dependencies
update:
    cargo update

# Clean build artifacts
clean:
    cargo clean
    rm -rf locales/

# Deep clean (includes generated files)
deep-clean: clean
    rm -rf .justfile
    rm -rf coverage/
    find . -name "*.rs.bk" -delete

# ==============================================================================
# Release
# ==============================================================================

# Prepare release (bump version, update changelog)
release-prep version:
    #!/usr/bin/env bash
    if [[ ! "{{version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Version must be in format X.Y.Z"
        exit 1
    fi

    # Update main Cargo.toml
    sed -i '' "s/^version = .*/version = \"{{version}}\"/" Cargo.toml

    # Update derive Cargo.toml
    sed -i '' "s/^version = .*/version = \"{{version}}\"/" derive/Cargo.toml

    # Update dependency version
    sed -i '' "s/bevy_i18n_derive = { version = \"[^\"]*\"/bevy_i18n_derive = { version = \"{{version}}\"/" Cargo.toml

    echo "✅ Updated version to {{version}}"
    echo "📝 Please update CHANGELOG.md"
    echo "🔖 Then run: git add . && git commit -m 'Bump version to {{version}}'"

# Create release tag
release-tag version:
    #!/usr/bin/env bash
    if [[ ! "{{version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Version must be in format X.Y.Z"
        exit 1
    fi
    git tag -a "v{{version}}" -m "Release v{{version}}"
    git push origin "v{{version}}"
    echo "✅ Created and pushed tag v{{version}}"
    echo "🚀 GitHub Actions will handle the release"

# ==============================================================================
# Development Utilities
# ==============================================================================

# Watch for changes and re-run tests (requires cargo-watch)
watch:
    cargo watch -x test --all-features

# Watch and run clippy
watch-clippy:
    cargo watch -x clippy --all-features

# Show dependency tree
deps:
    cargo tree

# Show outdated dependencies
outdated:
    cargo outdated

# Print project size analysis
size:
    cargo cargo-modules

# Run benchmarks
bench:
    cargo bench --all-features

# Install development tools
install-tools:
    cargo install cargo-watch
    cargo install cargo-audit
    cargo install cargo-outdated
    cargo install cargo-tarpaulin
    cargo install cargo-modules
    echo "✅ Development tools installed"

# ==============================================================================
# Git Utilities
# ==============================================================================

# Show git status
status:
    git status

# Show recent commits
log:
    git log --oneline -10

# Show diff of staged changes
diff-staged:
    git diff --staged

# Show diff of unstaged changes
diff:
    git diff

# Create a new branch
branch branch_name:
    git checkout -b {{branch_name}}

# Merge current branch to main
merge-branch:
    #!/usr/bin/env bash
    current=$(git branch --show-current)
    git checkout master
    git merge $current
    echo "✅ Merged $current to master"

# ==============================================================================
# Examples
# ==============================================================================

# List all available examples
list-examples:
    #!/usr/bin/env bash
    echo "📋 Available examples:"
    cargo run --example 2>&1 | grep "available examples" -A 20 || find examples -name "*.rs" -exec basename {} .rs \;

# ==============================================================================
# Documentation
# ==============================================================================

# Read main README
readme:
    bat README.md

# Read CI/CD documentation
readme-ci:
    bat docs/CI_CD.md

# Read workflow documentation
readme-workflow:
    bat docs/WORKFLOW.md

# ==============================================================================
# Testing Utilities
# ==============================================================================

# Run nextest (faster test runner)
test-nextest:
    cargo nextest run --all-features

# Run tests with nextest and output
test-nextest-verbose:
    cargo nextest run --all-features --success-output immediate

# Show test coverage in terminal (requires cargo-llvm-cov)
cov:
    cargo llvm-cov --lcov --output-path lcov.info

# Open coverage report in browser
cov-open:
    cargo llvm-cov --open

# ==============================================================================
# Helper Recipes
# ==============================================================================

# Setup development environment
setup: install-tools
    cargo fetch
    echo "✅ Development environment ready"
    echo "📚 Run 'just readme' to get started"

# Show project information
info:
    #!/usr/bin/env bash
    echo "📦 bevy_i18n Project Information"
    echo "================================"
    echo "Version: $(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Just: $(just --version)"
    echo ""
    echo "🔧 Available Features:"
    cargo run --quiet 2>&1 | grep "features:" -A 10 || echo "  yaml, po, derive"
    echo ""
    echo "📋 Common Commands:"
    echo "  just build       - Build project"
    echo "  just test        - Run tests"
    echo "  just i18n-check  - Validate i18n"
    echo "  just ci          - Run CI checks"
    echo "  just setup       - Setup dev environment"
