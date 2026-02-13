use luminara_math::geometry::{CsrMatrix, DiagonalMatrix};

#[test]
fn test_csr_construction() {
    let triplets = vec![(0, 0, 1.0), (0, 1, 2.0), (1, 1, 3.0)];
    let mat = CsrMatrix::from_triplets(2, 2, &triplets);

    assert_eq!(mat.get(0, 0), Some(&1.0));
    assert_eq!(mat.get(0, 1), Some(&2.0));
    // get returns Some for zero elements if stored? No, get returns reference.
    // If not stored, sprs::CsMat::get returns None.
    assert_eq!(mat.get(1, 0), None);
    assert_eq!(mat.get(1, 1), Some(&3.0));

    let row0 = mat.row(0).unwrap();
    // (0, 1.0), (1, 2.0)
    assert_eq!(row0.len(), 2);
    assert_eq!(row0[0], (0, 1.0));
    assert_eq!(row0[1], (1, 2.0));
}

#[test]
fn test_diagonal() {
    let diag = vec![1.0, 2.0, 3.0];
    let d = DiagonalMatrix::from_diag(diag);

    let csr = d.to_csr();
    assert_eq!(csr.get(0, 0), Some(&1.0));
    assert_eq!(csr.get(1, 1), Some(&2.0));
    assert_eq!(csr.get(2, 2), Some(&3.0));
    assert_eq!(csr.get(0, 1), None);
    assert_eq!(csr.get(1, 0), None);
}
