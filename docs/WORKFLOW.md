# I18n Workflow Guide

This guide explains how to use the i18n tools and CI/CD workflow for internationalization in bevy_i18n.

## Quick Start

### 1. Extract Translation Keys

Run the extraction script to scan your source code and generate a template:

```bash
./scripts/extract-i18n.sh
```

Or manually:

```bash
cargo build --release --bin i18n-extract
./target/release/i18n-extract src locales/template.yaml
```

This scans all `.rs` files in `src/` for:
- `T::new("key")`
- `T::with_vars("key", &[("var", "val")])`
- `T::plural("key", count)`
- `T::with_context("key", "context")`
- `T::ns("namespace").key("key")`

### 2. Create Locale Files

Copy the template to create locale files:

```bash
mkdir -p assets/locales
cp locales/template.yaml assets/locales/en.yaml
cp locales/template.yaml assets/locales/zh.yaml
```

Fill in the translations for each locale.

### 3. Validate Locale Files

Run the validation script:

```bash
./scripts/validate-i18n.sh
```

Or manually:

```bash
cargo build --release --bin i18n-validate
./target/release/i18n-validate assets/locales
```

This checks for:
- Missing keys across locales
- Extra keys (inconsistent key sets)
- Variable placeholder mismatches

## CI/CD Integration

The GitHub Actions workflow automatically:

1. **On PR/Push**:
   - Extracts translation keys from source
   - Validates all locale files
   - Comments on PRs with validation results
   - Reports coverage statistics

2. **On Release**:
   - Builds release binaries of i18n tools
   - Publishes to crates.io and GitHub releases

## Tool Usage

### i18n-extract

```bash
# Basic usage
i18n-extract <source_dir> [output_file]

# Examples
i18n-extract src
i18n-extract examples locales/examples.yaml
```

Scans Rust source files and generates a YAML template with:
- All translation keys found in code
- Variable placeholders extracted from `T::with_vars()` calls
- Source file references

### i18n-validate

```bash
# Basic usage
i18n-validate [OPTIONS] [locale_dir]

# Options
-t, --template <file>  Compare against a template YAML file
-d, --dir <dir>        Locale directory (default: assets/locales)

# Examples
i18n-validate assets/locales
i18n-validate -t locales/template.yaml assets/locales
```

Validates locale files for:
- Missing keys (errors)
- Inconsistent keys across locales (warnings)
- Variable placeholder mismatches (warnings)

## Example Workflow

### Adding a New Translation Key

1. **Add the key in code**:

```rust
commands.spawn((
    Text::new(""),
    T::with_vars("player.health", &[("current", "100"), ("max", "100")]),
));
```

2. **Extract keys**:

```bash
./scripts/extract-i18n.sh
```

3. **Update locale files**:

```yaml
# assets/locales/en.yaml
player:
  health: "Health: {current}/{max}"
```

```yaml
# assets/locales/zh.yaml
player:
  health: "生命值：{current}/{max}"
```

4. **Validate**:

```bash
./scripts/validate-i18n.sh
```

### Adding a New Locale

1. **Create new locale file**:

```bash
cp assets/locales/en.yaml assets/locales/ja.yaml
```

2. **Translate all keys**:

```yaml
# assets/locales/ja.yaml
game:
  title: "Bevy I18n デモ"
```

3. **Validate**:

```bash
./scripts/validate-i18n.sh
```

## Best Practices

1. **Always extract before committing**:
   ```bash
   ./scripts/extract-i18n.sh
   git add locales/template.yaml
   ```

2. **Use variables consistently**:
   - If EN uses `{name}` and `{count}`, all locales must use the same variables
   - Variable order doesn't matter, but presence does

3. **Keep locale files in sync**:
   - All locales should have the same keys
   - Use CI/CD to catch inconsistencies early

4. **Test with actual data**:
   - The tools validate structure, not content
   - Manually test with real translated strings

## Troubleshooting

### "No YAML files found" Error

Make sure your locale files exist:
```bash
ls assets/locales/*.yaml
```

### Variable Inconsistency Warnings

Check that all locales use the same variables:
```bash
./target/release/i18n-validate assets/locales
```

### Missing Key Errors

Ensure all locales have all keys:
```bash
./target/release/i18n-validate -t locales/template.yaml assets/locales
```

## CI/CD Status Badges

Add to your README:

```markdown
![CI](https://github.com/zazac-zhang/bevy-i18n/workflows/CI/badge.svg)
![I18n](https://github.com/zazac-zhang/bevy-i18n/workflows/I18n%20Validation/badge.svg)
```
