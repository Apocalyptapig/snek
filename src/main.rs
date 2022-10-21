//#![windows_subsystem = "windows"]

use bevy::{ecs::schedule::ShouldRun, prelude::*, render::texture::*};
use rand::{thread_rng, Rng};
use std::time::Duration;

const SNAKE_SIZE: f32 = 1.27;
const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);
const PADDING: f32 = 100.0;
#[derive(Component)]
struct SnakeHead {
    dir: SnakeDirection,
}

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Default, Debug, Copy, Clone)]
struct LastTailDirection(Option<DirectionPair>);

#[derive(Default)]
struct Score(u128);

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut, Debug)]
struct SnakeSegments(Vec<Entity>);

#[derive(Component)]
struct ScoreText;

// some assembly required
#[derive(Component, Copy, Clone, PartialEq, Debug)]
enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
    Null,
}

// proud of this type; pair.0 is the previous / entry direction,
// pair.1 is the inputted / exit direction

/*
dirpair:       snake:

input: left
_________
|       |       O o o <
< l     |           o
|___^u__|           o
 prev: up

*/

#[derive(Component, Copy, Clone, PartialEq, Debug)]
struct DirectionPair(SnakeDirection, SnakeDirection);

// bring-your-own-grid day
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
#[derive(Default, Debug, Clone)]
struct SpriteSheet(Handle<TextureAtlas>);

fn make_atlas(
    mut sprite_sheet: ResMut<SpriteSheet>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("assets.png");
    let snake_texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(16.0, 16.0),
        4,
        4,
        Vec2::new(3.0, 3.0),
        Vec2::new(1.0, 1.0),
    );
    sprite_sheet.0 = texture_atlases.add(snake_texture_atlas);
}

fn scored(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    mut score_reader: EventReader<ScoredEvent>,
    last_tail_position: Res<LastTailPosition>,
    last_tail_direction: Res<LastTailDirection>,
    mut score: ResMut<Score>,
    sprite_sheet: Res<SpriteSheet>,
) {
    if score_reader.iter().next().is_some() {
        score.0 += 1;

        segments.push(spawn_segment(
            &mut commands,
            last_tail_position.0.unwrap(),
            last_tail_direction.0.unwrap(),
            &sprite_sheet.0,
        ));
    }
}

struct FoodHelperEvent;

fn spawn_food_helper(mut writer: EventWriter<FoodHelperEvent>) {
    writer.send(FoodHelperEvent);
}

fn spawn_food(
    mut commands: Commands,
    mut helper_reader: EventReader<FoodHelperEvent>,
    mut score_reader: EventReader<ScoredEvent>,
    mut positions: Query<&mut Position>,
    segments: ResMut<SnakeSegments>,
) {
    if helper_reader.iter().next().is_some() || score_reader.iter().next().is_some() {
        let mut rng = thread_rng();

        let segment_positions = segments
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut pos = Position {
            x: rng.gen_range(0..GRID_WIDTH),
            y: rng.gen_range(0..GRID_WIDTH),
        };

        while segment_positions.contains(&pos) {
            pos = Position {
                x: rng.gen_range(0..GRID_WIDTH),
                y: rng.gen_range(0..GRID_WIDTH),
            };
        }

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
            .insert(pos)
            .insert(Food);
    }
}

// init snake, camera, textures
fn spawn_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    sprite_sheet: Res<SpriteSheet>,
) {
    commands.spawn_bundle(Camera2dBundle { ..default() });

    *segments = SnakeSegments(vec![
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..default()
                },
                texture_atlas: sprite_sheet.0.clone(),
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
            &sprite_sheet.0.clone(),
        ),
        spawn_segment(
            &mut commands,
            Position { x: 3, y: 1 },
            DirectionPair(SnakeDirection::Null, SnakeDirection::Null),
            &sprite_sheet.0,
        ),
    ]);
}

fn snake_controls(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<&mut SnakeHead>,
    mut head_directions: Query<&mut DirectionPair, With<SnakeHead>>,
    prev_directions: Query<&DirectionPair, Without<SnakeHead>>,
) {
    use SnakeDirection::*;

    for ((mut facing, mut direction), prev_directions) in
        //  |   3 damn days for like 5 lines
        //  V   it made corners work but ugh
        head_positions
            .iter_mut()
            .zip(head_directions.iter_mut())
            .zip(prev_directions.iter())
    {
        // what's all this mess? that's right, it's a bad fix for a weird problem.
        // on seemingly random occasions dirpairs would replicate themselves
        // until the line was all corner sprites

        // the fix is to use some logic to check "hey, is this corner gonna look weird?"
        // it would be nicer to actually fix the replication but it's a very bizarre bug
        // nothing more permanent than a temporary solution

        if keyboard_input.pressed(KeyCode::Left) && facing.dir != Right
            || keyboard_input.pressed(KeyCode::A) && facing.dir != Right
        {
            facing.dir = Left;

            // if it looks bad, assume it's a straight line
            if direction.0 != direction.1 && *direction == *prev_directions {
                direction.0 = facing.dir;
                direction.1 = facing.dir;
            } else {
                // if it looks good, promote old dir.1
                direction.0 = prev_directions.1;
                direction.1 = facing.dir;
            }
        } else if keyboard_input.pressed(KeyCode::Right) && facing.dir != Left
            || keyboard_input.pressed(KeyCode::D) && facing.dir != Left
        {
            facing.dir = Right;
            if direction.0 != direction.1 && *direction == *prev_directions {
                direction.0 = facing.dir;
                direction.1 = facing.dir;
            } else {
                direction.0 = prev_directions.1;
                direction.1 = facing.dir;
            }
        } else if keyboard_input.pressed(KeyCode::Down) && facing.dir != Up
            || keyboard_input.pressed(KeyCode::S) && facing.dir != Up
        {
            facing.dir = Down;
            if direction.0 != direction.1 && *direction == *prev_directions {
                direction.0 = facing.dir;
                direction.1 = facing.dir;
            } else {
                direction.0 = prev_directions.1;
                direction.1 = facing.dir;
            }
        } else if keyboard_input.pressed(KeyCode::Up) && facing.dir != Down
            || keyboard_input.pressed(KeyCode::W) && facing.dir != Down
        {
            facing.dir = Up;
            if direction.0 != direction.1 && *direction == *prev_directions {
                direction.0 = facing.dir;
                direction.1 = facing.dir;
            } else {
                direction.0 = prev_directions.1;
                direction.1 = facing.dir;
            }
        }
    }
}

