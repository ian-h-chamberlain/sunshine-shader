use bevy::asset::HandleId;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::renderer::RenderDevice;
use bevy::render::settings::WgpuFeatures;
use bevy::render::view::NoFrustumCulling;
use bevy::scene::SceneInstance;
use bevy::utils::HashMap;
use inline_tweak::tweak;

mod bubbles;
mod noisy;

use self::bubbles::{BubblesMaterial, BubblesMaterialPlugin};
use self::noisy::NoisyVertsMaterial;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (800.0, 600.0).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            }),
    );

    let render_device = app.world.resource::<RenderDevice>();

    // damn, seems like a web demo with this is probably not viable, since this feature is
    // listed as native-only. :(
    if !render_device
        .features()
        .contains(WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
    {
        error!(
            "Render device doesn't support feature \
            SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, \
            which is required for texture binding arrays"
        );
        return;
    }

    app.insert_resource(Msaa::Sample8)
        .insert_resource(ClearColor(Color::GRAY))
        .init_resource::<Materials>()
        .add_plugin(BubblesMaterialPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MaterialPlugin::<NoisyVertsMaterial>::default())
        .add_startup_system(setup)
        .add_system(initialize_materials)
        .add_system(set_custom_material)
        .add_system(rotate_model)
        .add_system(animate_noise)
        .add_system(animate_bubbles)
        // GO!
        .run();
}

#[derive(Resource, Debug, Default)]
struct Materials {
    bubbles: HashMap<HandleId, Handle<BubblesMaterial>>,
    noisy: HashMap<HandleId, Handle<NoisyVertsMaterial>>,
}

#[derive(Component)]
struct Colette;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.0, 1.5, -2.0)
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
        UseCustomMaterial,
        NoFrustumCulling,
    ));
}

#[derive(Component)]
struct UseCustomMaterial;

fn initialize_materials(
    standard: Res<Assets<StandardMaterial>>,
    mut bubbles: ResMut<Assets<BubblesMaterial>>,
    mut noisy_mats: ResMut<Assets<NoisyVertsMaterial>>,
    mut materials: ResMut<Materials>,
) {
    // only need to rerun this whenever new standard materials are added
    if !standard.is_changed() {
        return;
    }

    log::debug!(
        "got {} materials to copy into custom materials",
        standard.len()
    );

    for (id, standard) in standard.iter() {
        materials.bubbles.entry(id).or_insert_with(|| {
            log::debug!("creating bubbles mat for for {id:?}");

            bubbles.add(bubbles::material_from_standard(standard.clone()))
        });

        materials.noisy.entry(id).or_insert_with(|| {
            log::debug!("creating noisy mat for for {id:?}");

            noisy_mats.add(NoisyVertsMaterial {
                standard: standard.clone(),
                extended: default(),
            })
        });
    }
}

fn set_custom_material(
    mut commands: Commands,
    scenes: Query<(Entity, &SceneInstance), With<Colette>>,
    ent_materials: Query<(Entity, &Handle<StandardMaterial>, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
    scene_manager: Res<SceneSpawner>,
    materials: Res<Materials>,
) {
    for (entity, instance) in &scenes {
        if !scene_manager.instance_is_ready(**instance) {
            log::debug!("scene instance {entity:?} not spawned yet");
            continue;
        }

        // Based on https://github.com/bevyengine/bevy/discussions/8533
        for scene_ent in scene_manager.iter_instance_entities(**instance) {
            let Ok((ent, standard_mat, mesh_handle)) = ent_materials.get(scene_ent) else { continue };

            let Some(mesh) = meshes.get(mesh_handle)
            else {
                log::error!("mesh not found {mesh_handle:?}");
                continue;
            };

            let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                log::error!("no mesh positions found");
                return;
            };

            let colors: String = positions[..positions.len().min(36)]
                .iter()
                .skip(1)
                .step_by(3)
                .map(|&[r, g, b]| {
                    let (r, g, b) = (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
                    format!(
                        "  rgb({}, {}, {}) {:?}\n",
                        (255.0 * r) as u8,
                        (255.0 * g) as u8,
                        (255.0 * b) as u8,
                        [r, g, b]
                    )
                })
                .collect();

            log::debug!("First several positions:\n{colors}");

            let Some(bubble_mat) = materials.bubbles.get(&standard_mat.id()) else { continue };
            let Some(_noisy_mat) = materials.noisy.get(&standard_mat.id()) else { continue };

            log::debug!("updating {ent:?} material to {bubble_mat:?}");

            commands
                .entity(ent)
                .remove::<Handle<StandardMaterial>>()
                .insert(bubble_mat.clone());
        }
    }
}

fn rotate_model(time: Res<Time>, mut query: Query<&mut Transform, With<Colette>>) {
    for mut model in &mut query {
        model.rotate_y(tweak!(0.25) * time.delta_seconds());
    }
}

// First half of the animation: apply material with noisy vertex shader
fn animate_noise(
    material_handles: Query<&Handle<NoisyVertsMaterial>>,
    mut materials: ResMut<Assets<NoisyVertsMaterial>>,
) {
    // TODO: add UI button to play animation or something?

    for handle in &material_handles {
        let Some(material) = materials.get_mut(handle) else { continue };

        // TODO: bevy_inspector_egui would probably be nice for these
        material.extended.noise_magnitude = tweak!(0.15);
        material.extended.noise_scale = tweak!(60.0);
        material.extended.time_scale = tweak!(4.0);
    }
}

// Second half:
//  - explode into blobs
//      - SDF spheres? or just a basic billboard type particle
//      - possibly implemented with a more typical particle system in the real
//        game, but let's try with a shader just to see if it's feasible
//
fn animate_bubbles(
    material_handles: Query<&Handle<BubblesMaterial>>,
    mut materials: ResMut<Assets<BubblesMaterial>>,
) {
    for handle in &material_handles {
        let Some(material) = materials.get_mut(handle) else { continue };

        // TODO: bevy_inspector_egui would probably be nice for these
        material.extended.bubble_radius = tweak!(0.03);
    }
}

// TODO:
//  - explosion particle effect itself. TBD what this would look like
//  - move offscreen
//  - drop in from top while spinning (spherical blob shape)
//  - reconstitute into full sprite over time
