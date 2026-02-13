use luminara_math::algebra::Motor;
use luminara_math::glam::{Vec3, Quat};

fn main() {
    println!("Luminara Math - Motor Transform Demo");
    println!("====================================\n");

    // 1. Create a motor representing a translation
    let t = Vec3::new(5.0, 0.0, 0.0);
    let m_trans = Motor::from_translation(t);
    println!("Translation Motor (5, 0, 0): {:?}", m_trans);

    // 2. Create a motor representing a rotation (90 degrees around Z)
    let rot = Quat::from_rotation_z(std::f32::consts::PI / 2.0);
    let m_rot = Motor::from_rotation_translation(rot, Vec3::ZERO);
    println!("Rotation Motor (90 deg Z): {:?}", m_rot);

    // 3. Compose: Translate then Rotate
    // M_composed = M_rot * M_trans
    let m_composed = m_rot.geometric_product(&m_trans);
    println!("Composed Motor (T then R): {:?}", m_composed);

    // 4. Transform a point
    let p = Vec3::new(1.0, 0.0, 0.0);
    println!("\nOriginal Point: {:?}", p);

    let p_trans = m_trans.transform_point(p);
    println!("Translated (1+5, 0, 0): {:?}", p_trans);

    let p_rot = m_rot.transform_point(p);
    println!("Rotated (0, 1, 0): {:?}", p_rot);

    let p_final = m_composed.transform_point(p);
    // (1,0,0) -> (6,0,0) -> (0,6,0)
    println!("Transformed (0, 6, 0): {:?}", p_final);

    // 5. Interpolation
    println!("\nInterpolation (t=0.5):");
    let m_interp = m_trans.interpolate(&m_rot, 0.5);
    let p_interp = m_interp.transform_point(p);
    println!("Interpolated Point: {:?}", p_interp);
}
