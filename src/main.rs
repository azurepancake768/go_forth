use bevy::{prelude::*, window::PrimaryWindow};
use std::{iter, mem::transmute, process::exit};

const TILE_TEXTURE_DIMENSION: f32 = 32.0;
const TILE_TEXTURE_SCALE: f32 = 2.0;
const TILE_SIZE_PX: f32 = TILE_TEXTURE_DIMENSION * TILE_TEXTURE_SCALE;
const RIGHT_ANGLE: f32 = -3.141292 / 2.0;

fn main() {
    let mut level = Vec::from([TileType::Empty; 3]);
    level.append(&mut Vec::from([TileType::PlayerRot(Side::Left); 1]));
    level.append(&mut Vec::from([TileType::Empty; 3]));
    level.append(&mut Vec::from([TileType::Wall; 1]));
    level.append(&mut Vec::from([TileType::Empty; 3]));
    level.append(&mut Vec::from([TileType::PlayerRot(Side::Right); 1]));
    level.append(&mut Vec::from([TileType::RowShift(Facing::Left); 1]));
    level.append(&mut Vec::from([TileType::Empty; 1]));
    level.append(&mut Vec::from([TileType::PlayerRot(Side::Right); 1]));
    level.append(&mut Vec::from([TileType::PlayerRot(Side::Left); 1]));

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Level {
            width: 4,
            height: 4,
            level,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, handle_kb_input)
        .add_systems(PostUpdate, (render_positioned, render_level))
        .run();
}

fn render_positioned(
    mut q: Query<(&mut Transform, &LevelTransform)>,
    w: Query<&Window, With<PrimaryWindow>>,
    level: Res<Level>,
) {
    let window = w.single();

    for (
        mut t,
        LevelTransform {
            position: p,
            rotation: rot,
        },
    ) in q.iter_mut()
    {
        *t = Transform {
            translation: level_to_world_pos(p.x as u8, p.y as u8, &level, window).extend(0.0),
            rotation: rot.unwrap_or(Facing::Up).rotation_quat(),
            ..*t
        };
    }
}

fn render_level(
    mut q: Query<(&mut Handle<Image>, &mut Tile, &LevelTransform)>,
    assets: Res<AssetServer>,
    level: Res<Level>,
) {
    for (mut tex, mut tile, &LevelTransform { position: p, .. }) in q.iter_mut() {
        tile.0 = level.tile_at_vec(p);
        *tex = assets.load(format!("sprites/tiles/{}.png", tile.0.name()));
    }
}

fn handle_kb_input(
    mut q: Query<&mut LevelTransform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    level: ResMut<Level>,
) {
    let level = level.into_inner();
    let pos = q.single_mut();
    let transform @ &mut LevelTransform {
        position: pos,
        rotation: facing,
    } = pos.into_inner();
    match match keyboard_input.get_just_pressed().nth(0) {
        Some(KeyCode::Up) => Some(1),
        Some(KeyCode::W) => Some(1),
        Some(KeyCode::Down) => Some(-1),
        Some(KeyCode::S) => Some(-1),
        _ => None,
    } {
        Some(n) => {
            let new_pos: UVec2 =
                unsafe { transmute(pos.as_ivec2() + n * facing.unwrap_or(Facing::Up).forward()) };
            let mut rows = level.rows();
            if new_pos.x < level.width as u32 && new_pos.y < level.height as u32 {
                let old_pos = transform.position;
                transform.position = new_pos;
                match level.tile_at_vec(new_pos).step(level, transform) {
                    MoveOutcome::OK(o) => transform.position = o.unwrap_or(new_pos),
                    MoveOutcome::Win => {
                        info!("Win!");
                        exit(0)
                    }
                    MoveOutcome::Illegal => transform.position = old_pos,
                }
                //level.level = rows.iter().flatten().map(|i| *i).collect();
            }
        info!("{}", pos.y);
        }
        None => {}
    };
}

