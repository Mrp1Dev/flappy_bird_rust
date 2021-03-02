use bevy::{prelude::*, render::camera::OrthographicProjection, transform};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(gravity_system.system())
        .add_system(velocity_system.system())
        .run();
}

struct Velocity(Vec3);
struct Gravity(f32);
struct Bird;

enum Collider {
    Bird,
    Pillar,
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.58, 0.0).into()),
            transform: Transform::from_translation(Vec3 {
                x: -250.0,
                y: 0.0,
                z: 0.0,
            }),
            sprite: Sprite::new(Vec2 { x: 25.0, y: 25.0 }),
            ..Default::default()
        })
        .with(Velocity(Vec3::zero()))
        .with(Collider::Bird)
        .with(Gravity(125.0));
}

fn velocity_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (vel, mut transform) in query.iter_mut() {
        let translation = &mut transform.translation;
        *translation += time.delta_seconds() * vel.0;
    }
}

fn gravity_system(time: Res<Time>, mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut vel, gravity) in query.iter_mut() {
        vel.0.y -= gravity.0 * time.delta_seconds();
    }
}
