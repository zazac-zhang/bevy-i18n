# CI/CD Pipeline Guide

This document describes the CI/CD workflows for the bevy_i18n project.

## Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and pull request to `master`/`main` branch.

**Jobs:**

- **test**: Runs tests on stable, beta, and nightly Rust
  - Tests all features
  - Tests binary tools (`i18n-extract`, `i18n-validate`)
  - Uses caching for faster builds

- **fmt**: Checks code formatting with `rustfmt`

- **clippy**: Runs Clippy lints with `-D warnings` (treat warnings as errors)

- **build**: Verifies the crate builds with different feature combinations
  - All features
  - No default features
  - Individual features (derive)

- **docs**: Builds and checks documentation

### 2. I18n Workflow (`.github/workflows/i18n.yml`)

Runs when locale files, source code, or examples change.

**Jobs:**

- **extract-and-validate**:
  - Builds i18n tools (`i18n-extract`, `i18n-validate`)
  - Extracts translation keys from source code
  - Validates locale files against the extracted template
  - Uploads template as artifact
  - Comments on PRs with validation results

- **locale-consistency** (if locale files exist):
  - Validates all locale files for consistency
  - Checks for empty translations
  - Generates coverage report
  - Uploads coverage report as artifact

### 3. Release Workflow (`.github/workflows/release.yml`)

Runs on version tags (e.g., `v1.0.0`).

**Jobs:**

- **create-release**:
  - Generates release notes from git commits
  - Creates GitHub release

- **publish-crate**:
  - Verifies tag version matches `Cargo.toml`
  - Publishes `bevy_i18n` and `bevy_i18n_derive` to crates.io

- **build-binaries**:
  - Builds release binaries for multiple platforms:
    - Linux (x86_64)
    - macOS (x86_64, aarch64)
    - Windows (x86_64)
  - Uploads binaries to GitHub release

- **publish-docs**:
  - Builds documentation
  - Deploys to GitHub Pages

## Local Development

### Using i18n Tools Locally

Build the tools:

```bash
cargo build --release --bin i18n-extract --bin i18n-validate
```

Extract translation keys:

```bash
./target/release/i18n-extract src locales/template.yaml
```

Validate locale files:

```bash
./target/release/i18n-validate assets/locales
```

Validate against template:

```bash
./target/release/i18n-validate -t locales/template.yaml assets/locales
```

### Running CI Checks Locally

Run tests:

```bash
cargo test --all-features
```

Check formatting:

```bash
cargo fmt --all -- --check
```

Run Clippy:

```bash
cargo clippy --all-features -- -D warnings
```

## Release Process

1. Update versions in `Cargo.toml` and `derive/Cargo.toml`
2. Commit changes: `git commit -m "Bump version to X.Y.Z"`
3. Create tag: `git tag vX.Y.Z`
4. Push tag: `git push origin vX.Y.Z`
5. GitHub Actions will:
   - Create GitHub release
   - Publish to crates.io
   - Build and upload binaries
   - Deploy documentation

## Required Secrets

For the release workflow, configure these secrets in your GitHub repository:

- `CARGO_REGISTRY_TOKEN`: crates.io authentication token
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

## Status Badges

Add these badges to your README:

```markdown
![CI](https://github.com/zazac-zhang/bevy-i18n/workflows/CI/badge.svg)
![I18n](https://github.com/zazac-zhang/bevy-i18n/workflows/I18n%20Validation/badge.svg)
```
