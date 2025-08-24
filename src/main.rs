mod pan_orbit;
mod render;

use std::time::Duration;

use bevy::{prelude::*, render::view::NoFrustumCulling};
use leaves_bm::{Bound3, Constants, Simulation};

use crate::render::{CustomMaterialPlugin, InstanceData, InstanceMaterialData};

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
        Mesh3d(meshes.add(Circle::new(2.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    //

    // instanced boxes
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
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
    // commands.spawn((
    //     PointLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    // ));
    // // camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));
}

fn step_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut sim: ResMut<SimulationRes>,
    mut handles: Query<&mut InstanceMaterialData>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        use std::time::Instant;
        let start = Instant::now();
        sim.0.step();
        dbg!(Instant::now().duration_since(start));

        // let material_start = Instant::now();
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
                    - 1.0)
                    / 90.0;
                data.color = [value, value, value, 1.0];
            }
        }
        // dbg!(Instant::now().duration_since(material_start));
    }
}

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct SimulationRes(Simulation<{ X_COUNT as usize }, { Y_COUNT as usize }, { Z_COUNT as usize }>);

fn main() {
    let mut sim = Simulation::new(Constants {
        time_relaxation_constant: 0.5,
        speed_of_sound: 1.0 / (2.0_f32).sqrt(),
        // speed_of_sound: 1.0 / (3.0_f32).sqrt(),
    });
    for i in 0..X_COUNT {
        for j in 0..Y_COUNT {
            *sim.density
                // *sim.distributions
                //     .q0
                .get_mut(Bound3::new(i as usize, j as usize, 2).unwrap()) = 900.0;
        }
    }
    // *sim.density.get_mut(Bound3::new(0, 3, 5).unwrap()) = 50.0;
    // *sim.distributions.q0.get_mut(Bound3::new(0, 3, 5).unwrap()) = 60.0;

    // *sim.distributions
    //     .q0
    //     .get_mut(Bound3::new(10, 23, 0).unwrap()) = 50.0;
    // sim.calc_conditions();
    // *sim.density.get_mut(Bound3::new(18, 2, 2).unwrap()) = 1000.0;
    // *sim.density.get_mut(Bound3::new(18, 18, 2).unwrap()) = 10.0;
    // for _ in 0..300 {
    //     sim.step();
    // }
    let sim_res = SimulationRes(sim);

    use pan_orbit::{pan_orbit_camera, spawn_camera, PanOrbitState};
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
        .insert_resource(sim_res)
        .insert_resource(SimulationTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                pan_orbit_camera.run_if(any_with_component::<PanOrbitState>),
                step_simulation,
            ),
        )
        .run();
}
