//! Heat Method for geodesic distance computation.
//!
//! Computes geodesic distances on triangle meshes using the heat equation.

use super::manifold::{TriangleMesh, CholeskySolver};
use super::sparse_matrix::{CsrMatrix, DiagonalMatrix};
use glam::Vec3;

/// Compute geodesic distance from a source vertex to all other vertices.
///
/// Implements the Heat Method (Crane et al. 2013).
pub fn geodesic_distance_from(mesh: &TriangleMesh, source: usize) -> Option<Vec<f64>> {
    let n = mesh.vertex_count();
    if source >= n { return None; }

    let positions = &mesh.positions;

    // 1. Build matrices
    let l_mat = mesh.build_cotangent_laplacian();
    let m_mat = mesh.build_mass_matrix();

    // 2. Time step t = h^2
    let mut avg_len = 0.0;
    let mut edge_count = 0;
    for tri in &mesh.indices {
        let p0 = positions[tri[0]];
        let p1 = positions[tri[1]];
        let p2 = positions[tri[2]];
        avg_len += p0.distance(p1) as f64;
        avg_len += p1.distance(p2) as f64;
        avg_len += p2.distance(p0) as f64;
        edge_count += 3;
    }
    if edge_count == 0 { return None; }
    avg_len /= edge_count as f64;
    let t = avg_len * avg_len;

    // 3. Solve (M + t L) u = delta
    // A = M + t L
    let m_csr = m_mat.to_csr();

    // Scale L by t
    let l_scaled_inner = l_mat.inner.map(|&x| x * t);

    // Add M + tL
    let a_inner = &m_csr.inner + &l_scaled_inner;
    let a_mat = CsrMatrix { inner: a_inner };

    let solver1 = CholeskySolver::new(&a_mat)?;

    let mut delta = vec![0.0; n];
    delta[source] = 1.0;

    let u = solver1.solve(&delta);

    // 4. Compute normalized gradient field X = - grad u / |grad u|
    let mut div_x = vec![0.0; n];

    for tri in &mesh.indices {
        let i = tri[0];
        let j = tri[1];
        let k = tri[2];

        let p0 = positions[i];
        let p1 = positions[j];
        let p2 = positions[k];

        let n_vec = (p1 - p0).cross(p2 - p0);
        let area2 = n_vec.length(); // 2 * Area
        if area2 < 1e-12 { continue; }
        let normal = n_vec / area2;

        // Edges opposite to vertices
        let e_jk = p2 - p1; // opp i
        let e_ki = p0 - p2; // opp j
        let e_ij = p1 - p0; // opp k

        // grad u = (u_i * (N x e_jk) + u_j * (N x e_ki) + u_k * (N x e_ij)) / (2 A)
        let grad_u = (normal.cross(e_jk) * u[i] as f32 +
                      normal.cross(e_ki) * u[j] as f32 +
                      normal.cross(e_ij) * u[k] as f32) / area2;

        let g_len = grad_u.length();
        let x_vec = if g_len < 1e-12 { Vec3::ZERO } else { -grad_u / g_len };

        // 5. Compute divergence
        // Contribution to vertex i: 0.5 * cot(theta) ...
        // Using integrated divergence formula:
        // b_i = 0.5 * sum_T (N x X_T) . e_opp

        let n_cross_x = normal.cross(x_vec);

        div_x[i] += 0.5 * n_cross_x.dot(e_jk) as f64;
        div_x[j] += 0.5 * n_cross_x.dot(e_ki) as f64;
        div_x[k] += 0.5 * n_cross_x.dot(e_ij) as f64;
    }

    // 6. Solve L phi = div_x
    // Regularize L for Neumann boundary / singular matrix
    let diag_eps = DiagonalMatrix::from_diag(vec![1e-8; n]).to_csr();
    let l_reg_inner = &l_mat.inner + &diag_eps.inner;
    let l_reg = CsrMatrix { inner: l_reg_inner };

    let solver2 = CholeskySolver::new(&l_reg)?;
    let phi = solver2.solve(&div_x);

    // Shift result so source distance is 0
    let shift = phi[source];
    let phi_shifted: Vec<f64> = phi.iter().map(|&v| (v - shift).abs()).collect(); // abs just in case

    Some(phi_shifted)
}
