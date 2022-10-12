//#![windows_subsystem = "windows"]

// ----------
// imports

use bevy::{prelude::*, render::texture::*, time::FixedTimestep};
use rand::{thread_rng, Rng};

// imports
// ----------
// constants

const SNAKE_SIZE: f32 = 1.27;
const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

const PADDING: f32 = 100.0;

// constants
// ---------
// components

#[derive(Component)]
struct SnakeHead {
    dir: SnakeDirection,
}

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Default, Debug, Copy, Clone)]
struct LastTailDirection(Option<DirectionPair>);

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut, Debug)]
struct SnakeSegments(Vec<Entity>);

#[derive(Component, Copy, Clone, PartialEq, Debug)]
enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
    Null,
}

#[derive(Component, Copy, Clone, PartialEq, Debug)]
struct DirectionPair(SnakeDirection, SnakeDirection);

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

#[derive(Component)]
struct Food;

struct ScoredEvent;

// components
// ---------
// systems

fn scored(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    mut score_reader: EventReader<ScoredEvent>,
    last_tail_position: Res<LastTailPosition>,
    last_tail_direction: Res<LastTailDirection>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("assets.png");
    let snake_texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 4);
    let texture_atlas_handle = texture_atlases.add(snake_texture_atlas);

    if score_reader.iter().next().is_some() {
        segments.push(spawn_segment(
            &mut commands,
            last_tail_position.0.unwrap(),
            last_tail_direction.0.unwrap(),
            texture_atlas_handle,
        ));
    }
}

fn spawn_food(commands: &mut Commands) {
    let mut rng = thread_rng();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(20.0, 20.0, 20.0),
                ..default()
            },
            ..default()
        })
        .insert(Position {
            x: rng.gen_range(0..(GRID_WIDTH - 1)),
            y: rng.gen_range(0..(GRID_HEIGHT - 1)),
        })
        .insert(Food);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle { ..default() });
}

fn spawn_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("assets.png");
    let snake_texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 4);
    let texture_atlas_handle = texture_atlases.add(snake_texture_atlas);

    *segments = SnakeSegments(vec![
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..default()
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform {
                    scale: Vec3::from_array([SNAKE_SIZE; 3]),
                    ..default()
                },
                ..default()
            })
            .insert(SnakeHead {
                dir: SnakeDirection::Null,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(DirectionPair(SnakeDirection::Null, SnakeDirection::Null))
            .id(),
        spawn_segment(
            &mut commands,
            Position { x: 3, y: 2 },
            DirectionPair(SnakeDirection::Null, SnakeDirection::Null),
            texture_atlas_handle.clone(),
        ),
        spawn_segment(
            &mut commands,
            Position { x: 3, y: 1 },
            DirectionPair(SnakeDirection::Null, SnakeDirection::Null),
            texture_atlas_handle,
        ),
    ]);
}

fn snake_controls(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<&mut SnakeHead>,
    mut head_directions: Query<&mut DirectionPair, With<SnakeHead>>,
    prev_directions: Query<&DirectionPair, Without<SnakeHead>>
) {
    use SnakeDirection::*;

    for ((mut facing, mut direction), prev_directions) in

    head_positions.iter_mut()
    .zip(head_directions.iter_mut())
    .zip(prev_directions.iter()) {
        
        if keyboard_input.pressed(KeyCode::Left) && facing.dir != Right {
            direction.0 = prev_directions.1;
            facing.dir = Left;
            direction.1 = facing.dir;
        } else if keyboard_input.pressed(KeyCode::Right) && facing.dir != Left {
            direction.0 = prev_directions.1;
            facing.dir = Right;
            direction.1 = facing.dir;
        } else if keyboard_input.pressed(KeyCode::Down) && facing.dir != Up {
            direction.0 = prev_directions.1;
            facing.dir = Down;
            direction.1 = facing.dir;
        } else if keyboard_input.pressed(KeyCode::Up) && facing.dir != Down {
            direction.0 = prev_directions.1;
            facing.dir = Up;
            direction.1 = facing.dir;
        }
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut directions: Query<&mut DirectionPair>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut last_tail_direction: ResMut<LastTailDirection>,
) {
    use SnakeDirection::*;

    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut pos = positions.get_mut(head_entity).unwrap();

        match &head.dir {
            Left if pos.x > 0 => pos.x -= 1,
            Right if pos.x < GRID_WIDTH - 1 => pos.x += 1,
            Up if pos.y < GRID_WIDTH - 1 => pos.y += 1,
            Down if pos.y > 0 => pos.y -= 1,
            _ => return,
        }

        segment_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));

        let segment_directions = segments
            .iter()
            .map(|e| *directions.get_mut(*e).unwrap())
            .collect::<Vec<DirectionPair>>();

        segment_directions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(dir, segment)| {
                directions.get_mut(*segment).unwrap().0 = dir.0;
            });

        segment_directions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(dir, segment)| {
                directions.get_mut(*segment).unwrap().1 = dir.1;
            });

        //println!("{:#?}", segment_directions);

        *last_tail_direction = LastTailDirection(Some(*segment_directions.last().unwrap()));
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / GRID_WIDTH as f32 * (window.height() as f32 - PADDING),
            sprite_size.height / GRID_HEIGHT as f32 * (window.height() as f32 - PADDING),
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
            convert(
                pos.x as f32,
                window.height() as f32 - PADDING,
                GRID_WIDTH as f32,
            ),
            convert(
                pos.y as f32,
                window.height() as f32 - PADDING,
                GRID_HEIGHT as f32,
            ),
            10.0,
        );
    }
}

