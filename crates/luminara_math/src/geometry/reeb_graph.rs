//! Reeb graph for topological terrain navigation.
//!
//! Uses discrete Morse theory-inspired gradient tracing to construct the graph.

use glam::Vec3;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CriticalType {
    Minimum,
    Saddle,
    Maximum,
}

#[derive(Clone, Copy, Debug)]
pub struct ReebNode {
    pub position: Vec3,
    pub critical_type: CriticalType,
    pub id: usize,
    pub height: f32,
    pub grid_idx: (usize, usize),
}

#[derive(Clone, Copy, Debug)]
pub struct ReebEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

pub struct ReebGraph {
    pub nodes: Vec<ReebNode>,
    pub edges: Vec<ReebEdge>,
    pub adj: Vec<Vec<usize>>, // Indices into edges
}

#[derive(Clone)]
pub struct HeightMap {
    pub heights: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub scale: Vec3,
}

impl HeightMap {
    pub fn new(heights: Vec<f32>, width: usize, height: usize, scale: Vec3) -> Self {
        assert_eq!(heights.len(), width * height);
        Self {
            heights,
            width,
            height,
            scale,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            return f32::NEG_INFINITY;
        }
        self.heights[y * self.width + x]
    }

    pub fn world_pos(&self, x: usize, y: usize) -> Vec3 {
        Vec3::new(
            x as f32 * self.scale.x,
            self.get(x, y) * self.scale.y,
            y as f32 * self.scale.z,
        )
    }
}

impl ReebGraph {
    pub fn from_heightmap(map: &HeightMap) -> Self {
        let mut nodes = Vec::new();
        let mut node_map = vec![None; map.width * map.height]; // Map grid idx to node idx

        // 1. Identify critical points
        for y in 1..map.height - 1 {
            for x in 1..map.width - 1 {
                if let Some(crit_type) = classify_critical_point(map, x, y) {
                    let id = nodes.len();
                    let node = ReebNode {
                        position: map.world_pos(x, y),
                        critical_type: crit_type,
                        id,
                        height: map.get(x, y),
                        grid_idx: (x, y),
                    };
                    nodes.push(node);
                    node_map[y * map.width + x] = Some(id);
                }
            }
        }

        let mut graph = Self {
            nodes: nodes.clone(), // clone nodes to struct, but we need local access
            edges: Vec::new(),
            adj: vec![Vec::new(); nodes.len()],
        };

        // 2. Connect critical points via gradient paths
        // From each saddle, trace up to Max and down to Min
        for node in &nodes {
            if node.critical_type == CriticalType::Saddle {
                // Trace in all 8 directions
                // If we go UP, we should hit a Max (or Saddle).
                // If we go DOWN, we should hit a Min (or Saddle).

                // We simplify by checking all 8 neighbors. If neighbor is higher, trace ascent.
                // If neighbor is lower, trace descent.
                // Note: Multiple neighbors might lead to same extremum.

                let (sx, sy) = node.grid_idx;
                let mut connected = HashSet::new(); // Avoid duplicate edges

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (sx as isize + dx) as usize;
                        let ny = (sy as isize + dy) as usize;

                        let h_curr = map.get(sx, sy);
                        let h_next = map.get(nx, ny);

                        if h_next > h_curr {
                            // Ascent
                            if let Some(target) = trace_gradient(map, &node_map, nx, ny, true) {
                                connected.insert(target);
                            }
                        } else if h_next < h_curr {
                            // Descent
                            if let Some(target) = trace_gradient(map, &node_map, nx, ny, false) {
                                connected.insert(target);
                            }
                        }
                    }
                }

                for target in connected {
                    graph.add_edge(node.id, target);
                }
            }
        }

        graph
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        let dist = self.nodes[u].position.distance(self.nodes[v].position);
        let edge_idx = self.edges.len();
        self.edges.push(ReebEdge {
            from: u,
            to: v,
            weight: dist,
        });
        self.adj[u].push(edge_idx);

        let edge_idx_rev = self.edges.len();
        self.edges.push(ReebEdge {
            from: v,
            to: u,
            weight: dist,
        });
        self.adj[v].push(edge_idx_rev);
    }

    pub fn find_path(&self, start_pos: Vec3, end_pos: Vec3) -> Option<Vec<Vec3>> {
        // Find nearest nodes
        let start_node = self.nearest_node(start_pos)?;
        let end_node = self.nearest_node(end_pos)?;

        // A* search
        let mut open_set = BinaryHeap::new();
        open_set.push(State {
            cost: 0.0,
            position: 0.0,
            index: start_node,
        }); // position is heuristic? No.
            // State for BinaryHeap needs Ord.
            // Custom struct.

        let mut came_from = vec![usize::MAX; self.nodes.len()];
        let mut g_score = vec![f32::INFINITY; self.nodes.len()];
        g_score[start_node] = 0.0;

        open_set.push(State {
            cost: 0.0,     // f_score
            position: 0.0, // heuristic part? Just for ordering.
            index: start_node,
        });

        while let Some(State {
            cost: current_f,
            index: current,
            ..
        }) = open_set.pop()
        {
            if current == end_node {
                // Reconstruct path
                let mut path = Vec::new();
                let mut curr = current;
                while curr != start_node {
                    path.push(self.nodes[curr].position);
                    curr = came_from[curr];
                }
                path.push(self.nodes[start_node].position);
                path.reverse();
                return Some(path);
            }

            // If we found a better path already
            // BinaryHeap is max-heap. We invert cost.
            if -current_f < g_score[current] {
                continue;
            } // Actually I used negative logic in State impl?

            for &edge_idx in &self.adj[current] {
                let edge = &self.edges[edge_idx];
                let neighbor = edge.to;

                let tentative_g = g_score[current] + edge.weight;
                if tentative_g < g_score[neighbor] {
                    came_from[neighbor] = current;
                    g_score[neighbor] = tentative_g;
                    let h = self.nodes[neighbor]
                        .position
                        .distance(self.nodes[end_node].position);
                    open_set.push(State {
                        cost: -(tentative_g + h), // Max-heap, so negative
                        position: 0.0,
                        index: neighbor,
                    });
                }
            }
        }

        None
    }

    pub fn nearest_node(&self, pos: Vec3) -> Option<usize> {
        let mut best_dist = f32::MAX;
        let mut best_idx = None;
        for (i, node) in self.nodes.iter().enumerate() {
            let d = node.position.distance_squared(pos);
            if d < best_dist {
                best_dist = d;
                best_idx = Some(i);
            }
        }
        best_idx
    }
}

