mod egui;
mod keyboard;
mod pan_orbit;
mod render;

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_egui::PrimaryEguiContext;
use bevy_render::view::RenderLayers;
use leaves_bm::{Bound3, Constants, Float, Simulation};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    egui::{ColorBounds, RestartSim, UiState},
    keyboard::handle_keystrokes,
    render::{CustomMaterialPlugin, InstanceData, InstanceMaterialData},
};

const X_COUNT: i32 = 30;
const Y_COUNT: i32 = 30;
const Z_COUNT: i32 = 30;

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
                        position: Vec3::new(
                            X_COUNT as f32 / 2.0 - x,
                            Y_COUNT as f32 / 2.0 - y,
                            Z_COUNT as f32 / 2.0 - z,
                        ),
                        scale: 1.0,
                        color: LinearRgba::from(Color::hsla(x * 360., y, 0.5, 1.0)).to_f32_array(),
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

    // light
    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
            // shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-90.0)),
    ));
}

fn step_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut sim: ResMut<SimulationRes>,
    mut restart_sim: ResMut<RestartSim>,
    mut handles: Query<&mut InstanceMaterialData>,
    color_bounds: Res<ColorBounds>,
) {
    if restart_sim.0 {
        let constants = sim.0.constants;
        let mut new_sim = Simulation::new(constants);
        for i in 0..(X_COUNT as usize) {
            for j in 0..(Y_COUNT as usize) {
                // *new_sim
                //     .distributions
                //     .q0
                //     .get_mut(Bound3::new(i, j, (Z_COUNT as usize) / 2).unwrap()) = 10.0;

                // *sim.distributions.q1[0].get_mut(Bound3::new(i, j, 2).unwrap()) = (i as Float).sqrt();
                // *sim.distributions.q1[0].get_mut(Bound3::new(i, j, 2).unwrap()) = (i as Float).sqrt();
                *new_sim.distributions.q1[(j / 5) % 6].get_mut(Bound3::new(i, j, 2).unwrap()) =
                    (i as Float).abs().sqrt();
            }
        }
        // *new_sim
        //     .distributions
        //     .q0
        //     .get_mut(Bound3::new(0, 0, 0).unwrap()) = 10.0;
        // *new_sim
        //     .distributions
        //     .q0
        //     .get_mut(Bound3::new(4, 0, 0).unwrap()) = 10.0;
        // *new_sim
        //     .distributions
        //     .q0
        //     .get_mut(Bound3::new(3, 0, 0).unwrap()) = 10.0;
        // *new_sim
        //     .distributions
        //     .q0
        //     .get_mut(Bound3::new(2, 2, 0).unwrap()) = 10.0;
        new_sim.calc_conditions();
        *sim = SimulationRes(new_sim);
        restart_sim.0 = false;

        return;
    }
    if timer.0.tick(time.delta()).just_finished() {
        use std::time::Instant;
        let start = Instant::now();
        sim.0.step();
        // dbg!(Instant::now().duration_since(start));

        for mut material_data in handles.iter_mut() {
            for (i, data) in material_data.0.iter_mut().enumerate() {
                let i = i as i32;
                let x = i % X_COUNT;
                let y = (i / X_COUNT) % Y_COUNT;
                let z = i / X_COUNT / Y_COUNT;
                let value = (*sim
                    .0
                    .density
                    .get(Bound3::new(x as usize, y as usize, z as usize).unwrap())
                    - color_bounds.min)
                    / (color_bounds.max - color_bounds.min);
                // give a bit of nonlinearity to better see values close to 1.0
                // .sqrt();
                data.color = [value, value, value, 1.0];
            }
        }
    }
}

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct SimulationRes(Simulation<{ X_COUNT as usize }, { Y_COUNT as usize }, { Z_COUNT as usize }>);

fn main() {
    let sim = Simulation::new(Constants {
        time_relaxation_constant: 0.3,
        speed_of_sound: 1.0 / (3.0_f32).sqrt(),
    });

    let sim_res = SimulationRes(sim);

    use pan_orbit::{pan_orbit_camera, spawn_camera, PanOrbitState};
    use std::time::Duration;
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "BLM Simulator".to_string(),
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
        .register_type::<RestartSim>()
        .insert_resource(sim_res)
        .insert_resource(ColorBounds {
            min: 1.0,
            max: 10.0,
        })
        .insert_resource(RestartSim(true))
        .insert_resource(SimulationTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        // Debugger resources
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
