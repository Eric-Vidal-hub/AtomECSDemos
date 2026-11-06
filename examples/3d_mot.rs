//! # Doppler Sweep
//!
//! Simulate a cloud of atoms in a 3D MOT to measure the Doppler temperature limit for laser cooling.
//!
//! The Doppler Limit depends on temperature, see eg https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.61.169.
//!
//! Some parameters of the simulation can be set by writing a configuration file called `doppler.json`. This file
//! allows the user to control parameters, eg detuning. If the file is not written, a default detuning of 0.5 Gamma
//! is used, which corresponds to the minimum Doppler temperature.
//!
//! cargo build --example doppler_limit --target wasm32-unknown-unknown --release
//! wasm-bindgen --out-dir target/web target/wasm32-unknown-unknown/release/examples/doppler_limit.wasm --target web

extern crate atomecs;
extern crate nalgebra;
use atomecs::atom::{Atom, Force, Mass, Position, Velocity};
use atomecs::initiate::NewlyCreated;
use atomecs::integrator::Timestep;
use atomecs::laser::gaussian::GaussianBeam;
use atomecs::laser::LaserPlugin;
use atomecs::laser_cooling::force::{EmissionForceConfiguration, EmissionForceOption};
use atomecs::laser_cooling::photons_scattered::ScatteringFluctuationsOption;
use atomecs::laser_cooling::{CoolingLight, LaserCoolingPlugin};
use atomecs::magnetic::quadrupole::QuadrupoleField3D;
use atomecs::species::Rubidium87_780D2;
use atomecs_demos::atoms::add_meshes_to_atoms;
use atomecs_demos::camera::ortho_camera;
use atomecs_demos::BevyAtomECSPlugin;
use bevy::prelude::*;
use bevy::render::camera::Projection;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};

const BEAM_NUMBER: usize = 6;

pub struct DopperSimulationConfiguration {
    /// Detuning of laser beams, in units of MHz.
    pub detuning: f64,
    /// Number of simulation steps to evolve for.
    pub number_of_steps: i32,
}
impl Default for DopperSimulationConfiguration {
    fn default() -> Self {
        DopperSimulationConfiguration {
            detuning: -8.0,
            number_of_steps: 5000,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(atomecs::integrator::IntegrationPlugin);
    app.add_plugins(atomecs::initiate::InitiatePlugin);
    app.add_plugins(atomecs::magnetic::MagneticsPlugin);
    app.add_plugins(LaserPlugin::<{ BEAM_NUMBER }>);
    app.add_plugins(LaserCoolingPlugin::<Rubidium87_780D2, { BEAM_NUMBER }>::default());
    app.add_plugins(BevyAtomECSPlugin);
    app.add_systems(Update, add_meshes_to_atoms::<Rubidium87_780D2>);
    app.add_systems(Update, atomecs::output::console_output::console_output);
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }));
    app.add_systems(Update, atomecs::bevy_bridge::copy_positions);
    app.add_systems(Startup, setup_world);
    app.add_systems(Startup, create_atoms);
    app.add_systems(Startup, setup_camera);
    //app.add_systems(Startup, add_atomecs_watermark);
    app.insert_resource(atomecs::bevy_bridge::Scale { 0: 1e3 });
    app.insert_resource(Timestep { delta: 2.0e-5 });
    app.insert_resource(EmissionForceOption::On(EmissionForceConfiguration {
        explicit_threshold: 5,
    }));
    app.insert_resource(ScatteringFluctuationsOption::On);
    app.run();
}

pub fn setup_world(mut commands: Commands) {
    let configuration = DopperSimulationConfiguration::default();

    // Create magnetic field.
    commands
        .spawn(QuadrupoleField3D::gauss_per_cm(0.001 * 18.2, Vector3::z()))
        .insert(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        });

    // Create cooling lasers.
    let detuning = configuration.detuning;
    let power = 0.02;
    let radius = 66.7e-3 / (2.0_f64.sqrt());
    let beam_centre = Vector3::new(0.0, 0.0, 0.0);

    commands
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, 1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, -1,
        ));
    commands
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 0.0, -1.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, -1,
        ));
    commands
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(-1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
    commands
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(1.0, 0.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
    commands
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, 1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
    commands
        .spawn(GaussianBeam {
            intersection: beam_centre,
            e_radius: radius,
            power,
            direction: Vector3::new(0.0, -1.0, 0.0),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .insert(CoolingLight::for_transition::<Rubidium87_780D2>(
            detuning, 1,
        ));
}

fn create_atoms(mut commands: Commands) {
    let vel_dist = Normal::new(0.0, 0.42).unwrap();
    let pos_dist = Normal::new(0.0, 1.2e-4).unwrap();
    let mut rng = rand::rng();

    // Add atoms
    for _ in 0..1000 {
        commands
            .spawn(Position {
                pos: Vector3::new(
                    pos_dist.sample(&mut rng),
                    pos_dist.sample(&mut rng) - 0.002,
                    pos_dist.sample(&mut rng),
                ),
            })
            .insert(Velocity {
                vel: Vector3::new(
                    vel_dist.sample(&mut rng),
                    vel_dist.sample(&mut rng) + 3.5,
                    vel_dist.sample(&mut rng),
                ),
            })
            .insert(Force::default())
            .insert(Mass { value: 87.0 })
            .insert(Rubidium87_780D2)
            .insert(Atom)
            .insert(NewlyCreated);
    }
}

fn setup_camera(mut commands: Commands) {
    // set up the camera
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(ortho_camera(0.01)),
        Transform::from_xyz(4.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

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
