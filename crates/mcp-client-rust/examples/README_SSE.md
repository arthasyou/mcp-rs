# SSE Transport Async Notification Example

This example demonstrates how to use the SSE transport with asynchronous server-pushed notifications.

## Features

The SSE transport now supports two ways to handle server-pushed notifications:

1. **Synchronous Callback Handler** - Set a function that will be called immediately when a notification is received
2. **Asynchronous Channel Receiver** - Get a receiver channel to handle notifications in a separate async task

Both methods can be used simultaneously if needed.

## Running the Example

1. First, start the test SSE server:
   ```bash
   cargo run --package mcp-client-rust --example sse_server_test
   ```

2. In another terminal, run the client:
   ```bash
   cargo run --package mcp-client-rust --example sse_simple_test
   ```

## API Usage

### Setting up Synchronous Handler

```rust
transport.set_message_handler(|msg| {
    match &msg {
        JsonRpcMessage::Notification(notification) => {
            println!("Received notification: {:?}", notification);
        }
        _ => {}
    }
}).await;
```

### Setting up Asynchronous Handler

```rust
if let Some(mut receiver) = transport.take_message_receiver().await {
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            // Handle notifications asynchronously
            match msg {
                JsonRpcMessage::Notification(notification) => {
                    // Process notification
                }
                _ => {}
            }
        }
    });
}
```

## Key Benefits

- **Non-blocking**: Notifications are handled without blocking the main request/response flow
- **Flexible**: Choose synchronous or asynchronous handling based on your needs
- **Concurrent**: Both handlers can work simultaneously
- **Type-safe**: Full Rust type safety with the JsonRpcMessage enum

## Implementation Details

- The synchronous handler is called immediately in the SSE event loop
- The async handler receives messages through an unbounded channel
- Request/response operations remain synchronous as before
- Only notifications (server-pushed messages without an ID) are sent to the handlers