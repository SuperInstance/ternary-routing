# ternary-routing

Self-optimizing request routing with ternary feedback. Routes get boosted (+1), penalized (-1), or unchanged (0) based on latency/success. Routes converge to optimal without central config.

## Why This Matters

# ternary-routing
Self-optimizing request routing with ternary feedback.
Routes get boosted (+1), penalized (-1), or neutral (0).
Over time, traffic converges to optimal distribution.

## The Five-Layer Stack

This crate is part of the **Oxide Stack** — a distributed GPU runtime built on five layers:

```
┌─────────────────┐
│  cudaclaw        │  Persistent GPU kernels, warp consensus, SmartCRDT
├─────────────────┤
│  cuda-oxide      │  Flux → MIR → Pliron → NVVM → PTX compiler
├─────────────────┤
│  flux-core       │  Bytecode VM + A2A agent protocol
├─────────────────┤
│  pincher         │  "Vector DB as runtime, LLM as compiler"
├─────────────────┤
│  open-parallel   │  Async runtime (tokio fork)
└─────────────────┘
```

The key insight: **ternary values {-1, 0, +1} map directly to GPU compute**. They pack 16× denser than FP32, enable XNOR+popcount matmul, and conservation laws become compile-time checks.

## Design

Every value in this crate follows **ternary algebra** (Z₃):

| Value | Meaning | GPU Analog |
|-------|---------|------------|
| +1 | Positive / Active / Healthy | Warp vote yes |
| 0 | Neutral / Pending / Balanced | Warp vote abstain |
| -1 | Negative / Failed / Overloaded | Warp vote no |

This isn't arbitrary — ternary is the natural encoding for:
1. **BitNet b1.58** (Microsoft) — ternary LLMs at 60% less power
2. **GPU warp voting** — hardware ballot returns ternary consensus
3. **Conservation laws** — {-1, 0, +1} preserves quantity

## Key Types

```rust
pub enum Feedback
pub struct Route
pub fn new
pub fn record
pub struct TernaryRouter
pub fn new
pub fn add_route
pub fn select
pub fn weighted_select
pub fn record
pub fn rebalance
pub fn route_count
```

## Usage

```toml
[dependencies]
ternary-routing = "0.1.0"
```

```rust
use ternary_routing::*;
// See src/lib.rs tests for complete working examples
```

## Testing

```bash
git clone https://github.com/SuperInstance/ternary-routing.git
cd ternary-routing
cargo test    # 7 tests
```

## Stats

| Metric | Value |
|--------|-------|
| Tests | 7 |
| Lines of Rust | 165 |
| Public API | 14 items |

## License

Apache-2.0
