# MCP SSE Client Examples

这里包含了使用 SSE transport 连接到 MCP 服务的示例代码。

## 前置条件

确保 MCP 服务端正在运行：

```bash
cd /Users/ancient/src/rust/mcp-service
cargo run
```

服务端默认监听在 `http://localhost:3000`。

## 示例说明

### 1. simple_sse_client.rs

一个简单的 SSE 客户端示例，展示如何：
- 连接到 SSE 服务
- 接收服务端推送的消息
- 优雅地断开连接

运行命令：
```bash
cargo run --example simple_sse_client -p mcp-client-rust
```

### 2. sse_client_with_messages.rs

一个更完整的示例，展示如何：
- 连接到 SSE 服务
- 发送通知消息到服务端
- 接收并处理不同类型的 JSON-RPC 消息
- 错误处理和日志记录

运行命令：
```bash
cargo run --example sse_client_with_messages -p mcp-client-rust
```

## 可用的服务

MCP 服务端提供了以下服务（通过 URL 参数指定）：
- `chart` - 图表相关功能
- `counter` - 计数器功能  
- `corpus` - 语料库功能（需要配置 LLM 客户端）

连接示例：
```rust
// 连接到 counter 服务
let transport = SseTransport::new("http://localhost:3000/sse?service=counter");

// 连接到 chart 服务
let transport = SseTransport::new("http://localhost:3000/sse?service=chart");
```

## SSE Transport 工作原理

1. 客户端连接到 SSE endpoint
2. 服务端发送一个 `endpoint` 事件，包含用于发送消息的 POST URL
3. 客户端使用该 POST URL 发送消息到服务端
4. 服务端通过 SSE 连接推送消息到客户端

## 调试

设置环境变量来查看更详细的日志：

```bash
RUST_LOG=debug cargo run --example simple_sse_client -p mcp-client-rust
```

或者只查看 mcp_client_rust 的调试日志：

```bash
RUST_LOG=info,mcp_client_rust=debug cargo run --example simple_sse_client -p mcp-client-rust
```