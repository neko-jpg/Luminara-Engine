use glam::Vec3;
use luminara_math::geometry::{CriticalType, HeightMap, ReebGraph, ReebNode};

#[test]
fn test_single_peak() {
    // 5x5 map. Center (2,2) is peak.
    let width = 5;
    let height = 5;
    let mut heights = vec![0.0; 25];

    // Cone shape
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32) - 2.0;
            let dy = (y as f32) - 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            heights[y * width + x] = 5.0 - dist;
        }
    }

    let map = HeightMap::new(heights, width, height, Vec3::ONE);
    let graph = ReebGraph::from_heightmap(&map);

    // Center (2,2) should be Maximum.
    // Interior points around it are slopes.
    // No interior saddle or minimum.
    // So 1 node.

    let max_node = graph
        .nodes
        .iter()
        .find(|n| n.critical_type == CriticalType::Maximum);
    assert!(max_node.is_some());
    assert_eq!(max_node.unwrap().grid_idx, (2, 2));
}

#[test]
fn test_two_peaks_saddle() {
    // 7x5 map.
    // Peaks at (1, 2) and (5, 2). Saddle at (3, 2).
    let width = 7;
    let height = 5;
    let mut heights = vec![0.0; 35];

    for y in 0..height {
        for x in 0..width {
            // Two gaussians
            let d1 = (x as f32 - 1.0).powi(2) + (y as f32 - 2.0).powi(2);
            let d2 = (x as f32 - 5.0).powi(2) + (y as f32 - 2.0).powi(2);
            heights[y * width + x] = 3.0 * (-d1 / 2.0).exp() + 3.0 * (-d2 / 2.0).exp();
        }
    }

    // Make (3,2) a saddle explicitly if gaussian didn't create one nice enough
    // At (3,2), h approx 3*exp(-2) + 3*exp(-2) = 6*0.13 = 0.8
    // At (3,1) and (3,3) (sides), distance is larger?
    // d = 2^2 + 1^2 = 5. exp(-2.5). Smaller.
    // At (2,2) and (4,2), distance is smaller (1^2=1). exp(-0.5). Larger.
    // So (3,2) is Local Min in X, Local Max in Y?
    // Wait.
    // Along Y (x=3): (3,2) is h(3,2). Neighbors (3,1), (3,3) are lower. So Local Max along Y.
    // Along X (y=2): Neighbors (2,2), (4,2) are higher. So Local Min along X.
    // This is a SADDLE.

    let map = HeightMap::new(heights, width, height, Vec3::ONE);
    let graph = ReebGraph::from_heightmap(&map);

    // Should have 2 Maxima, 1 Saddle.
    let maxima = graph
        .nodes
        .iter()
        .filter(|n| n.critical_type == CriticalType::Maximum)
        .count();
    let saddles = graph
        .nodes
        .iter()
        .filter(|n| n.critical_type == CriticalType::Saddle)
        .count();

    assert!(maxima >= 2, "Expected at least 2 maxima, got {}", maxima);
    assert!(saddles >= 1, "Expected at least 1 saddle, got {}", saddles);

    // Check connectivity
    // Saddle should connect to both maxima (ascent)
    // And possibly minima (descent)

    // Find saddle
    let saddle_node = graph
        .nodes
        .iter()
        .find(|n| n.critical_type == CriticalType::Saddle)
        .unwrap();

    // Check neighbors of saddle
    let neighbors = &graph.adj[saddle_node.id];
    let connected_maxima = neighbors
        .iter()
        .map(|&e_idx| graph.edges[e_idx].to)
        .filter(|&id| graph.nodes[id].critical_type == CriticalType::Maximum)
        .count();

    assert!(connected_maxima >= 2, "Saddle should connect to 2 maxima");
}

#[test]
fn test_pathfinding() {
    // Construct a graph manually or use the two_peaks one
    // Let's use two_peaks logic but manual graph construction to ensure structure
    // Peak1 -- Saddle -- Peak2

    let mut nodes = Vec::new();
    nodes.push(ReebNode {
        position: Vec3::new(0.0, 0.0, 0.0),
        critical_type: CriticalType::Maximum,
        id: 0,
        height: 10.0,
        grid_idx: (0, 0),
    }); // Peak1
    nodes.push(ReebNode {
        position: Vec3::new(10.0, 0.0, 0.0),
        critical_type: CriticalType::Saddle,
        id: 1,
        height: 5.0,
        grid_idx: (10, 0),
    }); // Saddle
    nodes.push(ReebNode {
        position: Vec3::new(20.0, 0.0, 0.0),
        critical_type: CriticalType::Maximum,
        id: 2,
        height: 10.0,
        grid_idx: (20, 0),
    }); // Peak2

    let mut edges = Vec::new();
    let mut adj = vec![vec![], vec![], vec![]];

    // Edge 0-1
    edges.push(luminara_math::geometry::ReebEdge {
        from: 0,
        to: 1,
        weight: 10.0,
    });
    adj[0].push(0);
    edges.push(luminara_math::geometry::ReebEdge {
        from: 1,
        to: 0,
        weight: 10.0,
    });
    adj[1].push(1);

    // Edge 1-2
    edges.push(luminara_math::geometry::ReebEdge {
        from: 1,
        to: 2,
        weight: 10.0,
    });
    adj[1].push(2);
    edges.push(luminara_math::geometry::ReebEdge {
        from: 2,
        to: 1,
        weight: 10.0,
    });
    adj[2].push(3);

    let graph = ReebGraph { nodes, edges, adj };

    // Path from Peak1 to Peak2
    let path = graph.find_path(Vec3::new(0.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 0.0));
    assert!(path.is_some());
    let p = path.unwrap();
    assert_eq!(p.len(), 3);
    assert_eq!(p[0], Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(p[1], Vec3::new(10.0, 0.0, 0.0));
    assert_eq!(p[2], Vec3::new(20.0, 0.0, 0.0));
}
