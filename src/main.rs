use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const TRACK_RX: f32 = 34.0;
const TRACK_RZ: f32 = 22.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "APEX DRIFT".into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.055, 0.075, 0.09)))
        .add_systems(Startup, setup)
        .add_systems(Update, (drive_car, follow_camera))
        .run();
}

#[derive(Component)]
struct PlayerCar;

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct DriftState {
    speed: f32,
    yaw: f32,
    velocity: Vec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-25.0, 45.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            range: 120.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 28.0, 0.0),
    ));

    let ground_mat = materials.add(Color::srgb(0.16, 0.25, 0.20));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(150.0, 150.0))),
        MeshMaterial3d(ground_mat),
        Transform::from_xyz(0.0, -0.12, 0.0),
    ));

    let asphalt = materials.add(Color::srgb(0.105, 0.12, 0.14));
    let curb = materials.add(Color::srgb(0.92, 0.34, 0.22));
    let line = materials.add(Color::srgb(0.88, 0.87, 0.76));

    for i in 0..96 {
        let a = i as f32 / 96.0 * std::f32::consts::TAU;
        let x = TRACK_RX * a.cos();
        let z = TRACK_RZ * a.sin();
        let yaw = -a;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(13.5, 0.18, 4.8))),
            MeshMaterial3d(asphalt.clone()),
            Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(yaw)),
        ));

        let outer_x = (TRACK_RX + 8.0) * a.cos();
        let outer_z = (TRACK_RZ + 8.0) * a.sin();
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.2, 0.55, 2.0))),
            MeshMaterial3d(curb.clone()),
            Transform::from_xyz(outer_x, 0.18, outer_z).with_rotation(Quat::from_rotation_y(yaw)),
        ));

        let inner_x = (TRACK_RX - 8.0) * a.cos();
        let inner_z = (TRACK_RZ - 8.0) * a.sin();
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.6, 0.28, 1.4))),
            MeshMaterial3d(line.clone()),
            Transform::from_xyz(inner_x, 0.12, inner_z).with_rotation(Quat::from_rotation_y(yaw)),
        ));
    }

    let island_mat = materials.add(Color::srgb(0.22, 0.34, 0.25));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(13.0, 0.35))),
        MeshMaterial3d(island_mat),
        Transform::from_xyz(0.0, 0.05, 0.0),
    ));

    let body_mat = materials.add(Color::srgb(0.95, 0.28, 0.13));
    let glass_mat = materials.add(Color::srgb(0.07, 0.14, 0.19));
    let tire_mat = materials.add(Color::srgb(0.025, 0.03, 0.035));
    let wheel_mesh = meshes.add(Cylinder::new(0.42, 0.28));

    let car = commands.spawn((
        PlayerCar,
        DriftState {
            speed: 0.0,
            yaw: std::f32::consts::FRAC_PI_2,
            velocity: Vec3::ZERO,
        },
        Transform::from_xyz(TRACK_RX, 0.55, 0.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        Visibility::default(),
    )).id();

    commands.entity(car).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.1, 0.62, 4.2))),
            MeshMaterial3d(body_mat.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.65, 0.72, 1.9))),
            MeshMaterial3d(glass_mat),
            Transform::from_xyz(0.0, 0.58, -0.15),
        ));
        for x in [-1.08, 1.08] {
            for z in [-1.35, 1.35] {
                parent.spawn((
                    Mesh3d(wheel_mesh.clone()),
                    MeshMaterial3d(tire_mat.clone()),
                    Transform::from_xyz(x, -0.24, z)
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ));
            }
        }
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(TRACK_RX + 8.0, 7.0, -11.0)
            .looking_at(Vec3::new(TRACK_RX, 0.5, 0.0), Vec3::Y),
        FollowCamera,
    ));
}

fn drive_car(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut DriftState), With<PlayerCar>>,
) {
    let Ok((mut transform, mut state)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs().min(0.05);
    let (mut touch_throttle, mut touch_brake, mut touch_handbrake) = (false, false, false);
    let (mut touch_left, mut touch_right) = (false, false);
    if let Ok(window) = windows.single() {
        let w = window.width();
        let h = window.height();
        for touch in touches.iter() {
            let p = touch.position();
            if p.x < w * 0.42 {
                if p.x < w * 0.21 { touch_left = true; }
                else { touch_right = true; }
            } else if p.x > w * 0.76 && p.y > h * 0.52 {
                if p.x > w * 0.88 { touch_throttle = true; }
                else { touch_brake = true; }
            } else if p.x > w * 0.48 && p.y > h * 0.62 {
                touch_handbrake = true;
            }
        }
    }
    let throttle = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) || touch_throttle;
    let brake = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) || touch_brake;
    let left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) || touch_left;
    let right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) || touch_right;
    let handbrake = keys.pressed(KeyCode::Space) || touch_handbrake;

    if throttle { state.speed += 17.0 * dt; }
    if brake { state.speed -= 25.0 * dt; }
    if !throttle && !brake { state.speed *= 1.0 - 0.65 * dt; }
    state.speed = state.speed.clamp(-8.0, 34.0);

    let steer = (right as i8 - left as i8) as f32;
    let speed_factor = (state.speed.abs() / 10.0).clamp(0.15, 1.0);
    let drift_factor = if handbrake { 1.65 } else { 1.0 };
    state.yaw -= steer * speed_factor * drift_factor * 1.45 * dt * state.speed.signum();

    let forward = Vec3::new(state.yaw.sin(), 0.0, state.yaw.cos());
    let target_velocity = forward * state.speed;
    let grip = if handbrake { 0.035 } else { 0.12 };
    state.velocity = state.velocity.lerp(target_velocity, (grip * 60.0 * dt).clamp(0.0, 1.0));
    transform.translation += state.velocity * dt;
    transform.rotation = Quat::from_rotation_y(state.yaw);
}

fn follow_camera(
    time: Res<Time>,
    car_query: Query<&Transform, (With<PlayerCar>, Without<FollowCamera>)>,
    mut camera_query: Query<&mut Transform, (With<FollowCamera>, Without<PlayerCar>)>,
) {
    let (Ok(car), Ok(mut camera)) = (car_query.single(), camera_query.single_mut()) else {
        return;
    };

    let forward = car.rotation * Vec3::Z;
    let desired = car.translation - forward * 10.0 + Vec3::Y * 5.2;
    let look_at = car.translation + forward * 3.0 + Vec3::Y * 0.8;
    let blend = (time.delta_secs() * 4.5).clamp(0.0, 1.0);
    camera.translation = camera.translation.lerp(desired, blend);
    camera.look_at(look_at, Vec3::Y);
}
