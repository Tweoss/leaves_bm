mod egui;
mod keyboard;
mod pan_orbit;
mod render;

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_egui::PrimaryEguiContext;
use bevy_render::view::RenderLayers;
use leaves_bm::{
    lbm::{Constants, Initializer, Simulation},
    Bound3,
};
use rand::{rngs::SmallRng, SeedableRng};

use crate::{
    egui::{ColorBounds, Function, InitParams, SimControls, UiState},
    keyboard::handle_keystrokes,
    pan_orbit::spawn_camera,
    render::{CustomMaterialPlugin, InstanceData, InstanceMaterialData},
};

const X_COUNT: usize = 40;
const Y_COUNT: usize = 40;
const Z_COUNT: usize = 3;
const PARTICLE_COUNT: usize = 50;
const RNG_SEED: u64 = 0xDEADBEEF;

/// initialize 3d scene objects
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
    let bundle = (
        Mesh3d(meshes.add(Sphere::new(0.3))),
        render::ParticlePoint,
        InstanceMaterialData(
            (0..PARTICLE_COUNT)
                .map(|_| InstanceData {
                    position: Vec3::ZERO,
                    scale: 1.0,
                    color: LinearRgba::new(0.6, 0.2, 0.2, 1.0).to_f32_array(),
                })
                .collect(),
        ),
    );
    commands.spawn(bundle);

    // instanced boxes
    let bundle = (
        Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.3))),
        render::GridPoint,
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
        NoFrustumCulling,
    );
    commands.spawn(bundle);

    // egui camera (the whole window)
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

    // pan orbit camera (in a panel)
    spawn_camera(commands);
}

mod init {
    use leaves_bm::{
        lbm::{InitArgs, Particle},
        math::{Int3, Vec3},
    };
    use rand::Rng;

    use crate::{PARTICLE_COUNT, X_COUNT, Y_COUNT, Z_COUNT};

    pub fn wave(
        InitArgs {
            loc: (x, _, _),
            dir,
            ..
        }: InitArgs,
    ) -> Option<f32> {
        let vec: Vec3 = dir.into();
        if x != 0 && x + 1 != X_COUNT {
            return None;
        }
        let x_f = x as f32 - (X_COUNT as f32 / 2.0);
        let magnitude = vec.dot(Vec3::new(if x_f > 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0)) / 20.0;
        (magnitude > 0.0).then_some(magnitude)
    }
    pub fn circular(
        InitArgs {
            loc: (x, y, z),
            dir,
            weight,
        }: InitArgs,
    ) -> Option<f32> {
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
    }
    pub fn point(
        InitArgs {
            loc: (x, y, z),
            dir,
            ..
        }: InitArgs,
    ) -> Option<f32> {
        if x != X_COUNT / 2 || y != Y_COUNT / 2 || z != Z_COUNT / 2 {
            return None;
        }
        if dir == Int3::ZERO {
            Some(70.0)
        } else {
            None
        }
    }

    pub fn particles<T: Rng>(rng: &mut T) -> Vec<Particle<X_COUNT, Y_COUNT, Z_COUNT>> {
        (0..PARTICLE_COUNT)
            .map(|_| Particle::from_rng_bounds(rng))
            .collect()
    }
}

#[allow(clippy::too_many_arguments)]
fn step_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut sim: ResMut<SimulationRes>,
    mut controls: ResMut<SimControls>,
    params: Res<InitParams>,
    grid: Single<(&mut InstanceMaterialData, &render::GridPoint)>,
    particles: Single<
        (&mut InstanceMaterialData, &render::ParticlePoint),
        Without<render::GridPoint>,
    >,
    color_bounds: Res<ColorBounds>,
    constants: Res<egui::Constants>,
) {
    let init_func: Initializer = match params.function {
        Function::Wave => Box::new(init::wave),
        Function::Circle => Box::new(init::circular),
        Function::Point => Box::new(init::point),
    };
    let mut rerender = false;

    sim.0.constants = constants.into_inner().clone().into();

    if controls.restart_requested {
        rerender = true;
        controls.restart_requested = false;

        let mut rng = SmallRng::seed_from_u64(RNG_SEED);
        let mut new_sim = Simulation::new(sim.0.constants, init::particles(&mut rng));
        new_sim.initialize(init_func);

        *sim = SimulationRes(new_sim);
    }

    if !controls.paused && timer.0.tick(time.delta()).just_finished() {
        rerender = true;

        use std::time::Instant;
        let start = Instant::now();
        sim.0.step();
        dbg!(Instant::now().duration_since(start));
        if controls.single_step {
            controls.paused = true;
            controls.single_step = false;
        }
    }

    rerender |= color_bounds.is_changed();

    if rerender {
        let (mut grid, _) = grid.into_inner();
        #[allow(clippy::modulo_one)]
        for (i, data) in &mut grid.0.iter_mut().enumerate() {
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
        let (mut particles, _) = particles.into_inner();
        let iter_mut = particles.0.iter_mut();
        iter_mut
            .zip(sim.0.particles.iter())
            .for_each(|(data, particle)| {
                data.position.x = X_COUNT as f32 / 2.0 - particle.position.x;
                data.position.y = Y_COUNT as f32 / 2.0 - particle.position.y;
                data.position.z = Z_COUNT as f32 / 2.0 - particle.position.z;
            });
    }
}

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct SimulationRes(Simulation<X_COUNT, Y_COUNT, Z_COUNT>);

fn main() {
    let mut rng = SmallRng::seed_from_u64(RNG_SEED);
    let sim = Simulation::new(
        Constants {
            // Should be greater than 1 for some reason.
            time_relaxation_constant: 1.25,
            speed_of_sound: 1.0 / (3.0_f32).sqrt(),
            particle_mass: 0.15,
            particle_velocity_decay: 0.2,
        },
        init::particles(&mut rng),
    );

    use pan_orbit::{pan_orbit_camera, PanOrbitState};
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
        .insert_resource(egui::Constants::from(sim.constants))
        .insert_resource(SimulationTimer(Timer::new(
            Duration::from_millis(10),
            TimerMode::Repeating,
        )))
        .insert_resource(UiState::new())
        .insert_resource(SimulationRes(sim))
        .add_systems(Startup, setup)
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
