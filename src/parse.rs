use std::num::NonZero;

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take},
    multi::count,
    number::complete::{le_i16, le_u8, le_u16},
};

use super::elements::Element;
use super::encoding::{decode_multiline, decode_oneline, encode_multiline, encode_oneline};
use super::error::{DecodeError, EncodeError};

/// A ZZT world file.
#[derive(Clone, Debug)]
pub struct World {
    pub ammo: i16,
    pub gems: i16,
    pub keys: [bool; 7],
    pub health: i16,
    pub starting_board: i16,
    pub torches: i16,
    pub torch_cycles: i16,
    pub energizer_cycles: i16,
    pub score: i16,
    pub name: String,
    pub flags: [String; 10],
    pub time: i16,
    pub time_ticks: i16,
    pub saved_game: bool,
    pub boards: Vec<Board>,
}

impl Default for World {
    fn default() -> Self {
        World {
            ammo: 0,
            gems: 0,
            keys: [false; 7],
            health: 100,
            starting_board: 0,
            torches: 0,
            torch_cycles: 0,
            energizer_cycles: 0,
            score: 0,
            name: String::new(),
            flags: Default::default(),
            time: 0,
            time_ticks: 0,
            saved_game: false,
            boards: Vec::new(),
        }
    }
}

/// A ZZT board.
#[derive(Debug, Clone)]
pub struct Board {
    pub name: String,
    pub tiles: [Tile; 1500],
    pub max_shots: u8,
    pub is_dark: bool,
    pub exit_north: Option<NonZero<u8>>,
    pub exit_south: Option<NonZero<u8>>,
    pub exit_west: Option<NonZero<u8>>,
    pub exit_east: Option<NonZero<u8>>,
    pub restart_on_zap: bool,
    pub message: String,
    pub enter_x: u8,
    pub enter_y: u8,
    pub time_limit: i16,
    pub stats: Vec<Stat>,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            name: String::new(),
            tiles: [Tile::default(); 1500],
            max_shots: 255,
            is_dark: false,
            exit_north: None,
            exit_south: None,
            exit_west: None,
            exit_east: None,
            restart_on_zap: false,
            message: String::new(),
            enter_x: 1,
            enter_y: 1,
            time_limit: 0,
            stats: Vec::new(),
        }
    }
}

/// A status element on a ZZT board.
#[derive(Debug, Clone)]
pub struct Stat {
    pub x: u8,
    pub y: u8,
    pub x_step: i16,
    pub y_step: i16,
    pub cycle: i16,
    pub p1: u8,
    pub p2: u8,
    pub p3: u8,
    pub follower: i16,
    pub leader: i16,
    pub under: Tile,
    pub instruction_pointer: i16,
    pub program: Program,
}

impl Default for Stat {
    fn default() -> Self {
        Stat {
            x: 0,
            y: 0,
            x_step: 0,
            y_step: 0,
            cycle: 0,
            p1: 0,
            p2: 0,
            p3: 0,
            follower: -1,
            leader: -1,
            under: Tile {
                element: Element::Empty as u8,
                color: 0x0f,
            },
            instruction_pointer: 0,
            program: Program::default(),
        }
    }
}

/// A single tile on a ZZT board.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    pub element: u8,
    pub color: u8,
}

/// A stat's ZZT-OOP program.
///
/// In ZZT, stats can either have their own code or bind to another stat's code.
/// This enum prevents invalid states where both are set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Program {
    /// The stat owns its own code.
    Own(String),
    /// The stat is bound to another stat (index into the stats list).
    Bound(NonZero<u16>),
}

impl Default for Program {
    fn default() -> Self {
        Program::Own(String::new())
    }
}

impl World {
    /// Parse a ZZT world from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<World, DecodeError> {
        let (input, _) = tag(&[0xff, 0xff][..])
            .parse(bytes)
            .map_err(|_: nom::Err<DecodeError>| DecodeError::InvalidMagic)?;
        let (input, num_boards) = le_i16.parse(input)?;
        let (input, (ammo, gems, keys)) = (le_i16, le_i16, count(bool_u8, 7)).parse(input)?;
        let (input, (health, starting_board, torches, torch_cycles, energizer_cycles)) =
            (le_i16, le_i16, le_i16, le_i16, le_i16).parse(input)?;
        let (input, (_, score, name)) = (take(2usize), le_i16, pstring(20)).parse(input)?;
        let (input, flags) = count(pstring(20), 10).parse(input)?;
        let (_input, (time, time_ticks, saved_game)) = (le_i16, le_i16, bool_u8).parse(input)?;

        // Rest of header is padding; fast-forward starting from original input
        let (input, _) = take(512usize).parse(bytes)?;

        // Load boards
        let num_boards = num_boards as usize + 1;
        let (_input, chunks) = count(board_slice, num_boards).parse(input)?;
        let boards: Result<Vec<Board>, DecodeError> = chunks
            .iter()
            .map(|bytes: &&[u8]| Board::from_bytes(bytes))
            .collect();
        let boards = boards?;

