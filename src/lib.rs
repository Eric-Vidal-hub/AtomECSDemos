pub mod atoms;
pub mod camera;
pub mod lasers;

use bevy::prelude::*;

pub struct BevyAtomECSPlugin;
impl Plugin for BevyAtomECSPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Update, atoms::add_meshes_to_atoms);
        app.init_resource::<atoms::MaterialColorConfig>();
    }
}

pub fn add_atomecs_watermark(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
        commands
        .spawn((
            // Create a UI text node with the AtomECS watermark.
            Text::new("AtomECS"),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 50.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                align_self: AlignSelf::End,
                ..default()
            },
        ));
}
