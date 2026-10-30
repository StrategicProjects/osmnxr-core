# osmnxr-core

[![CI](https://github.com/StrategicProjects/osmnxr-core/actions/workflows/ci.yml/badge.svg)](https://github.com/StrategicProjects/osmnxr-core/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/osmnxr-core.svg)](https://crates.io/crates/osmnxr-core)
[![docs.rs](https://img.shields.io/docsrs/osmnxr-core)](https://docs.rs/osmnxr-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
![No dependencies](https://img.shields.io/badge/dependencies-none-brightgreen.svg)

Street-network graph algorithms for OpenStreetMap-style networks, with no
dependencies. This crate is the compute core of
[osmnxr](https://strategicprojects.github.io/osmnxr/), the R package that
brings [OSMnx](https://osmnx.readthedocs.io/) to R: the R layer downloads,
models and draws; this crate does the graph math.

It knows nothing about OpenStreetMap, geometry or coordinate systems. A caller
hands it edge lists of 0-based node indices with weights and gets back node
paths, edge indices, distances and scores. Everything it returns points back
into the caller's own data, so attributes and geometry can be recovered
exactly, parallel edges included.

**Website:** <https://strategicprojects.github.io/osmnxr-core/> ·
**API:** <https://strategicprojects.github.io/osmnxr-core/api/osmnxr_core/>

## What is in it

| Module | Contents |
|---|---|
| `graph` | `Graph`: a compressed-sparse-row directed graph built from parallel `from`/`to`/`weight` vectors. |
| `routing` | Dijkstra; shortest paths that report the edges traversed; Yen's k loopless shortest paths; distance matrices; a travelling-salesman heuristic (nearest neighbour + iterated local search, exact moves on asymmetric matrices, fixed seed); the Pareto front of bicriteria paths by Martins' label-setting, which with a budget is the exact constrained shortest path; and the LARAC Lagrangian relaxation, one Dijkstra per iteration with a proven lower bound, for graphs where exact search is slow. |
| `centrality` | Betweenness (Brandes) and closeness, with the usual normalisations. |
| `metrics` | Basic network statistics, compass bearings and the orientation entropy of a street grid. |
| `simplify` | Topology simplification (maximal chains of degree-2 nodes between endpoints) and connected components. |
| `spatial` | A uniform-grid index that snaps points to the nearest polyline, with the position along it. |

## Example

```rust
use osmnxr_core::graph::Graph;
use osmnxr_core::routing::{dijkstra, pareto_paths, shortest_path_route};

// a square 0 -> 1 -> 2 -> 3 -> 0 with a diagonal 0 -> 2
let from = [0, 1, 2, 3, 0];
let to = [1, 2, 3, 0, 2];
let length = [1.0, 1.0, 1.0, 1.0, 1.5];
let g = Graph::from_edges(&from, &to, &length, 4);

assert_eq!(dijkstra(&g, 0), vec![0.0, 1.0, 1.5, 2.5]);
let (path, _edge_slots, cost) = shortest_path_route(&g, 0, 3).unwrap();
assert_eq!((path, cost), (vec![0, 2, 3], 2.5));

// two criteria: length and, say, a penalty for the diagonal
let penalty = [0.0, 0.0, 0.0, 0.0, 5.0];
let front = pareto_paths(&from, &to, &length, &penalty, 4, 0, 3, f64::INFINITY, usize::MAX);
assert_eq!(front.len(), 2); // (2.5, 5.0) through the diagonal, (3.0, 0.0) around
```

`cargo run --example grid` walks a 3 × 3 grid through shortest path,
alternatives, the Pareto front and a budgeted query.

## Install

```sh
cargo add osmnxr-core
```

Rust 1.81 or later. No dependencies, no features, no `unsafe`.

## Conventions

- Nodes are `0 .. n_nodes`. Edges are directed: a two-way street is two edges.
- Weights are nonnegative and finite. The caller decides what they mean
  (metres, seconds, a penalty).
- Functions that return edges give indices into the caller's input order.
  `routing::shortest_path_route` returns CSR slots instead; map them with
  `routing::csr_edge_order` when you need input order.
- Unreachable is `f64::INFINITY`, an empty path or `None`. No value is made up.

## How it is checked

- The Pareto front is compared with brute-force enumeration of every simple
  path on random graphs.
- The travelling-salesman heuristic was benchmarked from R against
  `TSP::solve_TSP` on the same cost matrices: 54 wins, 26 ties and no losses
  over 80 instances of 8 to 50 stops, matching brute force where feasible.
- The LARAC solver found the exact optimum in every one of eight scenarios on
  the Delhi road network (1.0 million nodes, 2.1 million directed edges); the
  exact front is still the default in osmnxr, LARAC the fallback for size.
- CI runs `cargo fmt`, `clippy -D warnings`, the tests and the docs on Linux,
  macOS and Windows, on the oldest supported Rust (1.81) and for
  `wasm32-unknown-unknown`.

## Family

- [osmnxr](https://github.com/StrategicProjects/osmnxr): the R package (CRAN),
  which vendors this crate and calls it through
  [extendr](https://extendr.github.io/).

## Acknowledgements

The design follows [OSMnx](https://github.com/gboeing/osmnx) by Geoff Boeing.
The algorithms come from the literature: Dijkstra (1959), Yen (1971), Brandes
(2001), Martins (1984) for the bicriteria front, Jüttner et al. (2001) for
LARAC, and Martin, Otto and Felten (1991) for the double-bridge kick.

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).
