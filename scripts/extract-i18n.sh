#!/bin/bash
# Extract translation keys from source code
# Usage: ./scripts/extract-i18n.sh [output_file]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

OUTPUT_FILE="${1:-locales/template.yaml}"

echo "🔨 Building i18n-extract..."
cargo build --release --bin i18n-extract

echo ""
echo "📋 Extracting translation keys from src/..."
./target/release/i18n-extract src "$OUTPUT_FILE"

echo ""
echo "✅ Template generated at: $OUTPUT_FILE"
echo ""
echo "📝 Next steps:"
echo "   1. Review the template file"
echo "   2. Copy it to assets/locales/ for each language (e.g., en.yaml, zh.yaml)"
echo "   3. Fill in the translations"
echo "   4. Run ./scripts/validate-i18n.sh to check consistency"
