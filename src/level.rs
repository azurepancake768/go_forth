use crate::*;
use std::fs::*;

#[derive(Resource)]
pub struct Level {
    pub width: u8,
    pub height: u8,
    pub level: Vec<TileStored>,
    pub side_moves: u8
}

impl Level {
    pub fn tile_at(&self, x: u8, y: u8) -> TileStored {
        self.level[(y * self.width as u8 + x) as usize].clone()
    }

    pub fn tile_at_vec(&self, p: UVec2) -> TileStored {
        self.tile_at(p.x as u8, p.y as u8)
    }

    pub fn rows(&self) -> Vec<Vec<TileStored>> {
        let mut result: Vec<Vec<TileStored>> = iter::repeat(
            iter::repeat(TileEmpty)
                .take(self.width as usize)
                .map(|t| Box::new(t) as TileStored)
                .collect()
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

    pub fn from_bytes(source: &[u8]) -> Option<Level>{
        if source.len() < 3{ //3 is the length of empty level
            return None;
        }

        let (width, height) = (
            source[2] & 0b11110000 >> 4,
            source[2] & 0b00001111
        );
        
        let mut tiles: Vec<TileStored> = Vec::with_capacity((width * height) as usize);

        let mut i = 4;
        while i < source.len(){
            let (tile_len, tile_type_id) = (
                (source[i] & 0b11110000) >> 4,
                source[i] & 0b00001111
            );
            let meta_bytes = &source[i+1..i+1+tile_len as usize];
            tiles.push(match tile_type_id{
                0 => TileEmpty::parse(meta_bytes).unwrap().wrap(),
                1 => TileWall::parse(meta_bytes).unwrap().wrap(),
                2 => TileFinish::parse(meta_bytes).unwrap().wrap(),
                3 => TilePlayerRot::parse(meta_bytes).unwrap().wrap(),
                4 => TileRowShift::parse(meta_bytes).unwrap().wrap(),
                _ => TileEmpty::parse(meta_bytes).unwrap().wrap()
            });
            i += 1 + tile_len as usize;
        }
        Some(
            Level { width, height, level: tiles, side_moves: source[3] }
        )
    }

    pub fn from_file(path: &str) -> Option<Level>{
        let res = read(path);
        Level::from_bytes(&res.ok()?)
    }
}