fn collision_detection(
    mut commands: Commands,
    mut score_writer: EventWriter<ScoredEvent>,
    snake: Query<&Position, With<SnakeHead>>,
    food: Query<(Entity, &Position), With<Food>>,
) {
    for snake_pos in snake.iter() {
        for (ent, food_pos) in food.iter() {
            if snake_pos == food_pos {
                commands.entity(ent).despawn();
                spawn_food(&mut commands);
                score_writer.send(ScoredEvent);
            }
        }
    }
}

fn setup_board(mut commands: Commands) {
    for x in 0..(GRID_WIDTH) {
        if x % 2 == 0 {
            for y in (0..(GRID_HEIGHT - 1)).step_by(2) {
                draw_bg_element(x, y, 1.0, 1.0, (0.027, 0.212, 0.259), &mut commands)
            }
        } else {
            for y in (1..(GRID_HEIGHT)).step_by(2) {
                draw_bg_element(x, y, 1.0, 1.0, (0.027, 0.212, 0.259), &mut commands)
            }
        }
    }
}

fn setup_outline(mut commands: Commands) {
    const OUTLINE_COLOR: (f32, f32, f32) = (0.345, 0.431, 0.459);

    for y in 0..GRID_HEIGHT {
        draw_bg_element(-1, y, 1.0, 0.5, OUTLINE_COLOR, &mut commands);
        draw_bg_element(GRID_HEIGHT, y, 1.0, 0.5, OUTLINE_COLOR, &mut commands);
    }

    for x in 0..GRID_HEIGHT {
        draw_bg_element(x, -1, 0.5, 1.0, OUTLINE_COLOR, &mut commands);
        draw_bg_element(x, GRID_WIDTH, 0.5, 1.0, OUTLINE_COLOR, &mut commands);
    }
}

fn draw_bg_element(
    x: i32,
    y: i32,
    h: f32,
    w: f32,
    color: (f32, f32, f32),
    commands: &mut Commands,
) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(color.0, color.1, color.2),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(20.0 * w, 20.0 * h, 0.1),
                ..default()
            },
            ..default()
        })
        .insert(Position { x, y });
}

fn spawn_segment(
    commands: &mut Commands,
    pos: Position,
    dir: DirectionPair,
    texture_atlas_handle: Handle<TextureAtlas>,
) -> Entity {
    use SnakeDirection::*;
    let index = match dir.1 {
        Up | Down => 2,
        Left | Right => 1,
        Null => 2,
    };

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite { index, ..default() },
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                scale: Vec3::from_array([SNAKE_SIZE; 3]),
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(dir)
        .insert(pos)
        .id()
}

fn update_textures(
    mut query: Query<(&mut TextureAtlasSprite, &DirectionPair), (With<SnakeSegment>, Without<SnakeHead>)>,
) {
    use SnakeDirection::*;

    for (mut sprite, snake_direction) in query.iter_mut() {
        let index = match (snake_direction.0, snake_direction.1) {
            (Left, Up) | (Down, Right) => 5,

            (Right, Up) | (Down, Left) => 4,

            (Left, Down) | (Up, Right) => 9,

            (Right, Down) | (Up, Left) => 8,

            (Up, Up) | (Down, Down) => 2,

            (Left, Left) | (Right, Right) => 1,

            _ => 2,
        };

        sprite.index = index
    }
}

// systems2
// ---------
// mai

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "snek".to_string(),
            width: 500.0,
            height: 500.0,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.169, 0.212)))
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_startup_system(|mut commands: Commands| spawn_food(&mut commands))
        .add_event::<ScoredEvent>()
        //.add_startup_system(setup_board)
        .add_startup_system(setup_outline)
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .insert_resource(LastTailDirection::default())
        .add_system(snake_controls.before(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(snake_movement)
                .with_system(collision_detection.after(snake_movement))
                .with_system(scored.after(collision_detection))
                .with_system(update_textures.after(snake_movement)),
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