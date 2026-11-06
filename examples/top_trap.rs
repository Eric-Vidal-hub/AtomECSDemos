//! Time-Orbiting Potential (TOP) trap
//!
//! cargo install -f wasm-bindgen-cli
//! cargo build --example top_trap --target wasm32-unknown-unknown
//! wasm-bindgen --out-dir target/web target/wasm32-unknown-unknown/release/examples/doppler_limit.wasm
//!
extern crate atomecs;
extern crate nalgebra;
use atomecs::atom::{Atom, Force, Mass, Position, Velocity};
use atomecs::initiate::NewlyCreated;
use atomecs::integrator::Timestep;
use atomecs::magnetic::force::MagneticDipole;
use atomecs::magnetic::quadrupole::QuadrupoleField3D;
use atomecs::magnetic::top::UniformFieldRotator;
use atomecs::species::Rubidium87_780D2;
use atomecs_demos::atoms::add_meshes_to_atoms;
use atomecs_demos::camera::{control_camera, DemoCamera, ortho_camera};
use atomecs_demos::{add_atomecs_watermark, BevyAtomECSPlugin};
use bevy::prelude::*;
use bevy::render::camera::Projection;
use nalgebra::Vector3;
use rand_distr::{Distribution, Normal};

fn main() {
    let mut app = App::new();

    // Add magnetics systems (todo: as plugin)
    app.add_plugins(atomecs::integrator::IntegrationPlugin);
    app.add_plugins(atomecs::magnetic::MagneticsPlugin);
    app.add_systems(Update, atomecs::output::console_output::console_output);
    app.add_plugins(DefaultPlugins);
    app.add_systems(Update, atomecs::bevy_bridge::copy_positions);
    app.add_systems(Update, control_camera);
    app.add_systems(Update, add_meshes_to_atoms::<Rubidium87_780D2>);
    app.add_systems(Startup, add_atomecs_watermark);
    app.add_plugins(BevyAtomECSPlugin);
    app.add_systems(Startup, setup);
    app.insert_resource(atomecs::bevy_bridge::Scale { 0: 1e4 });
    app.add_systems(Startup, setup_atoms);

    // Create magnetic field.
    app.world_mut()
        .spawn(QuadrupoleField3D::gauss_per_cm(80.0, Vector3::z()))
        .insert(Position::default());

    app.world_mut().spawn(UniformFieldRotator { amplitude: 20.0, frequency: 3000.0 }) // Time averaged TOP theory assumes rotation frequency much greater than velocity of atoms
        .insert(atomecs::magnetic::uniform::UniformMagneticField { field: Vector3::new(0.0,0.0,0.0)}) // Time averaged TOP theory assumes rotation frequency much greater than velocity of atoms
        ;

    // Define timestep
    app.world_mut().insert_resource(Timestep { delta: 5e-5 }); //Aliasing of TOP field or other strange effects can occur if timestep is not much smaller than TOP field period.
                                                         //Timestep must also be much smaller than mean collision time.

    // Run the simulation for a number of steps.
    // for _i in 0..10000 {
    //      app.update();
    // }
    app.run();
}

fn setup_atoms(mut commands: Commands) {
    let p_dist = Normal::new(0.0, 50e-6).unwrap();
    let v_dist = Normal::new(0.0, 0.004).unwrap(); // ~100nK

    for _i in 0..5000 {
        commands
            .spawn(Position {
                pos: Vector3::new(
                    p_dist.sample(&mut rand::rng()),
                    p_dist.sample(&mut rand::rng()),
                    0.35 * p_dist.sample(&mut rand::rng()), //TOP traps have tighter confinement along quadrupole axis
                ),
            })
            .insert(Atom)
            .insert(Force::default())
            .insert(Velocity {
                vel: Vector3::new(
                    v_dist.sample(&mut rand::rng()),
                    v_dist.sample(&mut rand::rng()),
                    v_dist.sample(&mut rand::rng()),
                ),
            })
            .insert(NewlyCreated)
            .insert(MagneticDipole { mFgF: 0.5 })
            .insert(Mass { value: 87.0 })
            // .insert_bundle(PbrBundle {
            //     mesh: meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
            //     material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            //     transform: Transform::from_xyz(1.5, 0.5, 1.5),
            //     ..default()
            // })
            .insert(Rubidium87_780D2);
    }
}

fn setup(mut commands: Commands) {
    // set up the camera
    commands
        .spawn((
            Camera3d::default(),
            Projection::Orthographic(ortho_camera(0.01)),
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(DemoCamera::default());

    commands.spawn((
        PointLight {
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}
