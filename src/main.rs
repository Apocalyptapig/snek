use bevy::core::FixedTimestep;
use bevy::{prelude::*, window::PresentMode};

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;

#[derive(Component)]
struct SnakeHead {
    dir: SnakeDirection,
}

#[derive(PartialEq)]
enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
    Null,
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            // <--
            title: "snek".to_string(), // <--
            width: 500.0,              // <--
            height: 500.0,             // <--
            ..default()                // <--
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(setup_camera)
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_system(snake_controls.before(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(snake_movement),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_snake(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(25.0, 25.0, 25.0),
                ..default()
            },
            ..default()
        })
        .insert(SnakeHead {
            dir: SnakeDirection::Null,
        })
        .insert(Position { x: 3, y: 3 });
}

fn snake_controls(keyboard_input: Res<Input<KeyCode>>, mut head_positions: Query<&mut SnakeHead>) {
    use SnakeDirection::*;

    for mut facing in head_positions.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) && facing.dir != Right {
            facing.dir = Left
        } else if keyboard_input.pressed(KeyCode::Right) && facing.dir != Left {
            facing.dir = Right
        } else if keyboard_input.pressed(KeyCode::Down) && facing.dir != Up {
            facing.dir = Down
        } else if keyboard_input.pressed(KeyCode::Up) && facing.dir != Down {
            facing.dir = Up
        }
    }
}

fn snake_movement(mut head_positions: Query<(&mut Position, &mut SnakeHead)>) {
    use SnakeDirection::*;

    for (mut pos, facing) in head_positions.iter_mut() {
        match &facing.dir {
            Left if pos.x > 0 => pos.x -= 1,
            Right if pos.x < GRID_WIDTH - 1 => pos.x += 1,
            Up if pos.y < GRID_WIDTH - 1 => pos.y += 1,
            Down if pos.y > 0 => pos.y -= 1,
            _ => return,
        }

        //println!("{:?}", pos)
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / GRID_WIDTH as f32 * window.width() as f32,
            sprite_size.height / GRID_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, GRID_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, GRID_HEIGHT as f32),
            0.0,
        );
    }
}
