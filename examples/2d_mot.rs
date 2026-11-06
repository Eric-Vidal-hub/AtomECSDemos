//! A 2D+ mot configuration, loaded directly from oven.

use atomecs::atom::{Atom, Force, Mass};
use atomecs::atom::{Position, Velocity};
use atomecs::initiate::NewlyCreated;
use atomecs::integrator::Timestep;
use atomecs::laser::gaussian::GaussianBeam;
use atomecs::laser::LaserPlugin;
use atomecs::laser_cooling::force::{EmissionForceConfiguration, EmissionForceOption};
use atomecs::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use atomecs::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use atomecs::magnetic::quadrupole::QuadrupoleField3D;
use atomecs::shapes::Cuboid;
use atomecs::sim_region::{SimulationRegionPlugin, SimulationVolume, VolumeType};
use atomecs::species::Strontium88_461;
use atomecs_demos::atoms::{add_meshes_to_atoms, EmissiveColorConfig};
use atomecs_demos::camera::{control_camera, DemoCamera, ortho_camera};
use atomecs_demos::{add_atomecs_watermark, BevyAtomECSPlugin};
use bevy::prelude::*;
use bevy::render::camera::Projection;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};

const BEAM_NUMBER: usize = 6;

fn main() {
    let mut app = App::new();
    app.add_plugins(atomecs::integrator::IntegrationPlugin);
    app.add_plugins(atomecs::initiate::InitiatePlugin);
    app.add_plugins(atomecs::magnetic::MagneticsPlugin);
    app.add_plugins(LaserPlugin::<{ BEAM_NUMBER }>);
    app.add_plugins(LaserCoolingPlugin::<Strontium88_461, { BEAM_NUMBER }>::default());
    app.add_plugins(SimulationRegionPlugin);
    app.add_plugins(BevyAtomECSPlugin);
    app.add_systems(Update, atomecs::output::console_output::console_output);
    app.add_plugins(DefaultPlugins);
    app.add_systems(Update, atomecs::bevy_bridge::copy_positions);
    app.add_systems(Update, add_meshes_to_atoms::<Strontium88_461>);
    app.add_systems(Startup, setup_world);
    app.add_systems(Update, create_atoms);
    app.add_systems(Startup, setup_camera);
    app.add_systems(Update, control_camera);
    app.add_systems(Startup, add_atomecs_watermark);
    app.insert_resource(atomecs::bevy_bridge::Scale { 0: 7e1 });
    app.insert_resource(Timestep { delta: 2.0e-5 });
    app.insert_resource(EmissionForceOption::On(EmissionForceConfiguration {
        explicit_threshold: 5,
    }));
    app.insert_resource(EmissiveColorConfig { factor: 8.0 });
    app.add_systems(
        Update,
        atomecs_demos::atoms::update_emissive_color::<Strontium88_461>
            .after(atomecs::laser_cooling::LaserCoolingSystemsSet::Set),
    );
    app.insert_resource(ScatteringFluctuationsOption::On);
    app.run();
}

pub fn setup_world(mut commands: Commands) {
    // Create magnetic field.
    commands
        .spawn(QuadrupoleField3D::gauss_per_cm(27.0, Vector3::z()))
        .insert(Position::default());

    // Push beam along z
    let push_beam_radius = 1e-3;
    let push_beam_power = 0.020;
    let push_beam_detuning = -103.0;

    commands
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: push_beam_radius,
            power: push_beam_power,
            direction: Vector3::z(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(
            push_beam_detuning,
            -1,
        ));

    // Create cooling lasers.
    let detuning = -40.0;
    let power = 0.23;
    let radius = 17.0e-3; //33.0e-3 / (2.0 * 2.0_f64.sqrt()); // 33mm 1/e^2 diameter
    commands
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, 1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(detuning, 1));
    commands
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, -1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(detuning, 1));
    commands
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, 1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(detuning, 1));
    commands
        .spawn(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Strontium88_461>(detuning, 1));

    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation.
    commands
        .spawn(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .insert(Cuboid {
            half_width: Vector3::new(0.1, 0.01, 0.01),
        })
        .insert(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        });

    // The simulation bound also now includes a small pipe to capture the 2D MOT output properly.
    commands
        .spawn(Position {
            pos: Vector3::new(0.0, 0.0, 0.1),
        })
        .insert(Cuboid {
            half_width: Vector3::new(0.01, 0.01, 0.1),
        })
        .insert(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        });
}

fn create_atoms(mut commands: Commands) {
    let vel_dist = Normal::new(0.0, 2.0).unwrap();
    let mut rng = rand::rng();

    // Add atoms
    commands
        .spawn(Position {
            pos: Vector3::new(-0.08, 0.0, 0.0),
        })
        .insert(Velocity {
            vel: Vector3::new(
                vel_dist.sample(&mut rng) * 5.0 + 50.0,
                vel_dist.sample(&mut rng) * 2.0,
                vel_dist.sample(&mut rng) * 2.0,
            ),
        })
        .insert(Force::default())
        .insert(Mass { value: 88.0 })
        .insert(Strontium88_461)
        .insert(Atom)
        .insert(NewlyCreated);
}

fn setup_camera(mut commands: Commands) {
    // set up the camera
    commands
        .spawn((
            Camera3d::default(),
            Projection::Orthographic(ortho_camera(0.01)),
            Transform::from_xyz(4.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(DemoCamera::default());

    commands.spawn((
        DirectionalLight {
            illuminance: 30000.0,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-2.0),
            ..default()
        },
    ));
}
