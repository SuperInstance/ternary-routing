# ternary-routing

Self-optimizing request routing with ternary feedback — routes get boosted (+1), penalized (-1), or stay neutral (0).

## Why This Exists

Load balancers typically use static weights or slow-converging adaptive algorithms. Ternary routing uses a minimal feedback signal: after each request, the route gets rated +1 (good), 0 (neutral), or -1 (bad). Over time, routes accumulate scores and traffic converges toward the best-performing backend. The ternary signal is coarse enough to be fast (no floating-point calculation) but expressive enough to differentiate healthy, degraded, and failing routes.

## Architecture

### Core Types

- **`Feedback`** — Ternary: `Good (+1)`, `Neutral (0)`, `Bad (-1)`.
- **`Route`** — A backend with accumulated score, request count, and weighted average.
- **`TernaryRouter`** — Collection of routes with `select()` (highest score), `weighted_select()` (probabilistic), and `rebalance()`.

### Selection Strategies

- **`select`**: Deterministic — always picks the highest-scoring route.
- **`weighted_select`**: Probabilistic — softmax over scores, exploration/exploitation.
- **`rebalance`**: Decay all scores toward zero, preventing stale leaders.

## Usage

```rust
use ternary_routing::{TernaryRouter, Feedback};

let mut router = TernaryRouter::new();
router.add_route("gpu-node-1");
router.add_route("gpu-node-2");
router.add_route("gpu-node-3");

// Route traffic and collect feedback
let route = router.select().unwrap();
router.record("gpu-node-1", Feedback::Good);
router.record("gpu-node-2", Feedback::Bad);
router.record("gpu-node-3", Feedback::Neutral);

// Check distribution
let dist = router.distribution();
// gpu-node-1 gets the most traffic

// Periodically rebalance
router.rebalance();
```

## API Reference

| Method | Returns | Description |
|--------|---------|-------------|
| `new()` | `TernaryRouter` | Create empty router |
| `add_route(name)` | `()` | Register a backend |
| `select()` | `Option<&Route>` | Pick highest-scoring route |
| `weighted_select()` | `Option<&Route>` | Probabilistic selection |
| `record(route_name, feedback)` | `()` | Submit ternary feedback |
| `rebalance()` | `()` | Decay scores toward zero |
| `route_count()` | `usize` | Number of routes |
| `total_requests()` | `u64` | Requests routed |
| `distribution()` | `HashMap<String, f64>` | Traffic fractions per route |

## The Deeper Idea

This is **reinforcement learning stripped to its bones**. The ternary feedback is the reward signal (+1/0/-1), the route scores are the Q-values, and `select`/`weighted_select` are the policy. No neural networks, no gradient descent — just additive score tracking with decay. The ternary reward is the minimal information you need from the environment to learn which action is best. Anything more granular (e.g., latency in milliseconds) can be thresholded into these three buckets.

## Related Crates

- **ternary-routing** — this crate
- **ternary-backpressure** — upstream throttling based on downstream pressure
- **ternary-rate-limiter** — rate limiting with ternary feedback
- **ternary-dispatch** — kernel dispatch ordering
