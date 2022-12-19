//! Example of using random spawner params.
//! Spawns a random number of particles at random times.

use bevy::{
    log::LogPlugin,
    prelude::*,
    render::{mesh::shape::Cube, render_resource::WgpuFeatures, settings::WgpuSettings},
};
use bevy_inspector_egui::WorldInspectorPlugin;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use bevy_hanabi::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = WgpuSettings::default();
    options
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);
    // options
    //     .features
    //     .set(WgpuFeatures::MAPPABLE_PRIMARY_BUFFERS, false);
    // println!("wgpu options: {:?}", options.features);
    App::default()
        .insert_resource(options)
        .init_resource::<ParticleEffects>()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::WARN,
            filter: "bevy_hanabi=warn,spawn=trace".to_string(),
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(HanabiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_system(spawn_at_random_xy)
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
) {
    let mut camera = Camera3dBundle::default();
    camera.transform.translation = Vec3::new(0.0, 0.0, 100.0);
    commands.spawn(camera);

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            // Crank the illuminance way (too) high to make the reference cube clearly visible
            illuminance: 100000.,
            shadows_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Resource)]
pub struct ParticleEffects {
    entities: Vec<Entity>,
    spawn_next_at_secs: f32,
    rng: StdRng,
}

impl Default for ParticleEffects {
    fn default() -> Self {
        Self {
            entities: Vec::with_capacity(MAX_EFFECTS),
            spawn_next_at_secs: 0.0,
            rng: StdRng::from_entropy()
        }
    }
}

const MAX_EFFECTS: usize = 5;
const SPAWN_EFFECT_EVERY_SECS: f32 = 1.0;

pub fn spawn_at_random_xy(
    mut commands: Commands,
    mut spawned_effects: ResMut<ParticleEffects>,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds_wrapped();
    if now < spawned_effects.spawn_next_at_secs {
        return
    }
    spawned_effects.spawn_next_at_secs = now + SPAWN_EFFECT_EVERY_SECS;

    if spawned_effects.entities.len() >= MAX_EFFECTS {
        commands.entity(spawned_effects.entities.remove(0)).despawn_recursive();
    }

    let cube = meshes.add(Mesh::from(Cube { size: 1.0 }));
    let mat = materials.add(Color::PURPLE.into());

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.0, 0.0, 1.0, 1.0));
    gradient.add_key(1.0, Vec4::new(0.0, 0.0, 1.0, 0.0));

    let effect = effects.add(
        EffectAsset {
            name: "emit:burst".to_string(),
            capacity: 32768,
            spawner: Spawner::rate(Value::Single(10.)),
            ..Default::default()
        }
            .init(PositionSphereModifier {
                center: Vec3::ZERO,
                radius: 5.,
                dimension: ShapeDimension::Volume,
                speed: 2.0.into(),
            })
            .update(AccelModifier {
                accel: Vec3::new(0., 5., 0.),
            })
            .render(ColorOverLifetimeModifier { gradient }),
    );

    let x = spawned_effects.rng.gen_range(-50.0..50.0);
    let y = spawned_effects.rng.gen_range(-20.0..20.0);

    let effect_entity = commands
        .spawn((
            Name::new(format!("emit:random:{},{}", x, y)),
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect),
                transform: Transform::from_translation(Vec3::new(x, y, 0.)),
                ..Default::default()
            },
        ))
        .with_children(|p| {
            // Reference cube to visualize the emit origin
            p.spawn(PbrBundle {
                mesh: cube.clone(),
                material: mat.clone(),
                ..Default::default()
            });
        })
        .id();

    spawned_effects.entities.push(effect_entity);
}