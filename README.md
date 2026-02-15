# SSE (Server-Sent Events) Capability Provider

A wasmCloud capability provider that connects to remote SSE servers and forwards received events to components using the standard `wasmcloud:messaging` interface via wRPC. It implements unidirectional communication (receiving only) with automatic reconnection, configurable message size limits, and TLS support for `https://` connections.

## Building

Prerequisites: [Rust toolchain](https://www.rust-lang.org/tools/install), [wash CLI](https://wasmcloud.com/docs/installation)

```bash
# Build the provider (.par.gz archive)
wash build

# Build the test component
wash build -p ./component
```

## Testing

Run the automated integration test:

```bash
./tests/run_integration_test.sh
```

Or deploy as a WADM application:

```bash
wash up -d
wash app deploy ./wadm.yaml
```

See [TESTING.md](./TESTING.md) for detailed manual testing steps.

## Configuration

Link configuration values passed via `wash config put`:

| Key | Description | Default |
|-----|-------------|---------|
| `sse_url` | SSE server URL (`http://` or `https://`) | *required* |
| `max_reconnect_attempts` | Max reconnection attempts (0 = infinite) | `0` |
| `initial_reconnect_delay_ms` | Initial reconnect delay in ms | `1000` |
| `max_reconnect_delay_ms` | Max reconnect delay in ms (exponential backoff) | `60000` |
| `max_message_size` | Max message size in bytes | `1048576` |

## Messaging Interface

The provider uses the standard `wasmcloud:messaging@0.2.0` interface to forward SSE events to components. Each SSE event is wrapped in a `broker-message`:

```wit
// From wasmcloud:messaging@0.2.0
interface types {
    record broker-message {
        subject: string,       // "sse.<url>" identifying the source connection
        body: list<u8>,        // Raw SSE event data bytes
        reply-to: option<string>,
    }
}

interface handler {
    use types.{broker-message};
    handle-message: func(msg: broker-message) -> result<_, string>;
}
```

Components export `wasmcloud:messaging/handler` to receive events. The `subject` field is set to `sse.<sse_url>` so the component knows which connection the event came from. The `body` contains the data field(s) of the SSE event.

### Linking

```bash
# Create named config
wash config put sse-config \
  sse_url=http://127.0.0.1:8765/events

# Link component to provider using wasmcloud:messaging
wash link put <component-id> <provider-id> \
  wasmcloud messaging \
  --interface handler \
  --target-config sse-config
```

Or via WADM:

```yaml
# Link defined on the component (source) to the provider (target)
- type: link
  properties:
    target:
      name: sse-provider
      config:
        - name: sse-config
          properties:
            sse_url: https://example.com/events
    namespace: wasmcloud
    package: messaging
    interfaces: [handler]
```

## Architecture

```
SSE Server
    │ HTTP streaming (text/event-stream)
    ▼
SSE Provider (Rust + reqwest)
    │ wRPC calls via wasmcloud:messaging/handler (over NATS)
    ▼
wasmCloud Component (WebAssembly)
    exports wasmcloud:messaging/handler
```