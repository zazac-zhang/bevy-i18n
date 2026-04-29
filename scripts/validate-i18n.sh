#!/bin/bash
# Local i18n validation script
# Usage: ./scripts/validate-i18n.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "🔨 Building i18n tools..."
cargo build --release --bin i18n-extract --bin i18n-validate

echo ""
echo "📋 Extracting translation keys from source..."
./target/release/i18n-extract src locales/template.yaml

echo ""
echo "✅ Validating locale files..."
if [ -d "assets/locales" ]; then
    ./target/release/i18n-validate -t locales/template.yaml assets/locales
    echo ""
    echo "✨ All validations passed!"
else
    echo "⚠️  No assets/locales directory found"
    echo "   Create it and add your locale files to enable validation"
fi

echo ""
echo "📊 Template generated at: locales/template.yaml"
