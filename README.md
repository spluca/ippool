# IP Pool API

REST API for managing IP address allocation in /24 networks, written in Rust with Axum.

## Features

- Thread-safe IP allocation using Tokio RwLock
- Guaranteed unique IPs (no collisions)
- VM_ID to IP mapping with idempotent operations
- RESTful API with async/await
- Statistics and monitoring
- Docker support with multi-stage build

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/ippool

# With custom configuration
./target/release/ippool --network 192.168.1 --gateway 192.168.1.1 --port 9000

# Development mode
cargo run
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/health` | Health check |
| POST | `/api/v1/ip/allocate` | Allocate IP for VM |
| DELETE | `/api/v1/ip/release/{vm_id}` | Release IP by VM ID |
| DELETE | `/api/v1/ip/release-by-ip/{ip}` | Release IP by address |
| GET | `/api/v1/ip/{vm_id}` | Get allocation for VM |
| GET | `/api/v1/ip/allocations` | List all allocations |
| GET | `/api/v1/ip/stats` | Get pool statistics |

### Example: Allocate IP

```bash
curl -X POST http://localhost:8080/api/v1/ip/allocate \
  -H "Content-Type: application/json" \
  -d '{"vm_id": "srv-abc123", "hostname": "my-vm"}'
```

**Response (201 Created):**

```json
{
  "ip": "172.16.0.2",
  "vm_id": "srv-abc123",
  "gateway": "172.16.0.1",
  "network": "172.16.0.0/24",
  "hostname": "my-vm"
}
```

**Note:** Operations are idempotent - calling with the same `vm_id` returns the existing allocation.

## Configuration

```bash
Usage: ippool [OPTIONS]

Options:
  -p, --port <PORT>          Port to listen on [default: 8080]
  -n, --network <NETWORK>    Network prefix (e.g., 172.16.0) [default: 172.16.0]
  -g, --gateway <GATEWAY>    Gateway IP address [default: 172.16.0.1]
  -d, --debug               Enable debug mode
  -h, --help                Print help
```

## Docker

### Build

```bash
docker build -t ippool:latest .
```

**Image size:** ~32 MB (~9 MB compressed)

### Run

```bash
# Default configuration
docker run -d -p 8080:8080 --name ippool ippool:latest

# Custom configuration
docker run -d -p 9000:9000 --name ippool \
  ippool:latest --port 9000 --network 192.168.1 --gateway 192.168.1.1

# With debug logging
docker run -d -p 8080:8080 -e RUST_LOG=debug --name ippool ippool:latest
```

## Architecture

```
┌────────────────────────────────┐
│  Axum HTTP Server (Tokio)     │
├────────────────────────────────┤
│  Handlers (REST + JSON)        │
├────────────────────────────────┤
│  IP Pool (RwLock)              │
│  - allocated: HashMap<IP, VM>  │
│  - vm_to_ip: HashMap<VM, IP>   │
│  - available: Vec<String>      │
└────────────────────────────────┘
```

## Error Handling

| Error | HTTP Status | Description |
|-------|-------------|-------------|
| No available IPs | 503 | Pool exhausted |
| VM ID not found | 404 | No allocation exists |
| Invalid IP | 400 | IP not in network |
| Invalid request | 400 | Missing/invalid parameters |

## Technology Stack

- **Rust 2024** - Memory safety without GC
- **Axum 0.8.7** - Type-safe async HTTP framework
- **Tokio 1.48.0** - Async runtime
- **Serde 1.0** - JSON serialization
- **Clap 4.5** - CLI parsing
- **Tower/Tower-HTTP** - Middleware (CORS, tracing)

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_allocate_ip

# With output
cargo test -- --nocapture

# With coverage
cargo tarpaulin --out Html
```

**Test coverage:** 9 tests covering allocation, deallocation, idempotency, concurrency, and error handling.

## Performance

- **Latency:** Sub-millisecond response times
- **Throughput:** Thousands of requests/second
- **Memory:** No GC pauses, predictable usage
- **Concurrency:** Lock-free reads, async-safe writes

## Project Structure

```
ippool/
├── Cargo.toml       # Dependencies
├── Dockerfile       # Multi-stage build
├── .dockerignore    # Build optimization
└── src/
    ├── main.rs      # Server & routing
    ├── handlers.rs  # HTTP handlers
    └── ippool.rs    # Core logic + tests
```

## Integration Example

```bash
# Start server
./ippool --network 172.16.0 --gateway 172.16.0.1 &

# Allocate IP for VM
VM_IP=$(curl -s -X POST http://localhost:8080/api/v1/ip/allocate \
  -H "Content-Type: application/json" \
  -d "{\"vm_id\":\"$VM_ID\"}" | jq -r .ip)

# Use IP in VM setup...

# Release IP on cleanup
curl -X DELETE http://localhost:8080/api/v1/ip/release/$VM_ID
```

## License

Same as parent project.
