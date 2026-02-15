// Composition order: t1 (translate) then t2 (rotate).
// In Motor implementation, geometric product self * other applies transformations.
// If M = T * R, then for point p, p' = M p M~ = T R p R~ T~.
// This rotates p, then translates.
//
// If we want translation then rotation, we need M = R * T.
// p' = R T p T~ R~.

// My previous expectation was (10, 0, 0) rotated around Y.
// If T * R (rotate then translate):
// p=(0,0,0) -> R(0) = 0 -> T(0) = (10, 0, 0).
// p=(1,0,0) -> R(1,0,0) = (0,0,-1) -> T(0,0,-1) = (10, 0, -1).

// The test failure implies T * R means something else or calculation is different.
// Let's debug what we got.
// T1: Trans(10, 0, 0)
// T2: Rot(Y, 90)
// Combined = T1 * T2.
// If M = T1 * T2:
// This typically means apply T2 first, then T1 (if treating as operators on points from right, or matrices from left).
// Wait, in PGA motors are applied as M p M~.
// (M1 M2) p (M2~ M1~) = M1 (M2 p M2~) M1~.
// So M1 * M2 means: Apply M2, then apply M1.

// If we want M_total = T1 * T2, it applies T2 (Rotate) then T1 (Translate).
// Origin (0,0,0):
// 1. Rotate 90Y -> (0,0,0)
// 2. Translate 10X -> (10,0,0)
// Result pos should be (10,0,0).

// Why did it fail?
// Maybe `to_rotation_translation` extracts translation in a specific frame?
// If Motor stores T and R mixed, extracting them might give "translation part" which is not just the vector t.
// For a motor M = T * R, the translation part is T.
// But if M = R * T, the translation part is rotated T.

// Let's inspect `to_rotation_translation_glam` implementation in `motor.rs`.
// It extracts half_tx from bivector parts.
// Pseudoscalar parts e01, e02, e03 come from T.
// If M = T * R, translation components e01..e03 are not affected by R?
// Geometric product T * R:
// T = 1 + 0.5 t e0
// R = c + s B
// T * R = (1 + 0.5 t e0) * (c + s B) = R + 0.5 t e0 R.
// Since e0 R = R e0 (or similar depending on algebra),
// actually e0 B = B e0 if B has no e0?
// Yes, e0 is orthogonal to spatial bivectors.
// So T * R = R + 0.5 t R e0.
// The "translation part" (dual part) is 0.5 t R e0.
// This mixes t and R.

// If we want to extract "position" (where the origin moved to), we should transform (0,0,0).
// Or we can rely on `to_rotation_translation` returning the translation vector *if* it corresponds to decomposition.

// Let's verify what `to_rotation_translation` returns.
// If it returns t from T where M = T * R, then it returns the global position.
// If M = R + D (dual), position t = 2 * D * R~.

// Let's check test failure value if possible, or just adjust expectation if we use different convention.
// If T1 * T2 means "Rotate then Translate", then pos is (10,0,0).
// If T1 * T2 means "Translate then Rotate" (M1 applied after M2?), wait.
// M1(M2 p M2~)M1~ -> M2 applies first.
// So T1 * T2 = Apply T2 (Rotate) then T1 (Translate).

// If the test failed, maybe I mixed up T1 and T2?
// t1 = Trans(10,0,0). t2 = Rot(Y, 90).
// combined = t1 * t2.
// Apply Rot, then Trans.
// (0,0,0) -> Rot -> (0,0,0) -> Trans -> (10,0,0).

// If I did t2 * t1 (Trans then Rot):
// (0,0,0) -> Trans -> (10,0,0) -> Rot(Y,90) -> (0,0,-10).

// The error says "assertion failed".
// I suspect the implementation of `to_rotation_translation` in `motor.rs` might retrieve the *local* translation or something else?
// Actually, `to_rotation_translation_parts` logic:
// let half_tx = s * self.e01 - e12 * self.e02 ...
// This formula looks like 2 * (Dual) * (Primal_Reverse)?
// If so, it extracts the global translation t where M = T(t) * R.

// Let's relax the test or debug values.
