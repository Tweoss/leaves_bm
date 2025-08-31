mod egui;
mod keyboard;
mod pan_orbit;
mod render;

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_egui::PrimaryEguiContext;
use bevy_render::view::RenderLayers;
use leaves_bm::{
    math::{Int3, Vec3},
    Bound3, Constants, Float, Simulation,
};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    egui::{ColorBounds, Function, InitParams, SimControls, UiState},
    keyboard::handle_keystrokes,
    render::{CustomMaterialPlugin, InstanceData, InstanceMaterialData},
};

const X_COUNT: usize = 40;
const Y_COUNT: usize = 40;
const Z_COUNT: usize = 40;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 2.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // particles
    let particle_count = 10;
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);
    let particles = (0..particle_count)
        .map(|_| {
            let (x, y, z) = (
                rng.random_range(0..X_COUNT),
                rng.random_range(0..Y_COUNT),
                rng.random_range(0..Z_COUNT),
            );
        })
        .collect::<Vec<_>>();

    // instanced boxes
    let bundle = (
        Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 0.4))),
        InstanceMaterialData(
            (0..X_COUNT)
                .flat_map(|x| (0..Y_COUNT).map(move |y| (x, y)))
                .flat_map(|(x, y)| (0..Z_COUNT).map(move |z| (x, y, z)))
                .map(|(x, y, z)| {
                    let (x, y, z) = (x as f32, y as f32, z as f32);
                    InstanceData {
                        position: bevy::prelude::Vec3::new(
                            X_COUNT as f32 / 2.0 - x,
                            Y_COUNT as f32 / 2.0 - y,
                            Z_COUNT as f32 / 2.0 - z,
                        ),
                        scale: 1.0,
                        color: LinearRgba::from(Color::WHITE).to_f32_array(),
                    }
                })
                .collect(),
        ),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        NoFrustumCulling,
    );
    commands.spawn(bundle);

    // egui camera
    commands.spawn((
        Camera2d,
        Name::new("Egui Camera"),
        PrimaryEguiContext,
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));
}

fn step_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut sim: ResMut<SimulationRes>,
    mut controls: ResMut<SimControls>,
    params: Res<InitParams>,
    mut handles: Query<&mut InstanceMaterialData>,
    color_bounds: Res<ColorBounds>,
) {
    type Init = Box<dyn Fn(usize, usize, usize, Int3, Float) -> Option<f32>>;
    let wave: Init = Box::new(|x, _, _, dir, _| {
        let vec: Vec3 = dir.into();
        if x != 0 && x + 1 != X_COUNT {
            return None;
        }
        let x_f = x as f32 - (X_COUNT as f32 / 2.0);
        let magnitude = vec.dot(Vec3::new(if x_f > 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0)) / 20.0;
        (magnitude > 0.0).then_some(magnitude)
    });
    let circular: Init = Box::new(|x, y, z, dir, weight| {
        let vec: Vec3 = dir.into();
        let x_range = (X_COUNT / 4)..(X_COUNT * 3 / 4);
        let y_range = (Y_COUNT / 4)..(Y_COUNT * 3 / 4);
        if !x_range.contains(&x) {
            return None;
        }
        if !y_range.contains(&y) {
            return None;
        }
        if z != 0 {
            return None;
        }
        let (x_f, y_f) = (
            (x as i32 - (X_COUNT as i32 / 2)) as f32,
            (y as i32 - (Y_COUNT as i32 / 2)) as f32,
        );
        let magnitude = vec.dot(Vec3::new(-y_f, x_f, 0.0)) * weight;
        (magnitude > 0.0).then_some(magnitude)
    });
    let point: Init = Box::new(|x, y, z, dir, _| {
        if x != X_COUNT / 2 || y != Y_COUNT / 2 || z != Z_COUNT / 2 {
            return None;
        }
        if dir == Int3::ZERO {
            Some(70.0)
        } else {
            None
        }
    });

    let init_func = match params.function {
        Function::Wave => wave,
        Function::Circle => circular,
        Function::Point => point,
    };
    let scale = params.scale;

    let mut rerender = false;

    if controls.restart_requested {
        rerender = true;

        fn filler(dir: Int3) -> f32 {
            if dir == Int3::ZERO {
                1.0
            } else {
                0.0
            }
        }

        let constants = sim.0.constants;
        let mut new_sim = Simulation::new(constants);
        new_sim
            .distributions
            .iter_mut()
            .for_each(|(dist, dir, weight)| {
                for x in 0..X_COUNT {
                    for y in 0..Y_COUNT {
                        for z in 0..Z_COUNT {
                            *dist.get_mut(Bound3::new(x, y, z).unwrap()) =
                                init_func(x, y, z, dir, weight)
                                    .map(|v| v * scale)
                                    .unwrap_or_else(|| filler(dir));
                        }
                    }
                }
            });
        new_sim.calc_conditions();

        *sim = SimulationRes(new_sim);
        controls.restart_requested = false;
    }

    if !controls.paused && timer.0.tick(time.delta()).just_finished() {
        use std::time::Instant;
        let start = Instant::now();
        sim.0.step();
        if controls.single_step {
            controls.paused = true;
            controls.single_step = false;
        }
        rerender = true;
        dbg!(Instant::now().duration_since(start));
    }

    rerender |= color_bounds.is_changed();

    if rerender {
        for mut material_data in handles.iter_mut() {
            #[allow(clippy::modulo_one)]
            for (i, data) in material_data.0.iter_mut().enumerate() {
                let z = i % Z_COUNT;
                let y = (i / Z_COUNT) % Y_COUNT;
                let x = i / Z_COUNT / Y_COUNT;
                let value = (*sim.0.density.get(Bound3::new(x, y, z).unwrap()) - color_bounds.min)
                    / (color_bounds.max - color_bounds.min);
                let value = value.min(1.0);
                data.color = [value, value, value, 1.0];
                if value == 1.0 {
                    data.color = [1.0, 0.0, 0.0, 1.0]
                }
            }
        }
    }
}

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct SimulationRes(Simulation<X_COUNT, Y_COUNT, Z_COUNT>);

fn main() {
    let sim = Simulation::new(Constants {
        // Should be greater than 1 for some reason.
        time_relaxation_constant: 1.25,
        speed_of_sound: 1.0 / (3.0_f32).sqrt(),
    });

    let sim_res = SimulationRes(sim);

    use pan_orbit::{pan_orbit_camera, spawn_camera, PanOrbitState};
    use std::time::Duration;
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "LBM Simulator".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            CustomMaterialPlugin,
        ))
        // Debugger items.
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)
        .register_type::<ColorBounds>()
        .insert_resource(sim_res)
        .insert_resource(ColorBounds { min: 1.0, max: 1.5 })
        .insert_resource(SimControls {
            restart_requested: true,
            single_step: false,
            paused: false,
        })
        .insert_resource(InitParams {
            function: Function::Circle,
            scale: 1.0,
        })
        .insert_resource(SimulationTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .insert_resource(UiState::new())
        .add_systems(Startup, (setup, spawn_camera))
        .add_systems(bevy_egui::EguiPrimaryContextPass, egui::show_ui_system)
        .add_systems(
            PostUpdate,
            egui::set_camera_viewport.after(egui::show_ui_system),
        )
        .add_systems(
            Update,
            (
                pan_orbit_camera.run_if(any_with_component::<PanOrbitState>),
                step_simulation,
                handle_keystrokes,
            ),
        )
        .run();
}
