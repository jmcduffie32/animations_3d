use bevy::ecs::component::Component;
use bevy::pbr::PointLight;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::{
    app::{App, Startup},
    asset::Assets,
    ecs::system::{Commands, ResMut},
    math::{primitives::Cuboid, Vec3},
    pbr::{MeshMaterial3d, StandardMaterial},
    render::mesh::{Mesh, Mesh3d},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use wasm_bindgen::prelude::*;

// const FRACTAL_DEF: [[u32; 3]; 3] = [[1, 0, 2], [0, 2, 1], [2, 1, 0]];
const FRACTAL_DEF: [[u32; 2]; 2] = [[1, 0], [0, 1]];

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

#[derive(Resource)]
struct FractalDepth {
    depth: u32,
}

#[wasm_bindgen]
pub fn start() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin)
        .insert_resource(FractalDepth { depth: 0 })
        .add_systems(Startup, setup)
        .add_systems(Update, ui_fractal_depth)
        .run();
}

fn spawn_cube(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    scale: f32,
    position: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(scale, scale, scale))),
        MeshMaterial3d::<StandardMaterial>(Default::default()),
        Transform::from_translation(position),
        Shape,
    ));
}

fn spawn_fractal_recursive(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    scale: f32,
    position: Vec3,
    n: u32,
) {
    if n == 0 {
        spawn_cube(commands, meshes, scale, position);
    } else {
        let new_scale = scale / FRACTAL_DEF.len() as f32;
        for i in 0..FRACTAL_DEF.len() {
            for j in 0..FRACTAL_DEF[i].len() {
                spawn_fractal_recursive(
                    commands,
                    meshes,
                    new_scale,
                    position
                        + Vec3::new(
                            i as f32 * new_scale,
                            j as f32 * new_scale,
                            FRACTAL_DEF[i][j] as f32 * new_scale,
                        ),
                    n - 1,
                );
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    fractal_depth: Res<FractalDepth>,
) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        PanOrbitCamera::default(),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));


    spawn_fractal_recursive(
        &mut commands,
        &mut meshes,
        3.0,
        Vec3::new(0.0, 0.0, 0.0),
        fractal_depth.depth,
    );
}

fn ui_fractal_depth(
    mut contexts: EguiContexts,
    mut fractal_depth: ResMut<FractalDepth>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<Shape>>,
) {
    egui::Window::new("Fractal Depth").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut fractal_depth.depth, 0..=8).text("Fractal Depth"));
    });

    // Clear existing fractal shapes
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn fractal with updated depth
    spawn_fractal_recursive(
        &mut commands,
        &mut meshes,
        4.0,
        Vec3::new(0.0, 0.0, 0.0),
        fractal_depth.depth,
    );
}

fn main() {
    start()
}
