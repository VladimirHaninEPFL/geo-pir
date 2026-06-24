# Geo-PIR: Private Information Retrieval for Navigation Shortest Paths

## Overview

Geo-PIR is a client-server application that enables private shortest path computation on geographic networks using Private Information Retrieval (PIR). The system allows a client to compute the shortest path between two coordinates on a geographic graph (e.g., road networks) without revealing the user's location to the server.

The architecture supports multiple PIR protocols (such as SinglePass and Spiral) and different server approaches.

### Key Features

- **Privacy-Preserving Shortest Path**: Clients can query shortest paths without revealing the start and end coordinates to the server
- **Multiple PIR Protocols**: Support for different PIR architectures (Naive, SinglePass, Spiral)
- **IPC Communication**: Uses Inter-Process Communication (IPC) for client-server communication
- **Benchmarking Tools**: Includes scripts and Python utilities for benchmarking and visualization of results

## Building

This is a Rust project built with Cargo. To build the release binaries:

```bash
cargo build --release
```

This will create two main binaries in `target/release/`:
- `geo_server`: The PIR server process
- `geo_client`: The client that queries for shortest paths

## Running Client and Server

The client and server communicate via IPC using a specified country name, PIR architecture, and approach.

### Example: Computing a shortest path with SinglePass

**Step 1: Start the server(s)**

For SinglePass architecture (requires two servers - left and right):

```bash
./target/release/geo_server Switzerland SinglePass block0.1 left &
./target/release/geo_server Switzerland SinglePass block0.1 right &
```

For other architectures like Spiral or Naive (single server):

```bash
./target/release/geo_server Switzerland Spiral block0.1  &
```

**Step 2: Run the client**

Query the shortest path between two nodes (specified by their OSM IDs):

```bash
./target/release/geo_client Switzerland SinglePass block0.1 START_NODE_ID END_NODE_ID
```

Replace `START_NODE_ID` and `END_NODE_ID` with actual OSM node IDs from the graph.

### Example Output

```
Running A* from 123456 to 789012 in country Switzerland using approach block0.1 and architecture SinglePass ...
A* found a path with cost 42.523456
Path length: 15 nodes
Path: [123456, 234567, 345678, ..., 789012]
```

## Directory Structure

### `src/`
Core library code:
- `lib.rs`: Main library module exports
- `client.rs`: Client implementation for querying shortest paths
- `server.rs`: Server implementation for handling PIR queries
- `graph.rs`: Graph data structures and algorithms (A* search)
- `ipc.rs`: Inter-Process Communication module
- `data_entries.rs`: Data structures for graph entries
- `spiral.rs`: Spiral PIR protocol implementation
- `db_settings.rs`: Database and configuration settings
- `bin/`: Binary entry points
  - `geo_client.rs`: Client CLI binary
  - `geo_server.rs`: Server CLI binary
  - `graph_context.rs`: Graph loading and context utilities

### `data/`
Geographic graph data files:
- `*.csv`: Navigation CSV files (node and edge data)
- `*.edgelist`: Graph edge lists
- `*.gctx`: Graph context files (compiled graph format)

### `batch/`
Shell scripts for running benchmarks and tests:
- `run_distance_small.sh`: Run benchmarks on small distance queries
- `run_distance_large.sh`: Run benchmarks on large distance queries
- `run_distance_verylarge.sh`: Run benchmarks on very large distance queries
- `run_benchmarks_vis.sh`: Run visualization of benchmark results
- `run_Astar_result.sh`: Visualize A* search results
- `test.sh`: Run Rust test suite

### `python/`
Python utilities for analysis and visualization:
- `runBenchmakrs.py`: Script to execute benchmarks
- `visualiseBenchmakrs.py`: Visualize benchmark results
- `visualiseAStarResult.py`: Visualize A* pathfinding results

### `output for paper/`
Pre-computed benchmark results and outputs used in research papers. Contains results for various configurations of countries, architectures, and approaches.

### `resources for paper/`
Additional resources and reference materials for the research paper.


### External Dependencies
The project depends on:
- **SinglePass Repository**: Required for SinglePass PIR protocol support
  - **Installation**: Clone the SinglePass repository into the same parent directory as this repo
  - **Branch**: Check out the `archi` branch: `git checkout archi`
  - Expected directory structure:
    ```
    /parent/
    ├── geo-pir/
    └── SinglePass/  (on archi branch)
    ```