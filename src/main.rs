use bevy::log;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use inline_tweak::tweak;
use noisy::NoisyVertMaterial;

mod noisy;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample8)
        .insert_resource(ClearColor(Color::GRAY))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(MaterialPlugin::<ExtendedMaterial<NoisyVertMaterial>>::default())
        .add_startup_system(setup)
        .add_system(set_custom_material)
        .add_system(rotate_model)
        .add_system(animate_model)
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
        CustomMaterial,
    ));
}

#[derive(Component, Debug, Clone)]
struct CustomMaterial;

fn set_custom_material(
    mut commands: Commands,
    scenes: Query<(Entity, &SceneInstance), With<CustomMaterial>>,
    materials: Query<(Entity, &Handle<StandardMaterial>)>,
    scene_manager: Res<SceneSpawner>,
    standard_mats: Res<Assets<StandardMaterial>>,
    mut noisy_mats: ResMut<Assets<ExtendedMaterial<NoisyVertMaterial>>>,
) {
    for (entity, instance) in &scenes {
        if !scene_manager.instance_is_ready(**instance) {
            log::debug!("scene instance {entity:?} not spawned yet");
            continue;
        }

        // Based on https://github.com/bevyengine/bevy/discussions/8533
        for scene_ent in scene_manager.iter_instance_entities(**instance) {
            let Ok((ent, standard_mat)) = materials.get(scene_ent) else { continue };
            let Some(standard) = standard_mats.get(standard_mat) else { continue };

            // hmm, this part could probably be done at startup, idk though
            let noisy_mat = noisy_mats.add(ExtendedMaterial {
                standard: standard.clone(),
                extended: NoisyVertMaterial::default(),
            });

            log::debug!("updating {ent:?} material to {noisy_mat:?}");

            commands
                .entity(ent)
                .remove::<Handle<StandardMaterial>>()
                .insert(noisy_mat);
        }

        log::info!("scene {entity:?} custom material set, removing marker component");
        commands.entity(entity).remove::<CustomMaterial>();
    }
}

fn rotate_model(time: Res<Time>, mut query: Query<&mut Transform, With<Colette>>) {
    for mut model in &mut query {
        model.rotate_y(tweak!(1.5) * time.delta_seconds());
    }
}

fn animate_model(
    material_handles: Query<&Handle<ExtendedMaterial<NoisyVertMaterial>>>,
    mut materials: ResMut<Assets<ExtendedMaterial<NoisyVertMaterial>>>,
) {
    for handle in &material_handles {
        let Some(material) = materials.get_mut(handle) else { continue };

        // TODO: bevy_inspector_egui would probably be nice for these
        material.extended.noise_magnitude = tweak!(0.25);
        material.extended.noise_scale = tweak!(75.0);
    }

    // TODO: add UI button to play animation or something?

    // First half:
    //  - apply material with noisy vertex shader

    // Second half:
    //  - explode into blobs
    //      - SDF spheres? or just a basic billboard type particle
    //      - possibly implemented with a more typical particle system in the real
    //        game, but let's try with a shader just to see if it's feasible
    //
    //  - explosion particle effect itself. TBD what this would look like
    //
    //  - move offscreen

    // Spawn in:
    //  - drop in from top while spinning (spherical blob shape)
    //  - reformulate into full sprite over time
}
