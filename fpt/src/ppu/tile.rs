use std::fmt;
use std::fs::File;
use std::io::Write;

use crate::ppu::Frame;

pub const NUM_TILES: usize = 384;
pub const TILE_PIXEL_SIZE: usize = 8;

/// Holds a 8x8 tile image as it appears in VRAM
/// (2 bytes for each 8 pixel row)
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Tile {
    pub bytes: [u8; 16],
}

impl Tile {
    #[allow(unused)]
    pub fn load(data: &[u8; 16]) -> Tile {
        Tile { bytes: *data }
    }

    // Returns the color of the pixel - a byte with only the last two bits set
    // TODO: should return a bitfield
    pub fn get_pixel(&self, y: usize, x: usize) -> u8 {
        let low_bit = (self.bytes[2 * y] >> (7 - x)) & 1;
        let high_bit = (self.bytes[2 * y + 1] >> (7 - x)) & 1;

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

/// Represents the data that lives in VRAM:
/// 3 * 128 tile blocks and two 32x32 tile maps
pub struct VRamContents {
    /// Three blocks of 128 tiles shared by the BG/Win tiles and OBJ tiles
    pub tile_data: [Tile; NUM_TILES],
    /// The first 32x32 tile map, accessed when either LCDC.3 or LCDC.6 are 0
    pub tile_map0: [u8; 1024],
    /// The second 32x32 tile map, accessed when either LCDC.3 or LCDC.6 are 1
    pub tile_map1: [u8; 1024],
}

impl Default for VRamContents {
    fn default() -> VRamContents {
        VRamContents {
            tile_map0: [0; 1024],
            tile_map1: [0; 1024],
            tile_data: [Tile { bytes: [0; 16] }; 384],
        }
    }
}

impl VRamContents {
    pub fn load(vram: &[u8]) -> VRamContents {
        let mut tilemap = VRamContents::default();

        for i in 0..NUM_TILES {
            tilemap.tile_data[i]
                .bytes
                .clone_from_slice(&vram[(16 * i)..(16 * (i + 1))]);
        }

        tilemap.tile_map0.clone_from_slice(&vram[0x1800..0x1c00]);
        tilemap.tile_map1.clone_from_slice(&vram[0x1c00..0x2000]);

        tilemap
    }

    pub fn get_tile(&self, tile_i: usize, lcdc4: bool) -> Tile {
        if lcdc4 {
            // Unsigned addressing: tiles 0-255 are in blocks 0 and 1
            self.tile_data[tile_i]
        } else {
            // Signed addressing: tiles 0-127 are in block 2, and tiles 128-255 (i.e. -128 to -1) are in block 1
            if tile_i < 128 {
                self.tile_data[tile_i + 256]
            } else {
                self.tile_data[tile_i]
            }
        }
    }
}

/// Writes a Gameboy frame to a PGM file
pub fn write_pgm_screenshot(frame: &Frame, filename: &str) {
    // TODO: code dedup
    let mut file = File::create(filename).unwrap();

    // Write the header for a 160x144 PGM image with 4 shades of gray
    write!(file, "P2\n# Game Boy screenshot: {filename}\n160 144\n3\n").unwrap();

    // Our Game Boy's framebuffer seems to have a direct correspondence to this!
    for line in frame.chunks(160) {
        let pgm_line = line
            .iter()
            .map(|p| String::from((b'3' - *p) as char) + " ") // ASCII from '0' to '3'
            .collect::<String>()
            + "\n";

        file.write_all(pgm_line.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::map::VRAM;
    use crate::Gameboy;

    // Looks like a game boy
    #[rustfmt::skip]
    const THE_TILE: [u8; 16] = [
        0x3c, 0x7e,
        0x42, 0x42,
        0x42, 0x42,
        0x42, 0x42,
        0x7e, 0x5e,
        0x7e, 0x0a,
        0x7c, 0x56,
        0x38, 0x7c,
    ];

    #[test]
    #[rustfmt::skip]
    fn test_pixel_render() {
        let tile = Tile::load(&THE_TILE);

        let formatted = format!("{:?}", tile);

        assert_eq!(
            formatted,
            ["02333320",
                "03000030",
                "03000030",
                "03000030",
                "03133330",
                "01113130",
                "03131320",
                "02333200\n"].join("\n")
        )
    }

    #[test]
    fn test_one_tile_to_vram() {
        let gb: Gameboy = Gameboy::new();
        gb.bus
            .memory_mut()
            .slice_mut(VRAM.start..VRAM.start + 16)
            .clone_from_slice(&THE_TILE[..]);

        // Parse the VRAM with our structs
        let tm: VRamContents = VRamContents::load(gb.bus.memory_mut().slice(VRAM));

        assert_eq!(
            tm.tile_data[tm.tile_map0[0] as usize],
            Tile::load(&THE_TILE)
        );
    }

    #[test]
    fn test_photograph_ppu_frame_rendering_progress() {
        let mut gb: Gameboy = Gameboy::new();
        gb.bus
            .memory_mut()
            .slice_mut(VRAM.start..VRAM.start + 16)
            .clone_from_slice(&THE_TILE[..]);

        std::fs::create_dir_all("screenshots").unwrap();
        for ly in 0..154 {
            write_pgm_screenshot(
                gb.get_frame(),
                &format!("screenshots/test_one_tile_to_vram-ly_{ly:05}.pgm"),
            );
            gb.ppu.step(456);
        }
    }
}
