# Changelog

## 0.1.0

First release. The crate is the Rust core that shipped inside the R package
osmnxr up to version 0.1.5, extracted so other callers can use it and so
the R package can pin it like any other dependency.

- `graph`: CSR directed graph from edge lists.
- `routing`: Dijkstra, shortest paths with traversed edges, Yen's k shortest
  paths, distance matrices, a travelling-salesman heuristic, the Pareto front
  of bicriteria paths (exact constrained shortest path with a budget) and the
  LARAC Lagrangian relaxation.
- `centrality`: betweenness (Brandes) and closeness.
- `metrics`: basic statistics, bearings, orientation entropy.
- `simplify`: degree-2 chains and connected components.
- `spatial`: grid-indexed snapping of points to polylines.
