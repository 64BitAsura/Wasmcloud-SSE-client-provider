# SSE Provider Testing

## Quick Test (Automated)

```bash
./tests/run_integration_test.sh
```

This will automatically start a test SSE server, build and deploy the provider and component, create links, monitor event flow for 30 seconds, and report results.

## Manual Test Steps

### Prerequisites

Python 3 (no additional packages needed for SSE server).

### Step 1: Start the Test SSE Server

```bash
python3 tests/sse_server.py
```

Server listens on `http://127.0.0.1:8765/events`, sends JSON events every 3 seconds.

### Step 2: Build Provider and Component

```bash
wash build
wash build -p ./component
```

The provider archive will be in `build/` (`.par.gz`), the component in `component/build/` (`.wasm`).

### Step 3: Start wasmCloud Host

```bash
wash up
```

Wait until `wash get hosts` shows a host ID.

### Step 4: Deploy Provider and Component

```bash
wash start provider file://./build/wasmcloud-provider-sse.par.gz sse-provider
wash start component file://./component/build/custom_template_test_component.wasm test-component
```

Verify both are running:

```bash
wash get inventory
```

### Step 5: Create Config and Link

```bash
# Create named config
wash config put sse-config \
  sse_url=http://127.0.0.1:8765/events \
  max_reconnect_attempts=0 \
  initial_reconnect_delay_ms=1000

# Link component to provider (using standard wasmcloud:messaging interface)
wash link put test-component sse-provider \
  wasmcloud messaging \
  --interface handler \
  --target-config sse-config
```

### Step 6: Verify

Check the wasmCloud host output for:

- `SSE connection established: 200 OK`
- `Received SSE event: ... bytes`
- `Message successfully sent to component ...`
- `Received message - Subject: sse.http://..., Size: ... bytes`

The SSE server terminal should show client connections and sent events.

## Using WADM

You can also deploy the full application declaratively:

```bash
wash up -d
wash app deploy ./wadm.yaml
```

## Testing Edge Cases

### Reconnection

1. Stop the SSE server (Ctrl+C)
2. Observe reconnection attempts in provider logs
3. Restart the server — provider reconnects automatically

### Message Size Limits

Set `max_message_size` in the config to enforce size limits. Events exceeding the limit are skipped.

## Cleanup

```bash
wash down
```

## Troubleshooting

| Problem | Check |
|---------|-------|
| Provider not connecting | Is the SSE server running? Check `sse_url` in config |
| Component not receiving events | Run `wash link query` and `wash get inventory` |
| NATS connection issues | Run `wash get hosts`, try `wash down && wash up` |

## Architecture

```
SSE Server (127.0.0.1:8765/events)
    │ HTTP streaming (text/event-stream)
    ▼
SSE Provider (Rust + reqwest)
    │ wRPC calls (via NATS)
    ▼
wasmCloud Component (WebAssembly)
```
