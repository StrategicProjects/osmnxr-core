//! A 3 x 3 street grid with one slow diagonal: shortest path, alternatives,
//! and the Pareto front of (distance, penalty) paths.
//!
//! Run with `cargo run --example grid`.

use osmnxr_core::graph::Graph;
use osmnxr_core::routing::{csp_lagrangian, k_shortest_paths, pareto_paths, shortest_path_route};

fn main() {
    // Nodes 0..9 laid out as
    //   0 - 1 - 2
    //   |   |   |
    //   3 - 4 - 5
    //   |   |   |
    //   6 - 7 - 8
    // Every street is two-way, so it is two directed edges.
    let mut from = Vec::new();
    let mut to = Vec::new();
    let mut length = Vec::new();
    let mut penalty = Vec::new();
    let mut street = |a: usize, b: usize, len: f64, pen: f64| {
        for (u, v) in [(a, b), (b, a)] {
            from.push(u);
            to.push(v);
            length.push(len);
            penalty.push(pen);
        }
    };
    for row in 0..3 {
        for col in 0..3 {
            let n = row * 3 + col;
            if col < 2 {
                street(n, n + 1, 100.0, 1.0);
            }
            if row < 2 {
                street(n, n + 3, 100.0, 1.0);
            }
        }
    }
    // A shortcut across the middle: shorter, but it carries a heavy penalty
    // (say, a steep hill or a toll).
    street(0, 4, 120.0, 10.0);
    street(4, 8, 120.0, 10.0);

    let g = Graph::from_edges(&from, &to, &length, 9);

    let (path, _slots, cost) = shortest_path_route(&g, 0, 8).unwrap();
    println!("shortest by length: {path:?}  {cost} m");

    let routes = k_shortest_paths(&g, &from, &to, &length, 0, 8, 3);
    for (i, r) in routes.iter().enumerate() {
        println!("alternative {}: {:?}  {} m", i + 1, r.path, r.cost);
    }

    println!("\nPareto front of (length, penalty):");
    let front = pareto_paths(
        &from,
        &to,
        &length,
        &penalty,
        9,
        0,
        8,
        f64::INFINITY,
        usize::MAX,
    );
    for r in &front {
        println!("  {:?}  {} m  penalty {}", r.path, r.cost, r.resource);
    }

    let budget = 5.0;
    let best = csp_lagrangian(&from, &to, &length, &penalty, 9, 0, 8, budget, 50).unwrap();
    println!(
        "\ncheapest path with penalty <= {budget}: {:?}  {} m  (lower bound {}, {} Dijkstra runs)",
        best.path, best.cost, best.lower_bound, best.iterations
    );
}
