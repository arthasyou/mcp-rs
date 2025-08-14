#!/bin/bash

# Kill any existing server
pkill -f "sse_server_test" || true

# Start server in background
echo "Starting SSE server..."
cargo run --package mcp-client-rust --example sse_server_test &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Run client
echo "Starting SSE client..."
cargo run --package mcp-client-rust --example sse_client_test

# Kill server when done
kill $SERVER_PID 2>/dev/null || true