fn level_to_world_pos(x: u8, y: u8, level: &Level, window: &Window) -> Vec2 {
    let parity_offset = |n: u8| if n % 2 == 0 { 0.0 } else { -0.5 };
    Vec2::new(
        (window.width() / 2.0)
            + ((x as i8 - level.width as i8 / 2) as f32 + parity_offset(level.width as u8))
                * TILE_SIZE_PX,
        (window.height() / 2.0)
            + ((y as i8 - level.height as i8 / 2) as f32 + parity_offset(level.height as u8))
                * TILE_SIZE_PX,
    )
}

fn setup(
    mut commands: Commands,
    q: Query<&Window, With<PrimaryWindow>>,
    assets: Res<AssetServer>,
    level: Res<Level>,
) {
    let window = q.get_single().expect("Only one primary window");

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ..default()
    });

    for i in 0..level.width {
        for j in 0..level.height {
            let (i, j) = (i as u8, j as u8);
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(TILE_TEXTURE_SCALE)),
                    texture: assets
                        .load(format!("sprites/tiles/{}.png", level.tile_at(i, j).name())),
                    ..default()
                },
                LevelTransform::pos(i, j),
                Tile(level.tile_at(i, j)),
            ));
        }
    }

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(TILE_TEXTURE_SCALE)),
            texture: assets.load("sprites/player.png"),
            ..default()
        },
        LevelTransform::new(0, 0, Some(Facing::Up)),
        Player,
    ));
}

#[derive(Resource)]
struct Level {
    width: u8,
    height: u8,
    level: Vec<TileType>,
}

impl Level {
    fn tile_at(&self, x: u8, y: u8) -> TileType {
        self.level[(y * self.width as u8 + x) as usize]
    }

    fn tile_at_vec(&self, p: UVec2) -> TileType {
        self.tile_at(p.x as u8, p.y as u8)
    }

