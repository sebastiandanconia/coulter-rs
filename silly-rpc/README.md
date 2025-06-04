# silly-rpc

A deliberately silly example of exposing the **same** business logic over **gRPC** and **REST** from one Rust binary — so you can copy it for your serious RPC needs.

The "business logic" hands out two constants: the secp256k1 curve prime and the Bitcoin genesis block hash. We expose them at`POST /api/btc-constants/v1` — named to match the gRPC service (`BtcConstantService`), so the two transports agree on naming as well as on values.

gRPC and REST both call the same functions in the library crate, so they can never disagree with each other. That's the whole point.

## The three programs

Everything that is actually *served* (protobuf messages, gRPC service impl, REST router/handler) lives in the `silly_rpc` library. The three programs below are just *orchestration* on top of it, so pick whichever shape you need:

| Program | Serves | Run |
|---|---|---|
| `src/main.rs` (the binary) | gRPC **and** REST at once | `cargo run` |
| `examples/grpc_only.rs` | gRPC only | `cargo run --example grpc_only` |
| `examples/rest_only.rs` | REST only | `cargo run --example rest_only` |

Ports: gRPC on `0.0.0.0:50051`, REST on `0.0.0.0:3000`.

## Prerequisites

- Rust (stable) and `cargo`.
- `protoc` (the protobuf compiler) — on Debian/Ubuntu: `apt-get install protobuf-compiler`.
- Python 3 with `grpcio`, `grpcio-tools`, `protobuf`, `requests` for the test
  clients:
  ```sh
  pip3 install --break-system-packages grpcio grpcio-tools protobuf requests
  ```

The Python test clients import `btc_constants_pb2` / `btc_constants_pb2_grpc`, which aren't checked in. Generate them once:

```sh
python3 -m grpc_tools.protoc -I src/proto --python_out=. --grpc_python_out=. src proto/btc_constants.proto
```

## Run it

```sh
cargo run
# gRPC listening on http://0.0.0.0:50051
# REST listening on http://0.0.0.0:3000
```

## Test it

**gRPC** (needs the `btc_constants_pb2*.py` generated above):

```sh
python3 grpc_test_utility.py --transaction_id tx123 --client_id cli456
```

**REST over JSON** with `curl`:

```sh
curl -s -X POST http://localhost:3000/api/btc-constants/v1 \
  -H 'Content-Type: application/json' \
  -d '{"transaction-id":"tx123","client-id":"cli456"}'
```

**REST over raw protobuf** (bonus — same endpoint, different `Content-Type`):

```sh
python3 protobuf_test_utility.py --transaction_id tx123 --client_id cli456
```

Whichever transport you use, you should get back the same two constants:

- `sec256k1-prime` = `fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f`
- `btc-genesis-hash` = `000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f`

(The JSON client hex-encodes them; the gRPC/protobuf clients print the raw bytes as octal escapes. Same bytes, different avatars.)

## Layout

```
src/lib.rs            shared business logic (the BTC constants) + module glue
src/proto/            btc_constants.proto + generated Rust bindings (build.rs)
src/grpc_service.rs   BtcConstantServiceImpl — the gRPC service
src/rest.rs           AppState + build_router + the REST handler
src/main.rs           binary: gRPC + REST together (tokio::select!)
examples/grpc_only.rs gRPC only
examples/rest_only.rs REST only
grpc_test_utility.py     gRPC test client
protobuf_test_utility.py protobuf-over-HTTP test client
```

## But I want both on ONE port!

The two-port approach above is the boring, robust, production-shaped answer. Serving gRPC (HTTP/2) and REST (HTTP/1.1) on a single port by multiplexing on `Content-Type` is possible but finicky. Starting hints are in the comment at the bottom of `src/main.rs`. Good luck, you brave soul.
