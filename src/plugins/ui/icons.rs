use bevy::prelude::*;

pub struct IconsPlugin;

impl Plugin for IconsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_icons);
    }
}

fn load_icons(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Icons {
        chevron_right: asset_server.load("chevron_right.png"),
        chevron_down: asset_server.load("chevron_down.png"),
    });
}

#[derive(Resource)]
pub struct Icons {
    pub chevron_right: Handle<Image>,
    pub chevron_down: Handle<Image>,
}
