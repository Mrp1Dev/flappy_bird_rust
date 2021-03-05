use bevy::{
    app::startup_stage,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::collide,
    window::{self, WindowMode},
};
use rand::Rng;
use std::{ops::Range, time::Duration};

fn main() {
    static STATE: &str = "state";
    App::build()
        .add_resource(WindowDescriptor {
            title: "Flappy Bird".to_string(),
            width: 1920.0 * 0.7,
            height: 1080.0 * 0.7,
            vsync: true,
            resizable: true,
            mode: WindowMode::Windowed {},
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_resource(State::new(GameState::Started))
        .add_resource(ClearColor(ColorType::Bg.get_color()))
        .add_resource(LastPillarX(0.0))
        .add_resource(GameState::Started)
        .add_stage_before(stage::UPDATE, STATE, StateStage::<GameState>::default())
        //Startup Systems
        .add_startup_system(spawn_roof_floor.system())
        .add_startup_system(spawn_scoreboard.system())
        //Stateless systems:
        .add_system(velocity_system.system())
        .add_system(lifetime_system.system())
        .add_system(update_borders.system())
        .add_system(destroy_out_of_bounds.system())
        //Systems for started state
        .on_state_enter(STATE, GameState::Started, spawn_bird.system())
        .on_state_update(STATE, GameState::Started, game_run_check_system.system())
        //Systems for Running State
        .on_state_update(STATE, GameState::Running, flap_system.system())
        .on_state_update(STATE, GameState::Running, gravity_system.system())
        .on_state_update(STATE, GameState::Running, pillar_spawning_system.system())
        .on_state_update(STATE, GameState::Running, pillar_collision_system.system())
        .on_state_update(STATE, GameState::Running, explosion_system.system())
        .on_state_update(STATE, GameState::Running, score_collision_system.system())
        .on_state_update(STATE, GameState::Running, highscore_system.system())
        .on_state_update(STATE, GameState::Running, fade_out_system.system())
        .on_state_update(STATE, GameState::Running, update_scoreboard_system.system())
        //Systems for Over State
        .on_state_update(STATE, GameState::Over, restart_system.system())
        .run();
}

#[derive(Clone)]
enum GameState {
    Running,
    Started,
    Over,
}

enum ColorType {
    Pillar,
    Bird,
    Bg,
    Laser,
    Score,
}

//hardcode color values here!
impl ColorType {
    fn get_color(&self) -> Color {
        match self {
            ColorType::Pillar => Color::rgb_u8(255, 180, 84),
            ColorType::Bird => Color::rgb_u8(89, 194, 255),
            ColorType::Bg => Color::rgb_u8(13, 16, 22),
            ColorType::Laser => Color::rgb_u8(240, 46, 46),
            ColorType::Score => Color::rgb_u8(149, 230, 203),
        }
    }
}

struct Velocity(Vec3);
struct Gravity(f32);
struct Bird {
    flap_height: f32,
}

struct Explodes {
    explode: bool,
    particle_color: Color,
    particle_count: u32,
    particle_speed_range: Range<f32>,
    particle_lifetime_range: Range<f32>,
}
struct Lifetime(f32);
struct Collider;
struct Border;
struct Score(u64);
struct Highscore(u64);
struct FadeOut {
    start: bool,
    speed: f32,
}

struct ScoreTrigger {
    scored: bool,
}

struct DestroyAtRestart;
struct LastPillarX(f64);

fn spawn_bird(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        .spawn(SpriteBundle {
            material: materials.add(ColorType::Bird.get_color().into()),
            transform: Transform::from_translation(Vec3 {
                x: -275.0,
                y: 0.0,
                z: 1.0,
            }),
            sprite: Sprite::new(Vec2 { x: 27.0, y: 27.0 }),
            ..Default::default()
        })
        .with(Velocity(Vec3::zero()))
        .with(Collider)
        .with(Gravity(1000.0))
        .with(Bird { flap_height: 75.0 })
        .with(DestroyAtRestart)
        .with(Explodes {
            explode: false,
            particle_color: ColorType::Bird.get_color(),
            particle_count: 32,
            particle_speed_range: 300.0..850.0,
            particle_lifetime_range: 0.1..1.0,
        });
}

fn spawn_roof_floor(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    for _ in 0..2 {
        commands
            .spawn(SpriteBundle {
                material: materials.add(ColorType::Pillar.get_color().into()),
                transform: Transform::default(),
                sprite: Sprite::new(Vec2::zero()),
                ..Default::default()
            })
            .with(Collider)
            .with(Border);
    }
}

fn spawn_scoreboard(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text {
                        font: asset_server.load("Roboto-Thin.ttf"),
                        value: "0".to_string(),
                        style: TextStyle {
                            color: ColorType::Score.get_color(),
                            font_size: 90.0,
                            alignment: TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                vertical: VerticalAlign::Top,
                            },
                        },
                        ..Default::default()
                    },
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        position: Rect {
                            top: Val::Px(47.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Score(0));
            parent
                .spawn(TextBundle {
                    text: Text {
                        font: asset_server.load("Roboto-Black.ttf"),
                        value: "0".to_string(),
                        style: TextStyle {
                            color: ColorType::Score.get_color(),
                            font_size: 40.0,
                            alignment: TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                vertical: VerticalAlign::Top,
                            },
                        },
                        ..Default::default()
                    },
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        position: Rect {
                            top: Val::Px(100.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Highscore(0));
        });
}

fn game_run_check_system(mut state: ResMut<State<GameState>>, input: Res<Input<KeyCode>>) {
    if input.pressed(KeyCode::Space) {
        state.set_next(GameState::Running).unwrap();
    }
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

fn flap_system(keyboard: Res<Input<KeyCode>>, mut query: Query<(&Bird, &Gravity, &mut Velocity)>) {
    for (bird, gravity, mut velocity) in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::Space) {
            velocity.0.y = (2.0 * gravity.0 * bird.flap_height).sqrt();
        }
    }
}

fn pillar_spawning_system(
    windows: Res<Windows>,
    time: Res<Time>,
    mut last_pillar_x: ResMut<LastPillarX>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const PILLAR_GAP: f32 = 155.0;
    const PILLAR_WIDTH: f32 = 60.0;
    const PILLAR_SPAWN_DIST: f64 = 380.0;
    let window = windows.get_primary().expect("Couldn't get a window!");
    let pillar_speed = 350.0;
    if window.width() as f64 - last_pillar_x.0 > PILLAR_SPAWN_DIST {
        let rand_screen_percent = rand::thread_rng().gen_range(0.2..0.8);
        let x_pos = window.width() / 2.0 + PILLAR_WIDTH / 2.0;
        for i in [-1, 1].iter() {
            let mut pillar_height = window.height();
            pillar_height *= if *i == -1 {
                1.0 - rand_screen_percent
            } else {
                rand_screen_percent
            };
            let y_pos =
                (*i as f32) * ((pillar_height / 2.0 - window.height() / 2.0) - PILLAR_GAP / 2.0);
            commands
                .spawn(SpriteBundle {
                    material: materials.add(ColorType::Pillar.get_color().into()),
                    transform: Transform::from_translation(Vec3 {
                        x: x_pos,
                        y: y_pos,
                        z: 0.0,
                    }),
                    sprite: Sprite::new(Vec2 {
                        x: PILLAR_WIDTH,
                        y: pillar_height,
                    }),
                    ..Default::default()
                })
                .with(Velocity(Vec3 {
                    x: -pillar_speed,
                    y: 0.0,
                    z: 0.0,
                }))
                .with(Collider)
                .with(DestroyAtRestart);
        }
        //score Triggers:
        let y_pos = window.height() * (rand_screen_percent - 0.5);
        commands
            .spawn(SpriteBundle {
                material: materials.add(ColorType::Laser.get_color().into()),
                transform: Transform::from_translation(Vec3 {
                    x: x_pos,
                    y: y_pos,
                    z: 0.0,
                }),
                sprite: Sprite::new(Vec2 {
                    x: 5.0,
                    y: PILLAR_GAP,
                }),
                ..Default::default()
            })
            .with(Velocity(Vec3 {
                x: -pillar_speed,
                y: 0.0,
                z: 0.0,
            }))
            .with(ScoreTrigger { scored: false })
            .with(DestroyAtRestart)
            .with(FadeOut {
                start: false,
                speed: 3.0,
            });
        last_pillar_x.0 = (x_pos + window.width() / 2.0).into();
    }
    last_pillar_x.0 -= (pillar_speed * time.delta_seconds()) as f64;
}

fn pillar_collision_system(
    mut state: ResMut<State<GameState>>,

    mut bird_query: Query<(&Transform, &Sprite, &mut Explodes), (With<Collider>, With<Bird>)>,
    pillar_query: Query<(&Transform, &Sprite), (Without<Bird>, With<Collider>)>,
) {
    for (bird_transform, bird_sprite, mut explodes) in bird_query.iter_mut() {
        let bird_size = bird_sprite.size;
        let bird_pos = bird_transform.translation;
        for (collider, collider_sprite) in pillar_query.iter() {
            let collision = collide(
                bird_pos,
                bird_size * 0.95,
                collider.translation,
                collider_sprite.size,
            );
            if let Some(_) = collision {
                explodes.explode = true;
                state.set_next(GameState::Over).unwrap();
            }
        }
    }
}

fn score_collision_system(
    bird_query: Query<(&Transform, &Sprite), (With<Collider>, With<Bird>)>,
    mut score_trigger_query: Query<(&mut ScoreTrigger, &mut FadeOut, &Transform, &Sprite)>,
    mut scores: Query<&mut Score>,
) {
    for (bird_transform, bird_sprite, ..) in bird_query.iter() {
        let bird_size = bird_sprite.size;
        let bird_pos = bird_transform.translation;
        for (mut score_trigger, mut fade_out, collider, collider_sprite) in
            score_trigger_query.iter_mut()
        {
            let collision = collide(
                bird_pos,
                bird_size,
                collider.translation,
                collider_sprite.size,
            );
            if let Some(_) = collision {
                if !score_trigger.scored {
                    score_trigger.scored = true;
                    for mut score in scores.iter_mut() {
                        score.0 += 1;
                    }
                    fade_out.start = true;
                }
            }
        }
    }
}

fn fade_out_system(
    query: Query<(&Handle<ColorMaterial>, &FadeOut)>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (color, fade_out) in query.iter() {
        if fade_out.start {
            let color_material = materials.get_mut(color).unwrap();
            let alpha = color_material.color.a();
            color_material
                .color
                .set_a(alpha - time.delta_seconds() * fade_out.speed);
        }
    }
}

fn update_scoreboard_system(
    mut scoreboard_query: Query<(&mut Text, &Score)>,
    mut highscoreboard_query: Query<(&mut Text, &Highscore)>,
) {
    for (mut text, score) in scoreboard_query.iter_mut() {
        text.value = score.0.to_string();
    }

    for (mut text, highscore) in highscoreboard_query.iter_mut() {
        text.value = highscore.0.to_string();
    }
}

fn restart_system(
    commands: &mut Commands,
    keyboard: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    query: Query<Entity, With<DestroyAtRestart>>,
    mut scores: Query<&mut Score>,
) {
    if keyboard.pressed(KeyCode::Space) {
        for entity in query.iter() {
            commands.despawn(entity);
        }
        for mut score in scores.iter_mut() {
            score.0 = 0;
        }
        state.set_next(GameState::Started).unwrap();
    }
}

fn explosion_system(
    query: Query<(&Explodes, &Sprite, &Transform, Entity)>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (explodes, sprite, transform, entity) in query.iter() {
        if explodes.explode {
            for _ in 0..explodes.particle_count {
                let random_size = rand::thread_rng().gen_range(0.05..0.5) * sprite.size;
                let random_direction = {
                    let rand_x = rand::thread_rng().gen_range(-1.0..1.0);
                    let rand_y = rand::thread_rng().gen_range(-1.0..1.0);
                    Vec3 {
                        x: rand_x,
                        y: rand_y,
                        z: 0.0,
                    }
                    .normalize()
                };
                let random_velocity = rand::thread_rng()
                    .gen_range(explodes.particle_speed_range.clone())
                    * random_direction;
                let particle_lifetime =
                    rand::thread_rng().gen_range(explodes.particle_lifetime_range.clone());
                commands
                    .spawn(SpriteBundle {
                        material: materials.add(explodes.particle_color.into()),
                        transform: Transform::from_translation(transform.translation),
                        sprite: Sprite::new(random_size),
                        ..Default::default()
                    })
                    .with(Velocity(random_velocity))
                    .with(Lifetime(particle_lifetime));
                commands.despawn(entity);
            }
        }
    }
}
fn lifetime_system(
    commands: &mut Commands,
    mut query: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        if lifetime.0 <= 0.0 {
            commands.despawn(entity);
        } else {
            lifetime.0 -= time.delta_seconds();
        }
    }
}

fn update_borders(
    mut query: Query<(&mut Transform, &mut Sprite), With<Border>>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().expect("Couldn't get window.");
    let mut i = -1;
    for (mut transform, mut sprite) in query.iter_mut() {
        let y_pos = i as f32 * (window.height() / 2.0);
        transform.translation.y = y_pos;
        sprite.size = Vec2 {
            x: window.width(),
            y: 40.0,
        };
        i = 1;
    }
}

fn highscore_system(scores: Query<&Score>, mut highscores: Query<&mut Highscore>) {
    for mut highscore in highscores.iter_mut() {
        for score in scores.iter() {
            if score.0 > highscore.0 {
                highscore.0 = score.0;
            }
        }
    }
}

fn destroy_out_of_bounds(
    query: Query<(Entity, &Transform, &Sprite)>,
    commands: &mut Commands,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().expect("couldn't get window.");
    for (entity, transform, sprite) in query.iter() {
        if transform.translation.x + sprite.size.x < -window.width() / 2.0 {
            commands.despawn(entity);
        }
    }
}
