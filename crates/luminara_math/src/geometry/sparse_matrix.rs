//! Sparse matrix types for linear algebra operations.
//!
//! Provides CSR matrix format and diagonal matrices.

use sprs::{CsMat, TriMat};
use std::ops::Add;

/// Compressed Sparse Row Matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct CsrMatrix<T> {
    pub inner: CsMat<T>,
}

impl<T> CsrMatrix<T>
where T: Copy + Clone + Default + PartialEq + Add<Output = T>
{
    /// Create a CSR matrix from triplets (row, col, value).
    pub fn from_triplets(rows: usize, cols: usize, triplets: &[(usize, usize, T)]) -> Self {
        let mut trimat = TriMat::new((rows, cols));
        for &(r, c, v) in triplets {
            trimat.add_triplet(r, c, v);
        }
        Self {
            inner: trimat.to_csr(),
        }
    }

    /// Get element at (row, col).
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.inner.get(row, col)
    }

    /// Get non-zero elements of a row.
    pub fn row(&self, row_idx: usize) -> Option<Vec<(usize, T)>> {
        self.inner.outer_view(row_idx).map(|view| {
            view.indices().iter().cloned().zip(view.data().iter().cloned()).collect()
        })
    }
}

/// Diagonal Matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagonalMatrix<T> {
    pub diag: Vec<T>,
}

impl<T: Copy + Clone + Default + PartialEq + Add<Output = T>> DiagonalMatrix<T> {
    /// Create a diagonal matrix from a vector.
    pub fn from_diag(diag: Vec<T>) -> Self {
        Self { diag }
    }

    /// Convert to CSR matrix.
    pub fn to_csr(&self) -> CsrMatrix<T> {
        let n = self.diag.len();
        let mut trimat = TriMat::new((n, n));
        for (i, &v) in self.diag.iter().enumerate() {
            trimat.add_triplet(i, i, v);
        }
        CsrMatrix {
            inner: trimat.to_csr(),
        }
    }

    /// Get element at (i, i).
    pub fn get(&self, i: usize) -> Option<T> {
        if i < self.diag.len() {
            Some(self.diag[i])
        } else {
            None
        }
    }
}
