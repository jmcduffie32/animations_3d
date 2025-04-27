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

const FRACTAL_DEF: [[u32; 4]; 4] = [[2, 3, 0, 1], [1, 2, 3, 0], [0, 1, 2, 3], [3, 0, 1, 2]];

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

#[derive(Resource)]
struct FractalDepth {
    depth: u32,
}

#[derive(Resource)]
struct FractalDef {
    value: Vec<Vec<u32>>,
}

#[wasm_bindgen]
pub fn start() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin)
        .insert_resource(FractalDepth { depth: 0 })
        .insert_resource(FractalDef {
            value: FRACTAL_DEF.iter().map(|&x| x.to_vec()).collect(),
        })
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
    fractal_def: &Vec<Vec<u32>>,
) {
    if n == 0 {
        spawn_cube(commands, meshes, scale, position);
    } else {
        let new_scale = scale / fractal_def.len() as f32;
        for i in 0..fractal_def.len() {
            for j in 0..fractal_def[i].len() {
                spawn_fractal_recursive(
                    commands,
                    meshes,
                    new_scale,
                    position
                        + Vec3::new(
                            i as f32 * new_scale,
                            j as f32 * new_scale,
                            fractal_def[i][j] as f32 * new_scale,
                        ),
                    n - 1,
                    fractal_def,
                );
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    fractal_depth: Res<FractalDepth>,
    fractal_def: Res<FractalDef>,
) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
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
        fractal_def.value.len() as f32,
        Vec3::new(0.0, 0.0, 0.0),
        fractal_depth.depth,
        &fractal_def.value,
    );
}

fn ui_fractal_depth(
    mut contexts: EguiContexts,
    mut fractal_depth: ResMut<FractalDepth>,
    mut fractal_def: ResMut<FractalDef>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<Shape>>,
) {
    egui::Window::new("Fractal Definition").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut fractal_depth.depth, 0..=8).text("Fractal Depth"));

        let mut fractal_def_input = build_fractal_def_str(&fractal_def);
        ui.horizontal(|ui| {
            ui.label("Fractal Definition:");
            ui.text_edit_singleline(&mut fractal_def_input);
        });

        // allow the user to set each element of the fractal definition
        let fractal_def_vec = str_to_fractal_def(fractal_def_input);
        fractal_def.value = fractal_def_vec.clone();
    });

    // input for dimension of the fractal definition

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
        &fractal_def.value,
    );
}

const FRACTAL_DEF_ELEMNET_SEPARATOR: &str = ",";
const FRACTAL_DEF_ROW_SEPARATOR: &str = "|";

fn str_to_fractal_def(fractal_def_input: String) -> Vec<Vec<u32>> {
    let mut fractal_def_vec: Vec<Vec<u32>> = vec![];
    for (_i, row) in fractal_def_input
        .split(FRACTAL_DEF_ROW_SEPARATOR)
        .enumerate()
    {
        let mut row_vec: Vec<u32> = vec![];
        for (_j, elem) in row.split(FRACTAL_DEF_ELEMNET_SEPARATOR).enumerate() {
            if let Ok(num) = elem.trim().parse::<u32>() {
                row_vec.push(num);
            }
        }
        fractal_def_vec.push(row_vec);
    }
    fractal_def_vec
}

fn build_fractal_def_str(fractal_def: &ResMut<'_, FractalDef>) -> String {
    let fractal_def_input = fractal_def
        .value
        .iter()
        .map(|x| {
            x.iter()
                .map(|y| y.to_string())
                .collect::<Vec<String>>()
                .join(FRACTAL_DEF_ELEMNET_SEPARATOR)
        })
        .collect::<Vec<String>>()
        .join(FRACTAL_DEF_ROW_SEPARATOR);
    fractal_def_input
}

fn main() {
    start()
}