// Helpers

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f32,
    position: f32, // Unused
    index: usize,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Max-heap
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

fn classify_critical_point(map: &HeightMap, x: usize, y: usize) -> Option<CriticalType> {
    // Look at 8 neighbors. Count sign changes.
    let h = map.get(x, y);
    let neighbors = [
        (x + 1, y),
        (x + 1, y + 1),
        (x, y + 1),
        (x - 1, y + 1),
        (x - 1, y),
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
    ];

    let mut signs = Vec::with_capacity(8);
    for &(nx, ny) in &neighbors {
        let nh = map.get(nx, ny);
        if nh > h {
            signs.push(1);
        } else if nh < h {
            signs.push(-1);
        } else {
            signs.push(0);
        } // Flat? Treat as same component?
    }

    // Remove zeros (treat as connected to prev)
    // Or strictly greater/less?
    // If equal, it's a plateau. Simple method: treat as same (no change).
    // Better: treat as noise or use Simulation of Simplicity.
    // We filter out 0s.
    let signs_filtered: Vec<i32> = signs.into_iter().filter(|&s| s != 0).collect();
    if signs_filtered.is_empty() {
        return None;
    } // Flat

    // Count changes
    let mut changes = 0;
    for i in 0..signs_filtered.len() {
        let next = (i + 1) % signs_filtered.len();
        if signs_filtered[i] != signs_filtered[next] {
            changes += 1;
        }
    }

    if changes == 0 {
        if signs_filtered[0] == 1 {
            return Some(CriticalType::Minimum);
        }
        if signs_filtered[0] == -1 {
            return Some(CriticalType::Maximum);
        }
    } else if changes >= 4 {
        return Some(CriticalType::Saddle);
    }

    None // Regular point (2 changes)
}

fn trace_gradient(
    map: &HeightMap,
    node_map: &[Option<usize>],
    mut cx: usize,
    mut cy: usize,
    ascent: bool,
) -> Option<usize> {
    let w = map.width;
    let h = map.height;
    let mut visited = HashSet::new();

    loop {
        if let Some(id) = node_map[cy * w + cx] {
            // Reached a critical point
            return Some(id);
        }

        if !visited.insert((cx, cy)) {
            return None; // Loop detected
        }

        // Find steepest neighbor
        let mut best_h = map.get(cx, cy);
        let mut best_n = None;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = (cx as isize + dx) as usize;
                let ny = (cy as isize + dy) as usize;
                if nx >= w || ny >= h {
                    continue;
                }

                let nh = map.get(nx, ny);
                if ascent {
                    if nh > best_h {
                        best_h = nh;
                        best_n = Some((nx, ny));
                    }
                } else {
                    if nh < best_h {
                        best_h = nh;
                        best_n = Some((nx, ny));
                    }
                }
            }
        }

        match best_n {
            Some((nx, ny)) => {
                cx = nx;
                cy = ny;
            }
            None => return None, // Local extremum not marked? Should not happen if we marked all critical points.
        }
    }
}
