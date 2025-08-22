mod pan_orbit;

use std::time::Duration;

use bevy::prelude::*;
use leaves_bm::{Bound3, Constants, Simulation};

#[derive(Component)]
struct Cube {
    x: i32,
    y: i32,
    z: i32,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    //
    let x_count = 20;
    let y_count = 20;
    let z_count = 20;
    let box_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.2));
    let boxes: Vec<_> = (0..x_count)
        .flat_map(|x| (0..y_count).map(move |y| (x, y)))
        .flat_map(|(x, y)| (0..z_count).map(move |z| (x, y, z)))
        .map(|(x, y, z)| {
            (
                Cube { x, y, z },
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                Transform::from_xyz(
                    x_count as f32 / 2.0 - x as f32,
                    y_count as f32 / 2.0 - y as f32,
                    z_count as f32 / 2.0 - z as f32,
                ),
            )
        })
        .collect();
    commands.spawn_batch(boxes);
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut handles: Query<(&Cube, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        use std::time::Instant;
        let start = Instant::now();
        // let v1 = *sim.0.density.get(Bound3::new(2, 2, 2).unwrap());
        sim.0.step();
        // let w2 = sim.0.density.get(Bound3::new(18, 18, 2).unwrap());
        dbg!(Instant::now().duration_since(start));
        // dbg!(Instant::now().duration_since(start), v1, w2);

        let material_start = Instant::now();
        for (cube, material) in handles.iter_mut() {
            let material = materials.get_mut(material.id()).unwrap();
            let color = material.base_color;
            let mut color = color.to_linear();
            let value = *sim
                .0
                .density
                .get(Bound3::new(cube.x as usize, cube.y as usize, cube.z as usize).unwrap())
                - 1.0;
            // if cube.x == 2 && cube.y == 2 {
            //     dbg!(value);
            // }
            color.red = value;
            color.set_alpha(value);
            material.base_color = color.into();
        }
        dbg!(Instant::now().duration_since(material_start));
    }
}

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct SimulationRes(Simulation<20, 20, 20>);

fn main() {
    let mut sim = Simulation::new(Constants {
        time_relaxation_constant: 0.1,
        speed_of_sound: 1.0 / (3.0_f32).sqrt(),
    });
    for i in 0..20 {
        for j in 0..20 {
            *sim.density.get_mut(Bound3::new(i, j, 2).unwrap()) = 10.0;
        }
    }
    // *sim.density.get_mut(Bound3::new(18, 2, 2).unwrap()) = 1000.0;
    // *sim.density.get_mut(Bound3::new(18, 18, 2).unwrap()) = 10.0;
    // for _ in 0..300 {
    //     sim.step();
    // }
    let sim_res = SimulationRes(sim);

    // return;
    use pan_orbit::{pan_orbit_camera, spawn_camera, PanOrbitState};
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BLM Simulator".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(sim_res)
        .insert_resource(SimulationTimer(Timer::new(
            Duration::from_millis(1000),
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
