mod bevy_backend;
use bevy::prelude::*;
use std::{iter, mem::transmute, process::exit};
mod tile;
use tile::{*, Tile};
mod level;
use level::*;

pub const TILE_TEXTURE_DIMENSION: f32 = 32.0;
pub const TILE_TEXTURE_SCALE: f32 = 2.0;
pub const TILE_SIZE_PX: f32 = TILE_TEXTURE_DIMENSION * TILE_TEXTURE_SCALE;
pub const RIGHT_ANGLE: f32 = -3.141292 / 2.0;

fn main() {
        bevy_backend::run(Level::from_file("assets/levels/0.lvl").unwrap());
}

fn move_player(level: &mut Level, player: &mut Position, dir: Facing){
    let new_pos: UVec2 = unsafe {
        let increment = player.rotation.add_rotation(dir).forward();
        transmute(player.position.as_ivec2() + increment) 
    };
    if new_pos.x < level.width as u32 && new_pos.y < level.height as u32 {
        let old_pos = player.position;
        player.position = new_pos;
        match level.tile_at_vec(new_pos).step(level, player) {
            MoveOutcome::OK(o) => {
                if dir == Facing::Left || dir == Facing::Right && level.side_moves > 0{
                    level.side_moves -= 1;
                    player.position = o.unwrap_or(new_pos);
                }
                else if dir == Facing::Up || dir == Facing::Down {
                    player.position = o.unwrap_or(new_pos);
                }
                else{
                    info!("No side moves remaining");
                    player.position = old_pos;
                }
            },
            MoveOutcome::Win => {
                info!("Win!");
                exit(0)
            }
            MoveOutcome::Illegal => player.position = old_pos,
        }
    }

}

#[derive(Component, Clone, Copy)]
struct Position {
    position: UVec2,
    rotation: Facing,
}
impl Position {
    fn pos(x: u8, y: u8) -> Self {
        Self {
            position: UVec2::new(x as u32, y as u32),
            rotation: Facing::Up,
        }
    }

    fn new(x: u8, y: u8, rotation: Facing) -> Self {
        Self {
            position: UVec2::new(x as u32, y as u32),
            rotation,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
enum MoveOutcome {
    OK(Option<UVec2>/*, bool*/),
    Illegal,
    Win,
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

    fn rotate_by(&self, amount: i8) -> Facing {
        unsafe{
            transmute((transmute::<Facing, i8>(*self) + amount) % 4)
        }
    }

    fn add_rotation(&self, rhs: Facing) -> Facing{
        unsafe{
            transmute((transmute::<Facing, i8>(*self) + transmute::<Facing, i8>(rhs)) % 4)
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
