//! The public API as a caller sees it: the R package osmnxr is the first one.

use osmnxr_core::centrality::{betweenness_centrality, closeness_centrality};
use osmnxr_core::graph::Graph;
use osmnxr_core::metrics::{basic_stats, bearing, orientation_entropy};
use osmnxr_core::routing::{
    csp_lagrangian, csr_edge_order, dijkstra, distance_matrix, k_shortest_paths, pareto_paths,
    shortest_path_route, tsp_tour,
};
use osmnxr_core::simplify::{connected_components, simplify_paths};
use osmnxr_core::spatial::SegmentIndex;

/// A two-way chain 0 - 1 - 2 - 3 plus a two-way shortcut 0 - 3 that is longer
/// but cheaper in the second weight.
fn chain() -> (Vec<usize>, Vec<usize>, Vec<f64>, Vec<f64>) {
    let mut from = vec![];
    let mut to = vec![];
    let mut w1 = vec![];
    let mut w2 = vec![];
    let mut add = |a, b, x, y| {
        for (u, v) in [(a, b), (b, a)] {
            from.push(u);
            to.push(v);
            w1.push(x);
            w2.push(y);
        }
    };
    add(0, 1, 1.0, 5.0);
    add(1, 2, 1.0, 5.0);
    add(2, 3, 1.0, 5.0);
    add(0, 3, 4.0, 1.0);
    (from, to, w1, w2)
}

#[test]
fn routing_reports_paths_costs_and_traversed_edges() {
    let (from, to, w1, _) = chain();
    let g = Graph::from_edges(&from, &to, &w1, 4);
    assert_eq!(g.n_edges(), 8);

    let d = dijkstra(&g, 0);
    assert_eq!(d, vec![0.0, 1.0, 2.0, 3.0]);

    let (path, slots, cost) = shortest_path_route(&g, 0, 3).unwrap();
    assert_eq!(path, vec![0, 1, 2, 3]);
    assert_eq!(cost, 3.0);
    let order = csr_edge_order(&from, 4);
    let edges: Vec<usize> = slots.iter().map(|&s| order[s]).collect();
    // input-order indices of 0->1, 1->2, 2->3
    assert_eq!(edges, vec![0, 2, 4]);

    let k = k_shortest_paths(&g, &from, &to, &w1, 0, 3, 5);
    assert_eq!(k.len(), 2);
    assert_eq!(k[1].path, vec![0, 3]);
    assert_eq!(k[1].cost, 4.0);

    let m = distance_matrix(&g, &[0, 3], &[0, 3]);
    assert_eq!(m, vec![0.0, 3.0, 3.0, 0.0]);
}

#[test]
fn unreachable_targets_are_not_invented() {
    let g = Graph::from_edges(&[0], &[1], &[1.0], 3);
    assert!(shortest_path_route(&g, 0, 2).is_none());
    assert_eq!(dijkstra(&g, 0)[2], f64::INFINITY);
    assert!(k_shortest_paths(&g, &[0], &[1], &[1.0], 0, 2, 2).is_empty());
}

#[test]
fn pareto_front_and_budget_agree() {
    let (from, to, w1, w2) = chain();
    let front = pareto_paths(&from, &to, &w1, &w2, 4, 0, 3, f64::INFINITY, usize::MAX);
    let pairs: Vec<(f64, f64)> = front.iter().map(|r| (r.cost, r.resource)).collect();
    assert_eq!(pairs, vec![(3.0, 15.0), (4.0, 1.0)]);

    let within = pareto_paths(&from, &to, &w1, &w2, 4, 0, 3, 10.0, usize::MAX);
    assert_eq!(within.len(), 1);
    assert_eq!(within[0].path, vec![0, 3]);

    let larac = csp_lagrangian(&from, &to, &w1, &w2, 4, 0, 3, 10.0, 50).unwrap();
    assert_eq!(larac.path, vec![0, 3]);
    assert!(larac.lower_bound <= larac.cost + 1e-9);
    assert!(csp_lagrangian(&from, &to, &w1, &w2, 4, 0, 3, 0.5, 50).is_none());
}

#[test]
fn tsp_tour_is_a_permutation_from_start() {
    let n = 6;
    let cost: Vec<f64> = (0..n * n)
        .map(|k| {
            let (i, j) = (k / n, k % n);
            ((i as f64 - j as f64).abs() + if i < j { 0.5 } else { 0.0 }).max(0.0)
        })
        .collect();
    let tour = tsp_tour(&cost, n, 2, None, true, true);
    assert_eq!(tour.len(), n);
    assert_eq!(tour[0], 2);
    let mut sorted = tour.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..n).collect::<Vec<_>>());
}

#[test]
fn centrality_and_metrics() {
    let (from, to, w1, _) = chain();
    let g = Graph::from_edges(&from, &to, &w1, 4);
    let b = betweenness_centrality(&g, false);
    assert!(b[1] > 0.0 && b[2] > 0.0);
    assert!((b[1] - b[2]).abs() < 1e-12, "the chain is symmetric");
    let c = closeness_centrality(&g, true);
    assert!(c.iter().all(|&x| x > 0.0 && x <= 1.0));

    let s = basic_stats(&from, &to, &w1, 4);
    assert_eq!((s.n_nodes, s.n_edges), (4, 8));
    assert_eq!(s.total_length, 14.0);

    assert!((bearing(0.0, 0.0, 1.0, 0.0) - 0.0).abs() < 1e-9);
    assert!((bearing(0.0, 0.0, 0.0, 1.0) - 90.0).abs() < 1e-9);
    let grid = orientation_entropy(&[0.0, 90.0, 180.0, 270.0], 36);
    let organic = orientation_entropy(&(0..36).map(|k| k as f64 * 10.0).collect::<Vec<_>>(), 36);
    assert!(grid < organic);
}

#[test]
fn simplification_finds_chains_and_components() {
    // 0 - 1 - 2 - 3 (two-way) and an isolated pair 4 - 5
    let from = [0, 1, 1, 2, 2, 3, 4, 5];
    let to = [1, 0, 2, 1, 3, 2, 5, 4];
    let chains = simplify_paths(&from, &to, 6);
    assert!(chains.contains(&vec![0, 1, 2, 3]) || chains.contains(&vec![3, 2, 1, 0]));
    let comp = connected_components(&from, &to, 6);
    assert_eq!(comp[0], comp[3]);
    assert_ne!(comp[0], comp[4]);
    assert_eq!(comp[4], comp[5]);
}

#[test]
fn snapping_picks_the_nearest_polyline() {
    // edge 0: horizontal from (0,0) to (10,0); edge 1: vertical from (20,0) to (20,10)
    let vx = [0.0, 10.0, 20.0, 20.0];
    let vy = [0.0, 0.0, 0.0, 10.0];
    let offsets = [0, 2, 4];
    let index = SegmentIndex::build(&vx, &vy, &offsets);
    let s = index.nearest(3.0, 1.0, 5.0).unwrap();
    assert_eq!(s.edge, 0);
    assert!((s.dist - 1.0).abs() < 1e-12);
    assert!((s.x - 3.0).abs() < 1e-12 && s.y.abs() < 1e-12);
    assert!((s.pos - 3.0).abs() < 1e-12);
    let t = index.nearest(21.0, 7.0, 5.0).unwrap();
    assert_eq!(t.edge, 1);
    assert!((t.pos - 7.0).abs() < 1e-12);
    assert!(index.nearest(50.0, 50.0, 5.0).is_none());
}
