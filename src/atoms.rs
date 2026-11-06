//! Functionality for rendering atoms.

use bevy::prelude::*;
use atomecs::{atom::{Atom, Position}, laser_cooling::{transition::AtomicTransition, photons_scattered::TotalPhotonsScattered}, integrator::Timestep, bevy_bridge::Scale};
use nalgebra::clamp;

/// Scale a color by a factor, since `Color * f32` is not implemented in bevy 0.16.
pub fn scale_color(color: Color, factor: f32) -> Color {
    let mut srgba = color.to_srgba();
    srgba.red *= factor;
    srgba.green *= factor;
    srgba.blue *= factor;
    Color::Srgba(srgba)
}

/// adds meshes to atoms so they can be rendered.
pub fn add_meshes_to_atoms<T: AtomicTransition>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    color_factor: Res<MaterialColorConfig>,
    scale: Res<Scale>,
    query: Query<(Entity, &Position), (With<Atom>, Without<Mesh3d>)>
) {
    let color = scale_color(get_color::<T>(), color_factor.factor);
    for (entity, pos) in query.iter() {
        let p = pos.pos * scale.0;
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_xyz(p[0] as f32, p[1] as f32, p[2] as f32),
        ));
    }
}

#[derive(Resource)]
pub struct MaterialColorConfig {
    pub factor: f32
}
impl Default for MaterialColorConfig {
    fn default() -> Self {
        Self { factor: 1.0 }
    }
}

pub fn get_color<T : AtomicTransition>() -> Color {
    let wavelength_nm = T::wavelength() * 1e9;
    let color =
        if wavelength_nm < 480.0 {
            Color::srgb(0.0, 0.0, 1.0)
        } else if wavelength_nm < 550.0 {
            Color::srgb(0.0, 1.0, 0.0)
        } else {
            Color::srgb(1.0, 0.0, 0.0)
        };
    return color;
}

#[derive(Resource)]
pub struct EmissiveColorConfig {
    pub factor: f32
}
impl Default for EmissiveColorConfig {
    fn default() -> Self {
        Self { factor: 1.0 }
    }
}

pub fn update_emissive_color<T : AtomicTransition>(
    query: Query<(&MeshMaterial3d<StandardMaterial>, &TotalPhotonsScattered<T>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_step: Res<Timestep>,
    config: Res<EmissiveColorConfig>
)
where T : Default + Copy + Component
{
    for (material, total_scattered) in query.iter() {
        let expected_max = (T::gamma() / 2.0 * time_step.delta) as f32;
        match materials.get_mut(material) {
            None => {}
            Some(material_instance) => {
                let emissive_strength  = clamp(config.factor * total_scattered.total as f32 / expected_max, 0.0, 1.0);
                material_instance.emissive = material_instance.base_color.to_linear() * emissive_strength;
            }
        }
    }
}
