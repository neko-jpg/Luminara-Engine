#!/bin/bash
# Ultimate Demo Runner Script

set -e

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║  LUMINARA ENGINE - ULTIMATE PHASE 0-1 DEMO                   ║"
echo "║  Physics Playground with Debug Visualization                 ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check if running on WSL
if grep -qi microsoft /proc/version 2>/dev/null; then
    echo "⚠️  Detected WSL environment"
    echo "   For best performance, consider running on native Linux or Windows"
    echo ""
fi

# Build the demo
echo "🔨 Building Ultimate Demo..."
/home/arat2/.cargo/bin/cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Run the demo
echo "🚀 Launching Ultimate Demo..."
echo ""
echo "🎮 Controls:"
echo "  WASD + Mouse  : Fly Camera"
echo "  Shift         : Sprint"
echo "  Space/Ctrl    : Up/Down"
echo "  R             : Replay (Deterministic)"
echo "  G             : Toggle Gizmos"
echo "  P             : Pause Physics"
echo "  T             : Spawn Object"
echo "  C             : Clear Objects"
echo "  1-5           : Camera Presets"
echo ""
echo "Press Ctrl+C to exit"
echo ""

RUST_LOG=info /home/arat2/.cargo/bin/cargo run --release

echo ""
echo "👋 Demo closed. Thanks for watching!"