        Ok(World {
            ammo,
            gems,
            keys: keys.try_into().unwrap(),
            health,
            starting_board,
            torches,
            torch_cycles,
            energizer_cycles,
            score,
            name,
            flags: flags.try_into().unwrap(),
            time,
            time_ticks,
            saved_game,
            boards,
        })
    }

    /// Serialize this world to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let mut result = Vec::with_capacity(512);
        result.push_i16(-1); // file magic: ZZT world
        result.push_i16(self.boards.len() as i16 - 1);
        result.push_i16(self.ammo);
        result.push_i16(self.gems);
        for key in self.keys {
            result.push_bool(key);
        }
        result.push_i16(self.health);
        result.push_i16(self.starting_board);
        result.push_i16(self.torches);
        result.push_i16(self.torch_cycles);
        result.push_i16(self.energizer_cycles);
        result.push_padding(2);
        result.push_i16(self.score);
        result.push_string(20, &self.name)?;
        for flag in &self.flags {
            result.push_string(20, flag)?;
        }
        result.push_i16(self.time);
        result.push_i16(self.time_ticks);
        result.push_bool(self.saved_game);
        result.push_padding(512 - result.len());

        for board in &self.boards {
            result.extend_from_slice(&board.to_bytes()?);
        }
        Ok(result)
    }
}

impl Board {
    /// Parse a board from bytes (including the 2-byte size header).
    pub fn from_bytes(bytes: &[u8]) -> Result<Board, DecodeError> {
        // Ignore length bytes
        let (input, _) = le_u16.parse(bytes)?;

        // Read board name
        let (input, name) = pstring(50)(input)?;

        // Read terrain
        const NUM_TILES: usize = 60 * 25;
        let mut input = input;
        let mut tiles = [Tile::default(); NUM_TILES];
        let mut tile_index = 0;
        while tile_index < NUM_TILES {
            let (next_input, (count, element, color)) = (le_u8, le_u8, le_u8).parse(input)?;
            input = next_input;
            let count: usize = if count == 0 { 256 } else { count.into() };
            for _ in 0..count {
                if tile_index >= NUM_TILES {
                    return Err(DecodeError::InvalidTileCount(tile_index + 1));
                }
                tiles[tile_index] = Tile { element, color };
                tile_index += 1;
            }
        }

        // Read board info
        let (input, (max_shots, is_dark)) = (le_u8, bool_u8).parse(input)?;
        let (input, (exit_n, exit_s, exit_w, exit_e)) =
            (le_u8, le_u8, le_u8, le_u8).parse(input)?;
        let exit_north = NonZero::new(exit_n);
        let exit_south = NonZero::new(exit_s);
        let exit_west = NonZero::new(exit_w);
        let exit_east = NonZero::new(exit_e);
        let (input, (restart_on_zap, message)) = (bool_u8, pstring(58)).parse(input)?;
        let (input, (enter_x, enter_y, time_limit)) = (le_u8, le_u8, le_i16).parse(input)?;
        let (input, _) = take(16usize)(input)?;

        // Read stats
        let (input, num_stats) = le_i16(input)?;
        let num_stats = num_stats + 1;
        if num_stats < 0 {
            return Err(DecodeError::NegativeStatCount);
        }
        let (_input, stats) = count(Stat::parse, num_stats as usize).parse(input)?;

        Ok(Board {
            name,
            tiles,
            max_shots,
            is_dark,
            exit_north,
            exit_south,
            exit_east,
            exit_west,
            restart_on_zap,
            message,
            enter_x,
            enter_y,
            time_limit,
            stats,
        })
    }

    /// Parse a standalone .brd file (same format, no world header).
    pub fn from_brd_bytes(bytes: &[u8]) -> Result<Board, DecodeError> {
        Board::from_bytes(bytes)
    }

    /// Serialize this board to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let mut result = vec![];
        result.push_padding(2); // reserve space for board size
        result.push_string(50, &self.name)?;

        // Encode terrain
        let mut iter = self.tiles.iter().peekable();
        while let Some(tile) = iter.next() {
            let mut count: u8 = 1;
            while count < 255 && iter.peek().map_or(false, |&next_tile| next_tile == tile) {
                count += 1;
                iter.next();
            }
            result.push(count);
            result.push(tile.element);
            result.push(tile.color);
        }

        // Board info
        result.push(self.max_shots);
        result.push_bool(self.is_dark);
        result.push(self.exit_north.map_or(0, |n| n.get()));
        result.push(self.exit_south.map_or(0, |n| n.get()));
        result.push(self.exit_west.map_or(0, |n| n.get()));
        result.push(self.exit_east.map_or(0, |n| n.get()));
        result.push_bool(self.restart_on_zap);
        result.push_string(58, &self.message)?;
        result.push(self.enter_x);
        result.push(self.enter_y);
        result.push_i16(self.time_limit);
        result.push_padding(16);

