use cgmath::{Vector3, InnerSpace};

use crate::{mesh::MeshBuilder, color_normal_vertex::ColorNormalVertex};

// pub fn test_scene(renderer: &Renderer) -> Scene {
//     let mut camera = Camera::new(
//         Vector3::new(1.0, -2.0, 2.0),
//         Vector3::new(0.5, 0.5, 0.5),
//         Vector3::unit_z(),
//         1.25,
//         45.0,
//         0.1,
//         100.0,
//         renderer.device(),
//     );

//     let light = DirectionalLight::new(
//         Vector3::new(1.0, 1.0, -1.0),
//         0.5,
//         Vector3::new(1.0, 1.0, 1.0),
//     );
//     // let model = Model::new(
//     //     Transform::new(
//     //         Vector3::new(-0.5, -0.5, -0.5),
//     //         Quaternion::from_angle_x(cgmath::Rad(0.0)),
//     //     ),
//     //     cube_mesh(),
//     //     renderer.device(),
//     // );
    
//     Scene::new(
//         camera,
//         light,
//         // model,
//     )
// }

// todo: put these in echoes_procgen crate

pub fn gradient_quad_mesh() -> MeshBuilder<ColorNormalVertex> {
    let mut mesh = MeshBuilder::new();

    let lower_left = ColorNormalVertex::new(
        Vector3::new(-1.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
    );
    let lower_right = ColorNormalVertex::new(
        Vector3::new(1.0, 0.0, -1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(0.0, -1.0, 0.0),
    );
    let upper_left = ColorNormalVertex::new(
        Vector3::new(-1.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
    );
    let upper_right = ColorNormalVertex::new(
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(0.0, -1.0, 0.0),
    );

    mesh.add_quad_facing(lower_left, lower_right, upper_right, upper_left, Vector3::new(0.0, -1.0, 0.0));

    mesh
}

pub fn cube_mesh() -> MeshBuilder<ColorNormalVertex> {
    let mut mesh = MeshBuilder::new();

    let zzz = Vector3::<f32>::new(0.0, 0.0, 0.0);
    let zzo = Vector3::<f32>::new(0.0, 0.0, 1.0);
    let zoz = Vector3::<f32>::new(0.0, 1.0, 0.0);
    let zoo = Vector3::<f32>::new(0.0, 1.0, 1.0);
    let ozz = Vector3::<f32>::new(1.0, 0.0, 0.0);
    let ozo = Vector3::<f32>::new(1.0, 0.0, 1.0);
    let ooz = Vector3::<f32>::new(1.0, 1.0, 0.0);
    let ooo = Vector3::<f32>::new(1.0, 1.0, 1.0);

    // left
    mesh.add_quad_facing(
        ColorNormalVertex::new(zzz, zzz, -Vector3::<f32>::unit_x()),
        ColorNormalVertex::new(zzo, zzo, -Vector3::<f32>::unit_x()),
        ColorNormalVertex::new(zoo, zoo, -Vector3::<f32>::unit_x()),
        ColorNormalVertex::new(zoz, zoz, -Vector3::<f32>::unit_x()),
        -Vector3::<f32>::unit_x(),
    );

    // right
    mesh.add_quad_facing(
        ColorNormalVertex::new(ozz, ozz, Vector3::<f32>::unit_x()),
        ColorNormalVertex::new(ozo, ozo, Vector3::<f32>::unit_x()),
        ColorNormalVertex::new(ooo, ooo, Vector3::<f32>::unit_x()),
        ColorNormalVertex::new(ooz, ooz, Vector3::<f32>::unit_x()),
        Vector3::<f32>::unit_x(),
    );

    // front
    mesh.add_quad_facing(
        ColorNormalVertex::new(zzz, zzz, -Vector3::<f32>::unit_y()),
        ColorNormalVertex::new(zzo, zzo, -Vector3::<f32>::unit_y()),
        ColorNormalVertex::new(ozo, ozo, -Vector3::<f32>::unit_y()),
        ColorNormalVertex::new(ozz, ozz, -Vector3::<f32>::unit_y()),
        -Vector3::<f32>::unit_y(),
    );

    // back
    mesh.add_quad_facing(
        ColorNormalVertex::new(zoz, zoz, Vector3::<f32>::unit_y()),
        ColorNormalVertex::new(zoo, zoo, Vector3::<f32>::unit_y()),
        ColorNormalVertex::new(ooo, ooo, Vector3::<f32>::unit_y()),
        ColorNormalVertex::new(ooz, ooz, Vector3::<f32>::unit_y()),
        Vector3::<f32>::unit_y(),
    );

    // bottom
    mesh.add_quad_facing(
        ColorNormalVertex::new(zzz, zzz, -Vector3::<f32>::unit_z()),
        ColorNormalVertex::new(zoz, zoz, -Vector3::<f32>::unit_z()),
        ColorNormalVertex::new(ooz, ooz, -Vector3::<f32>::unit_z()),
        ColorNormalVertex::new(ozz, ozz, -Vector3::<f32>::unit_z()),
        -Vector3::<f32>::unit_z()
    );

    // top
    mesh.add_quad_facing(
        ColorNormalVertex::new(zzo, zzo, Vector3::<f32>::unit_z()),
        ColorNormalVertex::new(zoo, zoo, Vector3::<f32>::unit_z()),
        ColorNormalVertex::new(ooo, ooo, Vector3::<f32>::unit_z()),
        ColorNormalVertex::new(ozo, ozo, Vector3::<f32>::unit_z()),
        Vector3::<f32>::unit_z()
    );

    mesh
}

pub fn sphere_vertex(position: Vector3<f32>, radius: f32, color: Vector3<f32>) -> ColorNormalVertex {
    let normalized = radius * position.normalize();

    ColorNormalVertex::new(
        normalized,
        color,
        normalized,
    )
}

pub fn add_sphere_face(mesh: &mut MeshBuilder<ColorNormalVertex>, normal: Vector3<f32>, dx: Vector3<f32>, dy: Vector3<f32>, radius: f32, divisions: u32, color: Vector3<f32>) {
    let corner = (normal - dx - dy) / 2.0;

    for i in 0..divisions - 1 {
        for j in 0..divisions - 1 {
            let lower_left = corner + (i as f32 / (divisions - 1) as f32) * dx + (j as f32 / (divisions - 1) as f32) * dy;
            let lower_right = corner + ((i + 1) as f32 / (divisions - 1) as f32) * dx + (j as f32 / (divisions - 1) as f32) * dy;
            let upper_left = corner + (i as f32 / (divisions - 1) as f32) * dx + ((j + 1) as f32 / (divisions - 1) as f32) * dy;
            let upper_right = corner + ((i + 1) as f32 / (divisions - 1) as f32) * dx + ((j + 1) as f32 / (divisions - 1) as f32) * dy;

            let lower_left = sphere_vertex(lower_left, radius, color);
            let lower_right = sphere_vertex(lower_right, radius, color);
            let upper_left = sphere_vertex(upper_left, radius, color);
            let upper_right = sphere_vertex(upper_right, radius, color);

            mesh.add_quad_facing(lower_left, lower_right, upper_right, upper_left, normal);
        }
    }
}

pub fn simple_sphere_mesh(radius: f32, divisions: u32, color: Vector3<f32>) -> MeshBuilder<ColorNormalVertex> {
    let mut mesh = MeshBuilder::<ColorNormalVertex>::new();

    add_sphere_face(
        &mut mesh,
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        radius,
        divisions,
        color,
    );
    add_sphere_face(
        &mut mesh,
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        radius,
        divisions,
        color,
    );

    add_sphere_face(
        &mut mesh,
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        radius,
        divisions,
        color,
    );
    add_sphere_face(
        &mut mesh,
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        radius,
        divisions,
        color,
    );

    add_sphere_face(
        &mut mesh,
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        radius,
        divisions,
        color,
    );
    add_sphere_face(
        &mut mesh,
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        radius,
        divisions,
        color,
    );    

    mesh
}

// pub fn coniferous_mesh(
//     height: f32,
//     base_radius: f32,
//     branch_start: f32,
//     layers: u32,
// ) -> Mesh<ColorNormalVertex> {
//     let mesh = Mesh::new();

    

//     mesh
// }

// pub fn add_cone(
//     mesh: &mut Mesh<ColorNormalVertex>,
//     base: Vector3<f32>,
//     tip: Vector3<f32>,
//     base_radius: f32,
//     tip_radius: f32,
//     segments: i32,
//  ) {
//     for i in 0..segments {

//     }
// }

// pub fn get_orthogonal(
//     vector: Vector3<f32>,

// )