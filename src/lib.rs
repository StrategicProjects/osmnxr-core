//! Street-network graph algorithms for OpenStreetMap-style networks.
//!
//! `osmnxr-core` is the compute core of the R package
//! [osmnxr](https://strategicprojects.github.io/osmnxr/), a port to R of
//! [OSMnx](https://osmnx.readthedocs.io/), the Python library by Geoff Boeing
//! (Boeing, 2025, doi:10.1111/gean.70009). The street-network model this
//! crate serves (simplification, intersection consolidation, bearings and
//! orientation metrics) is OSMnx's; the crate is not affiliated with or
//! endorsed by the OSMnx project. It
//! holds the graph math and nothing else: no I/O, no geometry types, no
//! coordinate reference systems. A caller hands it edge lists of 0-based node
//! indices with weights, and gets back node paths, edge indices, distances
//! and scores. The crate has no dependencies.
//!
//! # Modules
//!
//! - [`graph`]: a compressed-sparse-row (CSR) directed graph built from
//!   parallel `from`/`to`/`weight` vectors.
//! - [`routing`]: Dijkstra, shortest paths that report the edges traversed
//!   (parallel edges included), Yen's k shortest paths, distance matrices, a
//!   travelling-salesman heuristic, the Pareto front of bicriteria paths
//!   (Martins' label-setting, exact constrained shortest paths) and the LARAC
//!   Lagrangian relaxation for large graphs.
//! - [`centrality`]: betweenness (Brandes) and closeness.
//! - [`metrics`]: basic network statistics, compass bearings and the
//!   orientation entropy of a street grid.
//! - [`simplify`]: topology simplification (chains of degree-2 nodes between
//!   endpoints) and connected components.
//! - [`spatial`]: grid-indexed snapping of points to the nearest polyline.
//!
//! # Conventions
//!
//! Nodes are `0 .. n_nodes`; edges are directed, so a two-way street is two
//! edges. Weights must be nonnegative and finite. Functions that return edges
//! give indices into the caller's input order, so attributes and geometry can
//! be recovered exactly. Unreachable targets are `f64::INFINITY` distances,
//! empty paths or `None`, never a made-up value.
//!
//! # Example
//!
//! ```
//! use osmnxr_core::graph::Graph;
//! use osmnxr_core::routing::{dijkstra, shortest_path_route};
//!
//! // a square 0 -> 1 -> 2 -> 3 -> 0 with a diagonal 0 -> 2
//! let from = [0, 1, 2, 3, 0];
//! let to = [1, 2, 3, 0, 2];
//! let weight = [1.0, 1.0, 1.0, 1.0, 1.5];
//! let g = Graph::from_edges(&from, &to, &weight, 4);
//!
//! let dist = dijkstra(&g, 0);
//! assert_eq!(dist, vec![0.0, 1.0, 1.5, 2.5]);
//!
//! let (path, _slots, cost) = shortest_path_route(&g, 0, 3).unwrap();
//! assert_eq!(path, vec![0, 2, 3]);
//! assert_eq!(cost, 2.5);
//! ```

pub mod centrality;
pub mod graph;
pub mod metrics;
pub mod routing;
pub mod simplify;
pub mod spatial;

pub use graph::Graph;
