use luminara_asset::{AssetId, Handle};
use luminara_math::Mat4;
use luminara_render::{DrawCommand, Mesh, PbrMaterial};

#[test]
fn test_draw_command_creation() {
    let mesh_handle = Handle::<Mesh>::new(AssetId::new(), 0);
    let mat_handle = Handle::<PbrMaterial>::new(AssetId::new(), 0);
    let transform = Mat4::IDENTITY;

    let cmd = Mesh::draw(mesh_handle.clone(), mat_handle.clone(), transform);

    match cmd {
        DrawCommand::DrawMesh {
            mesh,
            material,
            transform: t,
        } => {
            assert_eq!(mesh, mesh_handle);
            assert_eq!(material, mat_handle);
            assert_eq!(t, transform);
        }
        _ => panic!("Wrong command type"),
    }
}
