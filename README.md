# Ternary Routing — Self-Optimizing Request Routing with Ternary Feedback

**Ternary Routing** implements adaptive request routing where routes accumulate ternary feedback: **+1 (Good)** boosts the route, **-1 (Bad)** penalizes it, **0 (Neutral)** leaves it unchanged. Over time, traffic naturally converges to the optimal distribution without central configuration or static weights.

## Why It Matters

Static routing configurations become stale as conditions change: a fast route becomes slow, a reliable server degrades. Ternary routing adapts automatically by tracking per-route performance as accumulated ternary scores. Good responses boost the route; bad responses penalize it; neutral responses maintain the status quo. The key insight is that ternary feedback (3 states) captures enough signal for convergence while being simpler than continuous scores — no tuning of learning rates, no gradient computation. The softmax-style weight update ensures that well-performing routes get exponentially more traffic without ever starving alternatives entirely.

## How It Works

### Route State

Each `Route` tracks:
- `score`: Cumulative ternary feedback (starts at 0)
- `requests`: Total count
- `good` / `bad`: Per-feedback counts
- `weight`: Softmax-style weight for selection

### Feedback Application

```
Good (+1):    score += 1, good += 1
Bad (-1):     score -= 1, bad += 1
Neutral (0):  no change
weight = 1 + exp(score × 0.1)   // exponential boost/penalty
```

The `0.1` scaling factor controls how quickly routes converge: higher = faster but more volatile. Weight is O(1) per feedback update.

### Route Selection

**Greedy**: Always pick the route with highest score. O(r) for r routes. Exploitative — no exploration.

**Weighted**: Pick route proportional to weight. O(r) with rejection sampling. Balances exploitation (high-weight routes) and exploration (low-weight routes get occasional traffic).

### Rebalancing

`rebalance()` normalizes all weights to sum to 1.0, preventing weight inflation. Called periodically (every N requests). O(r).

### Convergence

In steady state with consistent feedback, routes converge to an exponential distribution: the best route gets ~80% of traffic, second-best ~15%, etc. This is the optimal explore/exploit balance for adversarial routing.

## Quick Start

```rust
use ternary_routing::{TernaryRouter, Feedback};

let mut router = TernaryRouter::new();
router.add_route("gpu-0");
router.add_route("gpu-1");
router.add_route("gpu-2");

// Route requests and record feedback
router.record("gpu-0", Feedback::Good);   // boost gpu-0
router.record("gpu-1", Feedback::Bad);    // penalize gpu-1

// Select best route
let best = router.select().unwrap();
println!("Best route: {}", best.name);

// Weighted selection for exploration
let weighted = router.weighted_select().unwrap();
```

```bash
cargo add ternary-routing
```

## API

| Type / Function | Description |
|---|---|
| `Feedback` | `Good(1)`, `Neutral(0)`, `Bad(-1)` |
| `Route` | `{ name, score, requests, good, bad, weight }` |
| `TernaryRouter` | `add_route()`, `select()`, `weighted_select()`, `record()`, `rebalance()` |

## Architecture Notes

Ternary routing directs traffic in **SuperInstance** fleet operations. Good routes (high γ = throughput) accumulate positive score; bad routes (high η = failures) accumulate negative score. The γ + η = C conservation manifests in the total traffic: the router distributes all requests, balancing growth (throughput) against entropy (failures). See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Awerbuch, Baruch & Kleinberg, Robert. "Adaptive Routing with End-to-End Feedback," *STOC*, 2004.
- Krishnan, S. et al. "Moving Beyond End-to-End Path Information," *NSDI*, 2009.
| Mitzenmacher, Michael & Upfal, Eli. *Probability and Computing*, Cambridge UP, 2017.

## License

Apache-2.0