// hate this function, ugh.
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

        // actual movement is like 6 lines lol
        match &head.dir {
            Left if pos.x > 0 => pos.x -= 1,
            Right if pos.x < GRID_WIDTH - 1 => pos.x += 1,
            Up if pos.y < GRID_WIDTH - 1 => pos.y += 1,
            Down if pos.y > 0 => pos.y -= 1,
            _ => return,
        }

        // cycle segment positions head-to-tail, effectively making them move
        segment_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));

        // carbon copies of the position cycler, when in rome
        let segment_directions = segments
            .iter()
            .map(|e| *directions.get_mut(*e).unwrap())
            .collect::<Vec<DirectionPair>>();

        segment_directions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(dir, segment)| {
                *directions.get_mut(*segment).unwrap() = *dir;
            });

        *last_tail_direction = LastTailDirection(Some(*segment_directions.last().unwrap()));
    }
}

// I don't know what this does but if I take it out everything cries. God help me
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

// I assume this maps the grid to the screen. cool
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

struct ScoredEvent;

// sometimes caveman solution is the solution
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
                score_writer.send(ScoredEvent);
            }
        }
    }
}

/*
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
*/

// draw the outline using math!!!!!
// todo: change to sprites instead of transform shapes
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

// back in my day we had to draw the border uphill both ways
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

// copied but shrunken spawn_snake, since it doesn't need to init anything
fn spawn_segment(
    commands: &mut Commands,
    pos: Position,
    dir: DirectionPair,
    texture_atlas_handle: &Handle<TextureAtlas>,
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
            texture_atlas: texture_atlas_handle.clone(),
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

// assigns indexes to dirpairs,
// changes sprite textures based on the type of dirpair (corner) detected
fn update_textures(
    mut query: Query<
        (&mut TextureAtlasSprite, &DirectionPair),
        (With<SnakeSegment>, Without<SnakeHead>),
    >,
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

fn setup_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let style = TextStyle {
        font: asset_server.load("FiraMono-Regular.ttf"),
        font_size: 250.0,
        color: Color::rgb(0.345, 0.431, 0.459),
    };

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section("0", style).with_alignment(TextAlignment::CENTER),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(ScoreText);
}

fn update_score_text(
    mut timer: ResMut<SnakeLoop>,
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>,
) {
    timer.0.set_duration(Duration::from_millis(
        (125.0 - (score.0 as f64 / 1.0)) as u64,
    ));

    for mut text in &mut query {
        text.sections[0].value = match score.0.to_string().chars().count() {
            1 => format!("00{}", score.0),
            2 => format!("0{}", score.0),
            _ => format!("{}", score.0),
        };
    }
}

// thanks Xion
#[derive(Deref, DerefMut)]
struct SnakeLoop(Timer);
fn snake_loop(mut timer: ResMut<SnakeLoop>, time: Res<Time>) -> ShouldRun {
    if timer.0.tick(time.delta()).just_finished() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowDescriptor {
            title: "snek".to_string(),
            width: 500.0,
            height: 500.0,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.169, 0.212)))
        .insert_resource(SpriteSheet::default())
        .add_startup_system(spawn_food_helper)
        .add_startup_system(make_atlas)
        .add_system(spawn_food)
        .add_startup_system(setup_score_text)
        .add_event::<ScoredEvent>()
        .add_event::<FoodHelperEvent>()
        .add_startup_system(setup_outline)
        .add_startup_system(spawn_snake.after(make_atlas))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .insert_resource(LastTailDirection::default())
        .insert_resource(Score(0))
        .insert_resource(SnakeLoop(Timer::new(Duration::from_millis(125), true)));
    }
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(snake_controls.before(snake_movement))
            .add_system(update_score_text)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(snake_loop)
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
            );
    }
}

fn main() {
    App::new()
        .add_plugin(SetupPlugin)
        .add_plugin(GameplayPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}
