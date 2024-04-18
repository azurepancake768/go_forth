use crate::{level::Level, Position, MoveOutcome, Facing, Side};
use bevy::prelude::UVec2;

pub type TileStored = Box<dyn Tile>;

impl Clone for TileStored{
    fn clone(&self) -> Self {
        self.clone_tile()
    }
}

pub trait Tile: Send + Sync{
    fn name(&self) -> String;
    fn step(&self, level: &mut Level, player: &mut Position) -> MoveOutcome;
    fn parse(meta: &[u8]) -> Option<Self> where Self: Sized;

    //for clone hack
    fn clone_tile(&self) -> TileStored;
    fn wrap(self) -> TileStored;
}

macro_rules! tile_clone_hack_boilerplate {
    () => {
        fn clone_tile(&self) -> TileStored {
        Box::new(self.clone())
    }
    fn wrap(self) -> TileStored{
        Box::new(self)
    }
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileEmpty;
impl Tile for TileEmpty{
    fn name(&self) -> String {
        String::from("empty")
    }
    fn step(&self, _level: &mut Level, _player: &mut Position) -> MoveOutcome {
        MoveOutcome::OK(None)
    }
    fn parse(_meta: &[u8]) -> Option<TileEmpty> {
        Some(TileEmpty)
    }
    tile_clone_hack_boilerplate!();
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileWall;
impl Tile for TileWall{
    fn name(&self) -> String {
        String::from("wall")
    }
    fn step(&self, _level: &mut Level, _player: &mut Position) -> MoveOutcome {
        MoveOutcome::Illegal
    }
    fn parse(_meta: &[u8]) -> Option<TileWall>{
        Some(TileWall)
    }
    tile_clone_hack_boilerplate!();
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileFinish;
impl Tile for TileFinish{
    fn name(&self) -> String {
        String::from("finish")
    }
    fn step(&self, _level: &mut Level, _player: &mut Position) -> MoveOutcome {
        MoveOutcome::Win
    }
    fn parse(_meta: &[u8]) -> Option<TileFinish> {
        Some(TileFinish)
    }
    tile_clone_hack_boilerplate!();
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TilePlayerRot{pub side: Side}
impl Tile for TilePlayerRot{
    fn name(&self) -> String {
        format!(
                "player_rot_{}",
                match self.side {
                    Side::Right => "right",
                    Side::Left => "left",
                }
            )
    }
    fn step(&self, _level: &mut Level, player: &mut Position) -> MoveOutcome {
        player.rotation =
                    player.rotation.rotate_by(match self.side {
                        Side::Left => 3,
                        Side::Right => 1,
                    });
                MoveOutcome::OK(None)
    }
    fn parse(meta: &[u8]) -> Option<TilePlayerRot> {
        if meta.len() == 0 {None}
        else if meta[0] == 0 {Some(TilePlayerRot{side: Side::Left})}
        else {Some(TilePlayerRot{side: Side::Right})}
    }
    tile_clone_hack_boilerplate!();
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileRowShift{pub facing: Facing}
impl Tile for TileRowShift{
    fn name(&self) -> String {
        format!(
                "row_shift_{}",
                match self.facing {
                    Facing::Up => "up",
                    Facing::Right => "right",
                    Facing::Down => "down",
                    Facing::Left => "left",
                }
            )
    }
    fn step(&self, level: &mut Level, player: &mut Position) -> MoveOutcome {
        let mut new_pos = (*player).position;
        let mut shift_horizontal = |dir: Facing| {
            let mut rows = level.rows();
            let mut row = rows[player.position.y as usize].clone();
            match dir {
                Facing::Right => {
                    row.insert(0, row[row.len() - 1].clone());
                    row.remove(row.len() - 1);
                }

                Facing::Left => {
                    row.push(row[0].clone());
                    row.remove(0);
                }
                _ => unreachable!(),
            };
            new_pos = UVec2::new(((player.position.x as i32 + dir.forward().x + level.width as i32) % level.width as i32) as u32, player.position.y);
            rows.remove(player.position.y as usize);
            rows.insert(player.position.y as usize, row);
            level.level = rows.iter().flatten().map(|t| (*t).clone()).collect();
        };

        match self.facing {
            Facing::Right => shift_horizontal(Facing::Right),
            Facing::Left => shift_horizontal(Facing::Left),
            _ => {
                let offset = match self.facing {
                    Facing::Up => -1,
                    Facing::Down => 1,
                    _ => unreachable!(),
                };
                level.level = level
                    .rows()
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        let mut row = row.clone();
                        row.insert(
                            player.position.x as usize,
                            level.tile_at(
                                player.position.x as u8,
                                (i as i32 + offset).rem_euclid(level.height as i32) as u8,
                            ),
                        );
                        row.remove((player.position.x + 1) as usize);
                        row
                    })
                    .flatten()
                    .collect();

            new_pos = UVec2::new(player.position.x, ((player.position.y as i32 + self.facing.forward().y + level.height as i32) % level.height as i32) as u32);
            }
        };
        MoveOutcome::OK(Some(new_pos))
    }
    fn parse(meta: &[u8]) -> Option<TileRowShift> {
        if meta.len() == 0 {None}
        else{
            use Facing::*;
            Some(TileRowShift{facing: match meta[0] & 0b00000011{
                0 => Up,
                1 => Right,
                2 => Down,
                3 => Left,
                _ => return None
            }})
        }
    }
    tile_clone_hack_boilerplate!();
}
