//! Manifold surface and discrete differential geometry.
//!
//! Provides cotangent Laplacian, mass matrix, and discrete exterior calculus.

use glam::Vec3;
use super::sparse_matrix::{CsrMatrix, DiagonalMatrix};
use sprs::TriMat;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexId(pub usize);

/// A triangle mesh suitable for differential geometry.
pub struct TriangleMesh {
    pub positions: Vec<Vec3>,
    pub indices: Vec<[usize; 3]>,
}

impl TriangleMesh {
    pub fn new(positions: Vec<Vec3>, indices: Vec<[usize; 3]>) -> Self {
        Self { positions, indices }
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Build the cotangent Laplacian matrix.
    /// L_ij = -0.5 * (cot alpha + cot beta) if j in neighbors(i)
    /// L_ii = sum_{j != i} -L_ij
    pub fn build_cotangent_laplacian(&self) -> CsrMatrix<f64> {
        let n = self.vertex_count();
        let mut trimat = TriMat::new((n, n));
        let mut diag = vec![0.0; n];

        for tri in &self.indices {
            let i = tri[0];
            let j = tri[1];
            let k = tri[2];

            let p0 = self.positions[i];
            let p1 = self.positions[j];
            let p2 = self.positions[k];

            // Cotangents of angles at p0, p1, p2
            // Angle at p0: between (p1-p0) and (p2-p0)
            let u0 = p1 - p0;
            let v0 = p2 - p0;
            let cot0 = cotan(u0, v0);

            // Angle at p1: between (p2-p1) and (p0-p1)
            let u1 = p2 - p1;
            let v1 = p0 - p1;
            let cot1 = cotan(u1, v1);

            // Angle at p2: between (p0-p2) and (p1-p2)
            let u2 = p0 - p2;
            let v2 = p1 - p2;
            let cot2 = cotan(u2, v2);

            // Edge (j, k) is opposite i (angle 0). Weight: 0.5 * cot0
            add_edge(&mut trimat, &mut diag, j, k, 0.5 * cot0);

            // Edge (k, i) is opposite j (angle 1). Weight: 0.5 * cot1
            add_edge(&mut trimat, &mut diag, k, i, 0.5 * cot1);

            // Edge (i, j) is opposite k (angle 2). Weight: 0.5 * cot2
            add_edge(&mut trimat, &mut diag, i, j, 0.5 * cot2);
        }

        // Add diagonal entries
        for i in 0..n {
            trimat.add_triplet(i, i, diag[i]);
        }

        CsrMatrix { inner: trimat.to_csr() }
    }

    /// Build the lumped mass matrix (diagonal).
    /// M_ii = 1/3 * sum of areas of incident triangles.
    pub fn build_mass_matrix(&self) -> DiagonalMatrix<f64> {
        let n = self.vertex_count();
        let mut diag = vec![0.0; n];

        for tri in &self.indices {
            let p0 = self.positions[tri[0]];
            let p1 = self.positions[tri[1]];
            let p2 = self.positions[tri[2]];

            // Area = 0.5 * |u x v|
            let area = 0.5 * (p1 - p0).cross(p2 - p0).length() as f64;
            let third_area = area / 3.0;

            diag[tri[0]] += third_area;
            diag[tri[1]] += third_area;
            diag[tri[2]] += third_area;
        }

        DiagonalMatrix { diag }
    }

    /// Build the discrete exterior derivative operator d0: 0-forms -> 1-forms.
    /// Returns a matrix of size |E| x |V|.
    pub fn build_exterior_derivative_0(&self) -> CsrMatrix<f64> {
        let n_v = self.vertex_count();
        let mut edges = BTreeSet::new();
        for tri in &self.indices {
            for k in 0..3 {
                let u = tri[k];
                let v = tri[(k+1)%3];
                if u < v { edges.insert((u, v)); } else { edges.insert((v, u)); }
            }
        }

        let n_e = edges.len();
        let mut trimat = TriMat::new((n_e, n_v));

        for (idx, &(u, v)) in edges.iter().enumerate() {
            trimat.add_triplet(idx, u, -1.0);
            trimat.add_triplet(idx, v, 1.0);
        }

        CsrMatrix { inner: trimat.to_csr() }
    }

    /// Build the Hodge star operator *0: 0-forms -> 2-forms (dual 0-forms).
    /// This is equivalent to the mass matrix (diagonal of dual areas).
    pub fn build_hodge_star_0(&self) -> DiagonalMatrix<f64> {
        self.build_mass_matrix()
    }
}

fn cotan(u: Vec3, v: Vec3) -> f64 {
    let dot = u.dot(v) as f64;
    let cross = u.cross(v).length() as f64;
    if cross.abs() < 1e-12 {
        0.0 // Degenerate triangle? Return 0 weight?
    } else {
        dot / cross
    }
}

fn add_edge(trimat: &mut TriMat<f64>, diag: &mut [f64], i: usize, j: usize, w: f64) {
    // Only one half-edge? No, Laplacian is symmetric.
    // We add -w to (i,j) and (j,i).
    // But since we iterate all triangles, each edge is visited twice (once for each adjacent face).
    // Wait. "0.5 * (cot alpha + cot beta)".
    // If we process triangle T1, we add 0.5 * cot alpha to edge (i,j).
    // When we process T2 (neighbor), we add 0.5 * cot beta to edge (i,j).
    // Total weight becomes 0.5(cot alpha + cot beta).
    // Correct.

    // We assume TriMat accumulates duplicates (Add impl).
    trimat.add_triplet(i, j, -w);
    trimat.add_triplet(j, i, -w);
    diag[i] += w;
    diag[j] += w;
}

/// Solver for symmetric positive definite systems.
///
/// Uses Conjugate Gradient method internally as sparse Cholesky factorization
/// is not available in the current dependency set.
pub struct CholeskySolver {
    mat: CsrMatrix<f64>,
}

impl CholeskySolver {
    pub fn new(mat: &CsrMatrix<f64>) -> Option<Self> {
        // CG works for any SPD matrix.
        Some(Self { mat: mat.clone() })
    }

    pub fn solve(&self, b: &[f64]) -> Vec<f64> {
        // Conjugate Gradient implementation
        let n = b.len();
        let mut x = vec![0.0; n];
        let mut r = b.to_vec(); // r = b - A*x (x=0)
        let mut p = r.clone();
        let mut rsold = dot(&r, &r);

        if rsold < 1e-20 { return x; }

        for _ in 0..n { // Max iterations = dim
            // Matrix-vector multiplication A*p
            let mut ap = vec![0.0; n];
            for (row_idx, row) in self.mat.inner.outer_iterator().enumerate() {
                let mut sum = 0.0;
                for (col_idx, &val) in row.indices().iter().zip(row.data()) {
                    sum += val * p[*col_idx];
                }
                ap[row_idx] = sum;
            }

            let alpha = rsold / dot(&p, &ap);

            for i in 0..n {
                x[i] += alpha * p[i];
                r[i] -= alpha * ap[i];
            }

            let rsnew = dot(&r, &r);
            if rsnew < 1e-20 { break; }

            let beta = rsnew / rsold;
            for i in 0..n {
                p[i] = r[i] + beta * p[i];
            }
            rsold = rsnew;
        }
        x
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
