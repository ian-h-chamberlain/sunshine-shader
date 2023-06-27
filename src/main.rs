use bevy::prelude::*;
use inline_tweak::tweak;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::GRAY))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        // .add_startup_system(setup_debug)
        .add_system(rotate_model)
        .run();
}

#[derive(Component)]
struct Colette;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.0, 1.5, -4.0)
            .looking_at(Vec3::new(0.0, 0.75, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(1.0, 0.7, 0.1),
            intensity: 3500.0,
            radius: 0.25,
            ..default()
        },
        transform: Transform::from_xyz(-3.5, 3.0, 0.75),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(1.0, 0.5, 0.7),
            intensity: 6000.0,
            radius: 0.25,
            ..default()
        },
        transform: Transform::from_xyz(-3.5, 3.0, 0.75),
        ..default()
    });

    commands.spawn((
        SceneBundle {
            scene: asset_server.load("colette/Colette.gltf#Scene0"),
            ..default()
        },
        Colette,
    ));
}

fn rotate_model(time: Res<Time>, mut query: Query<&mut Transform, With<Colette>>) {
    for mut model in &mut query {
        model.rotate_y(tweak!(1.5) * time.delta_seconds());
    }
}
