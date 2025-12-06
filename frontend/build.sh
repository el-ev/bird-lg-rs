#!/usr/bin/env bash
set -e

BACKEND_URL="${BACKEND_URL:-wss://lg.example.com/ws}"
USERNAME="${USERNAME:-admin}"
OUTPUT_DIR="${OUTPUT_DIR:-./dist-packed}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --backend-url)
            BACKEND_URL="$2"
            shift 2
            ;;
        --username)
            USERNAME="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--backend-url URL] [--username NAME] [--output DIR]"
            echo ""
            echo "Options:"
            echo "  --backend-url URL   WebSocket backend URL (default: wss://lg.example.com/ws)"
            echo "  --username NAME     Username for the looking glass (default: admin)"
            echo "  --output DIR        Output directory (default: ./dist-packed)"
            echo ""
            echo "Environment variables:"
            echo "  BACKEND_URL, USERNAME, OUTPUT_DIR can also be set via env vars"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Building frontend with trunk..."
trunk build --release

echo "==> Creating output directory: $OUTPUT_DIR"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo "==> Copying built assets..."
cp -r dist/* "$OUTPUT_DIR/"

echo "==> Generating config.json..."
cat > "$OUTPUT_DIR/config.json" << EOF
{
    "backend_url": "$BACKEND_URL",
    "username": "$USERNAME"
}
EOF

echo "==> Build complete!"
echo ""
echo "Output: $OUTPUT_DIR"
echo "Config:"
cat "$OUTPUT_DIR/config.json"
echo ""
echo ""
echo "For nginx deployment, add this location block:"
echo '  location /visitor-info {'
echo '      add_header Content-Type application/json;'
echo '      return 200 '\''{"node": "$http_x_node_name", "ip": "$remote_addr"}'\'';'
echo '  }'
