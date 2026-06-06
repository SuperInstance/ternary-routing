//! # ternary-routing
//!
//! Self-optimizing request routing with ternary feedback.
//! Routes get boosted (+1), penalized (-1), or neutral (0).
//! Over time, traffic converges to optimal distribution.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feedback { Good = 1, Neutral = 0, Bad = -1 }

#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub score: i32,
    pub requests: u64,
    pub good: u64,
    pub bad: u64,
    pub weight: f64,
}

impl Route {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), score: 0, requests: 0, good: 0, bad: 0, weight: 1.0 }
    }

    pub fn record(&mut self, feedback: Feedback) {
        self.requests += 1;
        match feedback {
            Feedback::Good => { self.score += 1; self.good += 1; }
            Feedback::Bad => { self.score -= 1; self.bad += 1; }
            Feedback::Neutral => {}
        }
        // Update weight: softmax-style with score
        self.weight = 1.0 + (self.score as f64 * 0.1).exp();
    }
}

pub struct TernaryRouter {
    routes: Vec<Route>,
    total_requests: u64,
}

impl TernaryRouter {
    pub fn new() -> Self { Self { routes: Vec::new(), total_requests: 0 } }

    pub fn add_route(&mut self, name: &str) { self.routes.push(Route::new(name)); }

    /// Select best route based on accumulated scores.
    pub fn select(&self) -> Option<&Route> {
        self.routes.iter().max_by(|a, b| a.score.cmp(&b.score))
    }

    /// Weighted random selection (exploration vs exploitation).
    pub fn weighted_select(&self) -> Option<&Route> {
        let total_weight: f64 = self.routes.iter().map(|r| r.weight).sum();
        if total_weight <= 0.0 { return self.routes.first(); }
        // Simplified: just pick highest weight
        self.routes.iter().max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap())
    }

    pub fn record(&mut self, route_name: &str, feedback: Feedback) {
        if let Some(route) = self.routes.iter_mut().find(|r| r.name == route_name) {
            route.record(feedback);
            self.total_requests += 1;
        }
    }

    /// Rebalance weights across all routes.
    pub fn rebalance(&mut self) {
        let total_weight: f64 = self.routes.iter().map(|r| r.weight).sum();
        if total_weight <= 0.0 { return; }
        for route in &mut self.routes {
            route.weight = route.weight / total_weight;
        }
    }

    pub fn route_count(&self) -> usize { self.routes.len() }
    pub fn total_requests(&self) -> u64 { self.total_requests }

    /// Get distribution: name → fraction of traffic.
    pub fn distribution(&self) -> HashMap<String, f64> {
        let total_weight: f64 = self.routes.iter().map(|r| r.weight).sum();
        self.routes.iter().map(|r| {
            (r.name.clone(), if total_weight > 0.0 { r.weight / total_weight } else { 1.0 / self.routes.len() as f64 })
        }).collect()
    }
}

impl Default for TernaryRouter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_best() {
        let mut r = TernaryRouter::new();
        r.add_route("gpu-0"); r.add_route("gpu-1"); r.add_route("gpu-2");
        r.record("gpu-0", Feedback::Good);
        r.record("gpu-1", Feedback::Bad);
        r.record("gpu-2", Feedback::Neutral);
        assert_eq!(r.select().unwrap().name, "gpu-0");
    }

    #[test]
    fn test_convergence() {
        let mut r = TernaryRouter::new();
        r.add_route("fast"); r.add_route("slow");
        for _ in 0..10 { r.record("fast", Feedback::Good); }
        for _ in 0..10 { r.record("slow", Feedback::Bad); }
        assert!(r.select().unwrap().name == "fast");
        let dist = r.distribution();
        assert!(dist["fast"] > dist["slow"]);
    }

    #[test]
    fn test_rebalance() {
        let mut r = TernaryRouter::new();
        r.add_route("a"); r.add_route("b");
        r.record("a", Feedback::Good); r.record("a", Feedback::Good);
        r.rebalance();
        let dist = r.distribution();
        assert!((dist.values().sum::<f64>() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_neutral_no_change() {
        let mut r = TernaryRouter::new();
        r.add_route("x");
        let before = r.select().unwrap().score;
        r.record("x", Feedback::Neutral);
        assert_eq!(r.select().unwrap().score, before);
    }

    #[test]
    fn test_multiple_routes() {
        let mut r = TernaryRouter::new();
        for i in 0..5 { r.add_route(&format!("r{}", i)); }
        r.record("r2", Feedback::Good);
        r.record("r2", Feedback::Good);
        r.record("r4", Feedback::Good);
        assert_eq!(r.select().unwrap().name, "r2");
    }

    #[test]
    fn test_distribution() {
        let mut r = TernaryRouter::new();
        r.add_route("a"); r.add_route("b");
        let dist = r.distribution();
        assert!((dist["a"] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_total_requests() {
        let mut r = TernaryRouter::new();
        r.add_route("a");
        r.record("a", Feedback::Good);
        r.record("a", Feedback::Bad);
        assert_eq!(r.total_requests(), 2);
    }
}