        // Stats
        let num_stats: i16 = (self.stats.len() as i16) - 1;
        result.push_i16(num_stats);
        for stat in &self.stats {
            result.extend_from_slice(&stat.to_bytes()?);
        }

        // Fix up board size
        let size: u16 = (result.len() - 2)
            .try_into()
            .map_err(|_| EncodeError::BoardTooLarge)?;
        result.splice(0..2, size.to_le_bytes());

        Ok(result)
    }
}

impl Stat {
    fn parse(input: &[u8]) -> IResult<&[u8], Self, DecodeError> {
        let (input, (x, y, x_step, y_step)) = (le_u8, le_u8, le_i16, le_i16).parse(input)?;
        let (input, (cycle, p1, p2, p3)) = (le_i16, le_u8, le_u8, le_u8).parse(input)?;
        let (input, (follower, leader)) = (le_i16, le_i16).parse(input)?;
        let (input, (under_element, under_color)) = (le_u8, le_u8).parse(input)?;
        let (input, _) = take(4usize)(input)?; // unused pointer
        let (input, (instruction_pointer, length)) = (le_i16, le_i16).parse(input)?;
        let (input, _) = take(8usize)(input)?; // padding

        let (input, program) = if length < 0 {
            // Negative length means bound to another stat
            (
                input,
                Program::Bound(NonZero::new((-length) as u16).unwrap()),
            )
        } else {
            // Positive length means own code
            let (input, code_bytes) = take(length as usize)(input)?;
            (input, Program::Own(decode_multiline(code_bytes)))
        };

        Ok((
            input,
            Stat {
                x,
                y,
                x_step,
                y_step,
                follower,
                leader,
                cycle,
                p1,
                p2,
                p3,
                under: Tile {
                    element: under_element,
                    color: under_color,
                },
                instruction_pointer,
                program,
            },
        ))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let mut result = vec![];
        result.push(self.x);
        result.push(self.y);
        result.push_i16(self.x_step);
        result.push_i16(self.y_step);
        result.push_i16(self.cycle);
        result.push(self.p1);
        result.push(self.p2);
        result.push(self.p3);
        result.push_i16(self.follower);
        result.push_i16(self.leader);
        result.push(self.under.element);
        result.push(self.under.color);
        result.push_padding(4); // unused pointer
        result.push_i16(self.instruction_pointer);

        match &self.program {
            Program::Bound(index) => {
                result.push_i16(-(index.get() as i16));
                result.push_padding(8);
            }
            Program::Own(code) => {
                let code_bytes = encode_multiline(code)?;
                result.push_i16(code_bytes.len() as i16);
                result.push_padding(8);
                result.extend_from_slice(&code_bytes);
            }
        }

        Ok(result)
    }
}

// Parsing helpers

fn bool_u8(input: &[u8]) -> IResult<&[u8], bool, DecodeError> {
    let (input, byte) = le_u8(input)?;
    Ok((input, byte != 0))
}

fn pstring(cap: u8) -> impl Fn(&[u8]) -> IResult<&[u8], String, DecodeError> {
    move |input: &[u8]| -> IResult<&[u8], String, DecodeError> {
        let (input, len) = le_u8(input)?;
        let actual_len = len.min(cap);
        let (input, data) = take(actual_len)(input)?;
        let (input, _) = take(cap - actual_len)(input)?;
        Ok((input, decode_oneline(data)))
    }
}

fn board_slice(bytes: &[u8]) -> IResult<&[u8], &[u8], DecodeError> {
    let (_, size) = le_u16.parse(bytes)?;
    take(size as usize + 2).parse(bytes)
}

// Serialization helpers

trait SerializationHelpers {
    fn push_bool(&mut self, value: bool);
    fn push_i16(&mut self, value: i16);
    fn push_string(&mut self, cap: u8, value: &str) -> Result<(), EncodeError>;
    fn push_padding(&mut self, size: usize);
}

impl SerializationHelpers for Vec<u8> {
    fn push_bool(&mut self, value: bool) {
        self.push(if value { 1 } else { 0 });
    }

    fn push_i16(&mut self, value: i16) {
        self.extend(value.to_le_bytes());
    }

    fn push_string(&mut self, cap: u8, value: &str) -> Result<(), EncodeError> {
        let bytes = encode_oneline(value)?;
        if bytes.len() > cap as usize {
            return Err(EncodeError::StringTooLong { max: cap });
        }
        self.push(bytes.len() as u8);
        self.extend_from_slice(&bytes);
        self.push_padding(cap as usize - bytes.len());
        Ok(())
    }

    fn push_padding(&mut self, size: usize) {
        self.resize(self.len() + size, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    const BYTES: &[u8] = include_bytes!("../tests/fixtures/all.zzt");

    #[test]
    fn roundtrip() {
        let world = World::from_bytes(BYTES).unwrap();
        let encoded = world.to_bytes().unwrap();
        assert_eq!(BYTES, &encoded[..]);
    }

    #[test]
    fn decode_snapshot() {
        let world = World::from_bytes(BYTES).unwrap();
        assert_debug_snapshot!(world);
    }
}
