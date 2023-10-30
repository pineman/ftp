use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Tile {
    pub pixels: [u8; 16],
}

impl Tile {
    #[allow(unused)]
    pub fn load(data: &[u8; 16]) -> Tile {
        Tile { pixels: *data }
    }
    pub fn get_pixel(&self, y: usize, x: usize) -> u8 {
        let low_bit = (self.pixels[2 * y] >> (7 - x)) & 1;
        let high_bit = (self.pixels[2 * y + 1] >> (7 - x)) & 1;

        (high_bit << 1) + low_bit
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..8 {
            for j in 0..8 {
                write!(f, "{}", self.get_pixel(i, j))?;
            }
            writeln!(f)?;
        }
        write!(f, "")
    }
}

pub struct TileMap {
    pub tile_map0: [u8; 1024],
    pub tile_map1: [u8; 1024],
    pub tiles: [Tile; 384],
}

impl TileMap {
    pub fn default() -> TileMap {
        TileMap {
            tile_map0: [0; 1024],
            tile_map1: [0; 1024],
            tiles: [Tile { pixels: [0; 16] }; 384],
        }
    }

    pub fn load(vram: &[u8]) -> TileMap {
        let mut tilemap = TileMap::default();

        for i in 0..384 {
            tilemap.tiles[i]
                .pixels
                .clone_from_slice(&vram[(16 * i)..(16 * (i + 1))]);
        }

        tilemap.tile_map0.clone_from_slice(&vram[0x1800..0x1c00]);
        tilemap.tile_map1.clone_from_slice(&vram[0x1c00..0x2000]);

        tilemap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::map::VRAM;
    use crate::Gameboy;

    #[test]
    #[rustfmt::skip]
    fn test_pixel_render() {
        let tile = Tile::load(&[
            0x3c, 0x7e,
            0x42, 0x42,
            0x42, 0x42,
            0x42, 0x42,
            0x7e, 0x5e,
            0x7e, 0x0a,
            0x7c, 0x56,
            0x38, 0x7c,
        ]);

        let formatted = format!("{:?}", tile);

        assert_eq!(
            formatted,
            vec![
                "02333320",
                "03000030",
                "03000030",
                "03000030",
                "03133330",
                "01113130",
                "03131320",
                "02333200\n"
            ].join("\n")
        )
    }

    #[test]
    fn test_one_tile_to_vram() {
        // A 8x8 Game Boy icon tile
        let one_tile: [u8; 16] = [
            0x3c, 0x7e, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7e, 0x5e, 0x7e, 0x0a, 0x7c, 0x56,
            0x38, 0x7c,
        ];

        // Set up the Game Boy
        let gb: Gameboy = Gameboy::new();
        gb.bus
            .memory()
            .slice_mut(VRAM.start..VRAM.start + 16)
            .clone_from_slice(&one_tile[..]);

        // Parse the VRAM with our structs
        let tm: TileMap = TileMap::load(gb.bus.memory().slice(VRAM));

        assert_eq!(tm.tiles[tm.tile_map0[0] as usize], Tile::load(&one_tile));
    }
}