    fn rows(&self) -> Vec<Vec<TileType>> {
        let mut result: Vec<Vec<TileType>> = iter::repeat(
            iter::repeat(TileType::Empty)
                .take(self.width as usize)
                .collect(),
        )
        .take(self.height as usize)
        .collect();
        for i in 0..self.width {
            for j in 0..self.height {
                result[i as usize][j as usize] = self.tile_at(j as u8, i as u8);
            }
        }
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TileType {
    Empty,
    Wall,
    Finish,
    PlayerRot(Side),
    RowShift(Facing),
}

impl TileType {
    fn name(&self) -> String {
        use TileType::*;
        match self {
            Empty => String::from("empty"),
            Wall => String::from("wall"),
            Finish => String::from("finish"),
            PlayerRot(side) => format!(
                "player_rot_{}",
                match side {
                    Side::Right => "right",
                    Side::Left => "left",
                }
            ),
            RowShift(facing) => format!(
                "row_shift_{}",
                match facing {
                    Facing::Up => "up",
                    Facing::Right => "right",
                    Facing::Down => "down",
                    Facing::Left => "left",
                }
            ),
        }
    }
    fn step(&self, level: &mut Level, player: &mut LevelTransform) -> MoveOutcome {
        use TileType::*;
        match self {
            Empty => MoveOutcome::OK(None),
            Wall => MoveOutcome::Illegal,
            Finish => MoveOutcome::Win,
            PlayerRot(side) => {
                player.rotation =
                    Some(player.rotation.unwrap_or(Facing::Up).rotate_by(match side {
                        Side::Left => -1,
                        Side::Right => 1,
                    }));
                MoveOutcome::OK(None)
            }
            RowShift(facing) => {
                let mut new_pos = player.position;
                let mut shift_horizontal = |dir: Facing| {
                    info!("idx {}, y {}", player.position.y, player.position.y);
                    let mut rows = level.rows();
                    let mut row = rows[player.position.y as usize].clone();
                    match dir {
                        Facing::Right => {
                            row.insert(0, row[row.len() - 1]);
                            row.remove(row.len() - 1);
                        }

                        Facing::Left => {
                            row.push(row[0]);
                            row.remove(0);
                        }
                        _ => panic!("Not a horiz shift"),
                    };
                    rows.remove(player.position.y as usize);
                    rows.insert(player.position.y as usize, row);
                    level.level = rows.iter().flatten().map(|t| *t).collect();
                    new_pos.x = ((player.position.x as i32 + dir.forward().x + level.width as i32) % level.width as i32) as u32;
                };

                match facing {
                    Facing::Right => shift_horizontal(Facing::Right),
                    Facing::Left => shift_horizontal(Facing::Left),
                    _ => {
                        let offset = match facing {
                            Facing::Up => 1,
                            Facing::Down => -1,
                            _ => panic!("Not a vert shift"),
                        };
                        let old_level = level.rows().clone();
                        let mut old_level_mut = old_level.clone();
                        old_level_mut.iter_mut().for_each(|mut row| {
                            row.insert(
                                player.position.x as usize,
                                old_level[(player.position.y as isize + offset).abs() as usize]
                                    [player.position.x as usize],
                            );
                            row.remove(player.position.x as usize);
                        });
                        level.level = old_level_mut.iter().flatten().map(|t| *t).collect()
                    }
                };
                MoveOutcome::OK(Some(new_pos))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i8)]
enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    fn forward(&self) -> IVec2 {
        match self {
            Facing::Up => IVec2::Y,
            Facing::Right => IVec2::X,
            Facing::Down => IVec2::NEG_Y,
            Facing::Left => IVec2::NEG_X,
        }
    }

    fn right(&self) -> IVec2 {
        match self {
            Facing::Up => IVec2::X,
            Facing::Right => IVec2::NEG_Y,
            Facing::Down => IVec2::NEG_X,
            Facing::Left => IVec2::Y,
        }
    }

    fn back(&self) -> IVec2 {
        match self {
            Facing::Up => IVec2::NEG_Y,
            Facing::Right => IVec2::NEG_X,
            Facing::Down => IVec2::Y,
            Facing::Left => IVec2::X,
        }
    }

    fn left(&self) -> IVec2 {
        match self {
            Facing::Up => IVec2::NEG_Y,
            Facing::Right => IVec2::NEG_X,
            Facing::Down => IVec2::Y,
            Facing::Left => IVec2::X,
        }
    }

    fn rotate_by(&self, amount: i8) -> Facing {
        use Facing::*;
        match (amount % 4).abs() {
            1 => match self {
                Up => Right,
                Right => Down,
                Down => Left,
                Left => Up,
            },
            2 => match self {
                Up => Down,
                Right => Left,
                Down => Up,
                Left => Right,
            },
            3 => match self {
                Up => Left,
                Right => Up,
                Down => Right,
                Left => Down,
            },
            4 => *self,
            _ => {
                panic!("-1<x%4<4");
            }
        }
    }

    fn rotation_quat(&self) -> Quat {
        Quat::from_rotation_z(*self as u32 as f32 * RIGHT_ANGLE)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Side {
    Right,
    Left,
}

impl Into<Facing> for Side {
    fn into(self) -> Facing {
        match self {
            Side::Right => Facing::Right,
            Self::Left => Facing::Left,
        }
    }
}

#[derive(Component)]
struct LevelTransform {
    position: UVec2,
    rotation: Option<Facing>,
}
impl LevelTransform {
    fn pos(x: u8, y: u8) -> Self {
        Self {
            position: UVec2::new(x as u32, y as u32),
            rotation: None,
        }
    }

    fn pos_vec(position: UVec2) -> Self {
        Self {
            position,
            rotation: None,
        }
    }

    fn new(x: u8, y: u8, rotation: Option<Facing>) -> Self {
        Self {
            position: UVec2::new(x as u32, y as u32),
            rotation,
        }
    }

    fn new_vec(position: UVec2, rotation: Option<Facing>) -> Self {
        Self { position, rotation }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Tile(TileType);

enum MoveOutcome {
    OK(Option<UVec2>),
    Illegal,
    Win,
}
