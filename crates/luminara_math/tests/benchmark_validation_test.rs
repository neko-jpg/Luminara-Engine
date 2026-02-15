// bentch.mdで指摘された検証テスト

use glam::Vec3;
use luminara_math::geometry::bvh::{Aabb, Bvh, Primitive};

// 2. BVHのアロケーション回数計測用のダミープリミティブ
#[derive(Clone)]
struct TestPrimitive {
    aabb: Aabb,
}

impl Primitive for TestPrimitive {
    fn aabb(&self) -> Aabb {
        self.aabb
    }
    fn intersect(&self, _origin: Vec3, _dir: Vec3) -> Option<f32> {
        None
    }
}

#[test]
fn test_bvh_allocation_count() {
    // 10万プリミティブを生成
    let count = 100_000;
    let primitives: Vec<TestPrimitive> = (0..count)
        .map(|i| {
            let x = (i % 100) as f32;
            let y = ((i / 100) % 100) as f32;
            let z = (i / 10000) as f32;
            let min = Vec3::new(x, y, z);
            let max = min + Vec3::new(0.5, 0.5, 0.5);
            TestPrimitive {
                aabb: Aabb::new(min, max),
            }
        })
        .collect();

    // BVH構築
    let bvh = Bvh::build(primitives);

    // 構築が成功したことを確認（rootはフィールド）
    // BvhNodeは常に存在するので、プリミティブ数を確認
    assert_eq!(
        bvh.primitives.len(),
        count,
        "BVH should contain all primitives"
    );

    println!("BVH構築完了: {} プリミティブ", count);
    println!("注意: アロケーション回数の詳細計測にはdhatやallocation-counterが必要");
}

// 3. シンプレクティック積分器のエネルギー保存テスト
#[cfg(test)]
mod symplectic_tests {
    use luminara_math::algebra::{Bivector, LieGroupIntegrator, Motor, SymplecticEuler};

    #[test]
    fn test_energy_conservation() {
        // 回転運動のシミュレーション
        let mut motor = Motor::IDENTITY;
        let mut velocity = Bivector::new(0.1, 0.0, 0.0, 0.0, 0.0, 0.0);
        let dt = 0.016; // 60FPS相当
        let steps = 1000;

        // 初期エネルギー（角運動量の大きさの二乗）
        let initial_energy = velocity.norm_squared();

        // シンプレクティック積分（位置と速度の両方を更新）
        for _ in 0..steps {
            let (new_motor, new_velocity) =
                SymplecticEuler::step(motor, velocity, dt, |_| Bivector::ZERO);
            motor = new_motor;
            velocity = new_velocity;
        }

        // 最終エネルギー
        let final_energy = velocity.norm_squared();

        // エネルギー変動
        let drift = (final_energy - initial_energy).abs();
        let relative_drift = drift / initial_energy;

        println!("初期エネルギー: {}", initial_energy);
        println!("最終エネルギー: {}", final_energy);
        println!("エネルギー変動: {} ({:.6}%)", drift, relative_drift * 100.0);

        // 許容誤差: 1000ステップで1%以内
        assert!(
            relative_drift < 0.01,
            "エネルギー変動が大きすぎます: {:.6}%",
            relative_drift * 100.0
        );
    }

    #[test]
    fn test_symplectic_vs_rk4() {
        let motor = Motor::IDENTITY;
        let velocity = Bivector::new(0.1, 0.0, 0.0, 0.0, 0.0, 0.0);
        let dt = 0.016;
        let steps = 100;

        // シンプレクティック積分
        let mut m_symplectic = motor;
        let mut v_symplectic = velocity;
        for _ in 0..steps {
            let (new_m, new_v) =
                SymplecticEuler::step(m_symplectic, v_symplectic, dt, |_| Bivector::ZERO);
            m_symplectic = new_m;
            v_symplectic = new_v;
        }

        // MK4積分（Lie群版RK4）
        let mut m_mk4 = motor;
        for _ in 0..steps {
            m_mk4 = LieGroupIntegrator::step(m_mk4, dt, |_| velocity);
        }

        println!("Symplectic結果: {:?}", m_symplectic);
        println!("MK4結果: {:?}", m_mk4);

        // 両方とも有効な結果を返すことを確認
        assert!(m_symplectic.s.is_finite());
        assert!(m_mk4.s.is_finite());
    }
}
