use bevy::{
    prelude::*,
    window::PrimaryWindow
};
use std::{iter, mem::transmute, process::exit};

const TILE_TEXTURE_DIMENSION: f32 = 32.0;
const TILE_TEXTURE_SCALE: f32 = 2.0;
const TILE_SIZE_PX: f32 = TILE_TEXTURE_DIMENSION * TILE_TEXTURE_SCALE;

fn main() {
    let mut level = Vec::from([TileType::Empty; 8]);
    level.append(&mut Vec::from([TileType::Wall; 4]));
    level.append(&mut Vec::from([TileType::Empty; 4]));

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
    mut q: Query<(&mut Transform, &Positioned)>,
    w: Query<&Window, With<PrimaryWindow>>,
    level: Res<Level>,
) {
    let window = w.single();

    for (mut t, Positioned(p)) in q.iter_mut() {
        t.translation = level_to_world_pos(p.x as u8, p.y as u8, &level, window).extend(0.0);
    }
}

fn render_level(
    mut q: Query<(&mut Handle<Image>, &mut Tile, &Positioned)>,
    assets: Res<AssetServer>,
    level: Res<Level>,
) {
    for (mut tex, mut tile, &Positioned(p)) in q.iter_mut() {
        tile.0 = level.tile_at_vec(p);
        *tex = assets.load(format!(
            "sprites/tiles/{}.png",
            tile.0.name()
        ));
    }
}

fn handle_kb_input(
    mut q: Query<(&Player, &mut Positioned)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut level: ResMut<Level>,
) {
    let (Player(facing), mut pos) = q.single_mut();
    let Positioned(ref mut pos) = *pos;

    match match keyboard_input.get_just_pressed().nth(0) {
        Some(KeyCode::Up) => Some(1),
        Some(KeyCode::W) => Some(1),
        Some(KeyCode::Down) => Some(-1),
        Some(KeyCode::S) => Some(-1),
        _ => None,
    } {
        Some(n) => {
            let new_pos = pos.wrapping_add(unsafe { transmute(n * facing.forward()) });
            let mut rows = level.rows();
            if new_pos.x < level.width as u32 && new_pos.y < level.height as u32 {
                match level.tile_at_vec(new_pos).step(
                    *pos, *facing, &mut rows,
                ) {
                    MoveOutcome::OK(o) => *pos = o.unwrap_or(new_pos),
                    MoveOutcome::Win => {
                        info!("Win");
                        exit(0);
                    }
                    MoveOutcome::Illegal => {}
                }
                level.level = rows.iter().flatten().map(|i| *i).collect();
            }
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
                    texture: assets.load(format!(
                        "sprites/tiles/{}.png",
                        level.tile_at(i, j).name()
                    )),
                    ..default()
                },
                Positioned::new(i, j),
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
        Positioned(UVec2::splat(0)),
        Player(Facing::Up),
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
    }/*

    fn tile_at_mut(&mut self, x:u8, y: u8) -> &mut TileType{
        &mut self.level[(y * self.width as u8 + x) as usize]
    }*/

    fn tile_at_vec(&self, p: UVec2) -> TileType {
        self.tile_at(p.x as u8, p.y as u8)
    }/*

    fn tile_at_vec_mut(&mut self, p:UVec2) -> &mut TileType{
       &mut self.tile_at(p.x as u8, p.y as u8)
    }*/

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
}

impl TileType{
    fn name(&self) -> String{
        match self{
            Self::Empty => String::from("empty"),
            Self::Wall  => String::from("wall")
        }
    }
    fn step(&self, pos: UVec2, player_facing: Facing, level: &mut Vec<Vec<TileType>>) -> MoveOutcome {
        match self{
            Self::Empty => MoveOutcome::OK(None),
            Self::Wall  => MoveOutcome::Illegal
        }
    }
}

#[derive(Clone, Copy)]
enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    fn forward(&self) -> IVec2 {
        match *self {
            Self::Up => IVec2::new(0, 1),
            Self::Right => IVec2::new(1, 0),
            Self::Down => IVec2::new(0, -1),
            Self::Left => IVec2::new(-1, 0),
        }
    }
}

#[derive(Component)]
struct Positioned(UVec2);
impl Positioned {
    fn new(x: u8, y: u8) -> Self {
        Self(UVec2::new(x as u32, y as u32))
    }
}

#[derive(Component)]
struct Player(Facing);

#[derive(Component)]
struct Tile(TileType);

enum MoveOutcome {
    OK(Option<UVec2>),
    Illegal,
    Win,
}
