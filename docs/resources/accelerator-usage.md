---
name: accelerator-usage
description: 'Understand how the Octane Accelerator integrates with the Octra wallet extension, how to use its HTTP/WebSocket API, run benchmarks, and configure remote proving via encrypted relay tunnels.'
---

# Octane Accelerator Usage

## Overview

The Octane Accelerator is a native desktop proving service that runs on
`127.0.0.1:19876`. It performs PVAC (Private Verifiable Arithmetic Commitments)
operations — encrypting values, generating range proofs, zero proofs, and
decrypting balances — for the Octra wallet browser extension. The extension
connects to the accelerator locally or via an encrypted relay tunnel for remote
usage.

## HTTP API

### Health Check

```http
GET /health → {"status":"ready"}
```

### Decrypt Balance

```http
POST /decrypt
Content-Type: application/json

{
  "cipher_b64": "<hfhe_v1|base64 ciphertext>",
  "pvac_sk_b64": "<base64 PVAC secret key>",
  "pvac_pk_b64": "<base64 PVAC public key>"
}

→ {"value": 1000000}
```

### Benchmark (Range Proof)

```http
POST /benchmark
Content-Type: application/json

{
  "op": "range_proof",
  "value": 42
}

→ {"ok": true, "elapsed_ms": 1234.5, "operation": "range_proof", "value": 42}
```

The benchmark endpoint generates an ephemeral keypair, encrypts a test value,
and times a full range proof generation. No keys or secrets are required — it's
self-contained.

### Export Pairing File

```http
POST /pair/export → <pairing file text>
```

Generates a new relay pairing identity and returns the `.pair` file contents.

## WebSocket Proving API

```http
WS /prove
```

Send a JSON message with the operation:

```json
{
  "operation": "shield|stealth|claim|decrypt|encrypt|range_proof|zero_proof|commit|ct_sub",
  "pvac_sk_b64": "<base64 PVAC secret key>",
  "pvac_pk_b64": "<base64 PVAC public key>",
  "amountRaw": "1000000",
  "seedB64": "<base64 32-byte seed>",
  "blindingB64": "<base64 32-byte blinding factor>"
}
```

The server streams status updates:

```json
{"type": "status", "step": "Encrypting value..."}
{"type": "status", "step": "Generating zero proof..."}
{"type": "result", "data": { ... }}
```

### Operations

| Operation | Purpose | Extra Fields |
| --- | --- | --- |
| `shield` | Encrypt amount + commitment + zero proof | — |
| `stealth` | Send to another user (delta + range proofs) | `currentCipherB64` |
| `claim` | Claim received funds | — |
| `decrypt` | Decrypt a ciphertext to plaintext value | `cipher_b64` |
| `encrypt` | Encrypt a value | — |
| `range_proof` | Generate standalone range proof | `cipher_b64` |
| `zero_proof` | Generate zero-knowledge zero proof | `cipher_b64` |
| `commit` | Pedersen commitment | — |
| `ct_sub` | Ciphertext subtraction | `a_b64`, `b_b64` |

## Remote Proving via Encrypted Relay

The accelerator supports remote proving through a Cloudflare-hosted relay at
`wss://octane-relay.octane-fast.workers.dev`. This allows a wallet on a phone
or remote machine to use a prover running on a desktop.

### Security Model

- **End-to-end encrypted**: Communication uses X25519 ECDH key agreement +
  NaCl secretbox (XSalsa20-Poly1305). The relay server cannot read message
  contents.
- **Private rooms**: Each pairing generates a random 128-bit room ID. Only
  devices with the room ID can join.
- **Key isolation**: The X25519 secret key never leaves the accelerator machine
  (stored in `~/.octane/relay_secret.key`).
- **Pairing file**: Contains only the relay URL, room ID, and the accelerator's
  public key — enough for the wallet to establish an encrypted channel but not
  to impersonate the prover.

### Pairing Flow

1. User clicks "Export Pairing File…" in the tray menu (or calls
   `POST /pair/export`).
2. A `.pair` file is generated containing relay URL, room ID, and public key.
3. User transfers the file to their remote device and imports it into the Octra
   wallet.
4. The wallet connects to the relay room and establishes an encrypted session
   using the public key.
5. All prove requests and responses are encrypted end-to-end through the relay.

### Configuration Files

- `~/.octane/relay.conf` — relay URL and room ID (loaded on startup)
- `~/.octane/relay_secret.key` — 64 bytes: X25519 secret key (32) + public key
  (32)
- `~/.octane/accelerator.log` — server stdout/stderr log

## System Tray App (macOS)

The tray app (`tray/main.m`) is a macOS menu bar application that:

- Starts/stops the accelerator server process
- Shows running status (● Running / ○ Stopped)
- Provides "Export Pairing File…" to set up remote proving
- Provides "Run Benchmark" to test range proof speed with a progress bar
- Registers as a login item (auto-start on boot)
- Auto-restarts the server if it crashes (health check every 5s)

## Integration with Wallet Extension

The Octra wallet extension connects to the accelerator in two ways:

1. **Local**: Direct HTTP/WebSocket to `127.0.0.1:19876` when on the same
   machine.
2. **Remote**: Through the encrypted relay tunnel when the wallet has imported
   a pairing file.

The extension sends PVAC keys with each request — the accelerator is stateless
and does not store wallet keys.

## Building

```bash
# Build the server binary
make

# Compile the tray app (macOS only)
clang -framework Cocoa -framework ServiceManagement \
  -framework UniformTypeIdentifiers -fobjc-arc -O2 \
  -o tray_app tray/main.m
```

## Testing the Benchmark

```bash
# Start the accelerator
./octane-accelerator

# In another terminal, run a benchmark
curl -s -X POST http://127.0.0.1:19876/benchmark \
  -H "Content-Type: application/json" \
  -d '{"op":"range_proof","value":42}'
```
