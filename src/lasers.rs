//! Things for visualising lasers

use atomecs::{laser::gaussian::GaussianBeam, laser_cooling::transition::AtomicTransition, bevy_bridge::Scale};
use bevy::{prelude::*, pbr::NotShadowCaster};

use crate::atoms::{MaterialColorConfig, get_color, scale_color};

/// adds meshes to atoms so they can be rendered.
pub fn add_meshes_to_lasers<T: AtomicTransition>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    color_factor: Res<MaterialColorConfig>,
    scale: Res<Scale>,
    query: Query<(Entity, &GaussianBeam), Without<Mesh3d>>
) {
    let color = scale_color(get_color::<T>(), color_factor.factor).with_alpha(0.05);
    for (entity, beam) in query.iter() {
        let pos = beam.intersection * scale.0;
        let mat = StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let dir = Vec3::new(
            (beam.direction[0] ) as f32,
            (beam.direction[1] ) as f32,
            (beam.direction[2] ) as f32
        );
        let up = if Vec3::Y.dot(dir).abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let rotation = Transform::default().looking_at(dir, up).rotation * Quat::from_rotation_x(std::f32::consts::PI / 2.0);
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new((beam.e_radius * scale.0) as f32, 40.0))),
            MeshMaterial3d(materials.add(mat)),
            Transform::from_xyz(pos[0] as f32, pos[1] as f32, pos[2] as f32).with_rotation(rotation),
        ))
        .insert(NotShadowCaster);
    }
}
