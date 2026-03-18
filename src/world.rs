use std::{fmt::Debug, num::NonZero};

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take},
    multi::count,
    number::complete::{le_i16, le_u8, le_u16},
};

use super::elements::Element;
use super::errors::{DecodeError, EncodeError};
use super::text::{decode_multiline, decode_oneline, encode_multiline, encode_oneline};

/// A data structure representing a ZZT world.
///
/// ZZT uses the same format for both worlds proper (.ZZT files) as well as their saved games (.SAV); saved games are
/// just snapshots of the world state after it's been played in. This `World` struct is for working with either type of
/// file.
///
/// Some of the fields are more esoteric than others due to this need of being able to represent the entire world state,
/// not just the starting conditions. If you just want to create a new world, it's safe to call `::default()` and leave
/// most of the fields alone. The only one that needs a non-default value is [`World::name`].
#[derive(Clone, Debug)]
pub struct World {
    /// Value of the `ammo` counter.
    pub ammo: i16,
    /// Value of the `gems` counter.
    pub gems: i16,
    /// Which keys the player is carrying.
    pub keys: Keys,
    /// Value of the `health` counter.
    pub health: i16,
    /// Index of the board the player starts on.
    ///
    /// If this is a saved game, this field holds the index of the board that the player is _currently_ on, which is
    /// where they will start play upon restoring their save.
    pub starting_board: i16,
    /// Value of the `torches` counter.
    pub torches: i16,
    /// Remaining cycles of torch light (0 = torch not active).
    pub torch_cycles: i16,
    /// Remaining cycles of energizer effect (0 = not active).
    pub energizer_cycles: i16,
    /// Value of the `score` counter.
    pub score: i16,
    /// Stem of the world's filename, e.g., "TOWN" for a world named TOWN.ZZT (max 20 characters).
    ///
    /// This field should typically match the filename of the .ZZT file you're writing to disk. If it differs, ZZT may
    /// have trouble loading your file.
    ///
    /// For saved games (.SAV files), this field should match the filename of the .ZZT file that was being played.
    /// played. ZZT uses this field to associate the saved game with the world file it came from.
    pub name: String,
    /// Named flags set by ZZT-OOP `#set` commands (max 20 characters each).
    ///
    /// Empty strings represent unused slots. The slot order is mostly invisible to ZZT-OOP, and ZZT will fill up the
    /// slots from lowest to highest. However, order does matter once the array is full: trying to `#set` an eleventh
    /// flag overwrites the tenth slot.
    ///
    /// The file format supports arbitrary names, but ZZT-OOP only recognizes uppercase flags. Additionally, ZZT-OOP's
    /// quirky parsing rules mean that many special characters are not recognized; see [this wiki
    /// article](https://wiki.zzt.org/wiki/Set) for details.
    pub flags: [String; 10],
    /// Value of the `time` counter.
    pub time: i16,
    /// Sub-second state for the `time` countdown.
    ///
    /// When on a board with a time limit, ZZT decrements the `time` counter approximately every second. This field
    /// tracks partial seconds that have elapsed, so that ZZT can decide between decrementing `time` during this tick
    /// versus waiting another tick.
    pub time_ticks: i16,
    /// Whether this file is a saved game (.SAV) rather than a world file (.ZZT).
    ///
    /// ZZT 3.2 uses this as an anti-cheat mechanism: the built-in editor refuses to open any files that have it set to
    /// true. Otherwise, people could temporarily rename their save to have a .ZZT extension, then edit their way out of
    /// a sticky situation.
    ///
    /// (This was never very secure, and is even less of a deterrent nowadays. But it's still part of the file format.)
    pub saved_game: bool,
    /// The boards in this world.
    ///
    /// A valid world must have at least 1 board (the title screen). Trying to serialize an empty world will result in
    /// an encoding error.
    ///
    /// ZZT 3.2 supports a maximum of 101 boards per world. The file format (and zztff) can handle larger worlds, but
    /// trying to load them in ZZT may cause it to crash.
    pub boards: Vec<Board>,
}

impl Default for World {
    fn default() -> Self {
        World {
            ammo: 0,
            gems: 0,
            keys: Keys::default(),
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

/// The player's keys, as stored in the world header.
#[derive(Clone, Debug, Default)]
pub struct Keys {
    pub blue: bool,
    pub green: bool,
    pub cyan: bool,
    pub red: bool,
    pub purple: bool,
    pub yellow: bool,
    pub white: bool,
}

/// An individual ZZT board.
///
/// Boards mainly exist within a containing [`World`]. But as part of a editing workflow, they can
/// also be packaged as standalone .BRD files, which use the same format; use
/// [`Board::from_brd_bytes`] to parse one.
#[derive(Clone)]
pub struct Board {
    /// The board's title.
    ///
    /// Board titles are not shown during gameplay; they exist for organizational purposes while
    /// editing.
    ///
    /// The file format supports a maximum length of 50 characters. But editors usually impose
    /// shorter limits: ZZT's internal editor artificially limits titles to 34 characters, and
    /// titles longer than 42 characters may cause visual glitches when displayed in a
    /// standard-width message box.
    pub name: String,
    /// The 60x25 terrain grid, stored in order from left-to-right, top-to-bottom.
    ///
    /// For convenience, [`Board::tile`] and [`Board::set_tile`] are available for coordinate-based
    /// access, but in some cases if you are updating lots of tiles, it may be more efficient to
    /// directly mutate this array.
    pub tiles: [Tile; 1500],
    /// Maximum number of bullets the player can have on screen at once on this board.
    pub max_shots: u8,
    /// Whether the board is dark.
    pub is_dark: bool,
    /// Board index of the northern neighbor, or `None` for no exit.
    ///
    /// Internally, ZZT uses zero to indicate "no exit". For this reason, it is impossible for a
    /// board edge to link to the title screen, because board index 0 is unrepresentable.
    pub exit_north: Option<NonZero<u8>>,
    /// Board index of the southern neighbor, or `None` for no exit.
    pub exit_south: Option<NonZero<u8>>,
    /// Board index of the western neighbor, or `None` for no exit.
    pub exit_west: Option<NonZero<u8>>,
    /// Board index of the eastern neighbor, or `None` for no exit.
    pub exit_east: Option<NonZero<u8>>,
    /// Whether the player restarts at the board entrance after taking damage.
    pub restart_on_zap: bool,
    /// The currently-flashing status bar message (max 58 characters).
    pub message: String,
    /// X coordinate where the player entered the board (1-based).
    ///
    /// The entrance coordinates are part of the "re-enter when zapped" feature. The values only
    /// really matter if the user starts out on this board; they will be overwritten with the
    /// player's actual entry coordinates during play.
    pub enter_x: u8,
    /// Y coordinate where the player entered the board (1-based).
    pub enter_y: u8,
    /// Time limit for the board in seconds (0 = no limit).
    pub time_limit: i16,
    /// Status elements on this board (objects, creatures, the player, etc.).
    ///
    /// ZZT 3.2 supports a maximum of 151 stats per board. The file format can handle more than
    /// that, and zztff will happily read/write more than 151 stats, but ZZT may crash if this limit
    /// is exceeded.
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

/// Manual implementation of Debug that hides the contents of the `tiles` array.
///
/// The default formatting of 1500 Tile structs is almost never useful to see in the Debug output
/// for a Board, let alone a World. And visualizing terrain isn't the focus of this library. So to
/// make the stock Debug more generally useful, we skip the terrain.
///
/// If you want to inspect the terrain, you probably want to write your own visualizer for it.
/// (Or you could Debug format the `tiles` field individually if you honestly really want that.)
impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Board")
            .field("name", &self.name)
            .field("tiles", &format_args!("[Tile; 1500]"))
            .field("max_shots", &self.max_shots)
            .field("is_dark", &self.is_dark)
            .field("exit_north", &self.exit_north)
            .field("exit_south", &self.exit_south)
            .field("exit_west", &self.exit_west)
            .field("exit_east", &self.exit_east)
            .field("restart_on_zap", &self.restart_on_zap)
            .field("message", &self.message)
            .field("enter_x", &self.enter_x)
            .field("enter_y", &self.enter_y)
            .field("time_limit", &self.time_limit)
            .field("stats", &self.stats)
            .finish()
    }
}

/// A status element on a ZZT board.
///
/// Stats provide additional data for specific tiles on the board. The majority of tiles will be
/// static, such as most of the board terrain (walls, water, empties, etc). But many things that
/// move, or any tile that needs state or code, will have an associated stat.
///
/// A stat's x/y coordinates are what link them to their associated tiles. Typically this is a 1:1
/// relationship: if there is an object tile at (12, 34), the board's stat list will usually have
/// exactly one entry that points to (12, 34). However, there are advanced use cases for having
/// multiple stats (or no stats!) in specific situations. So zztff does not enforce a 1:1
/// relationship---that is the responsibility of editing software that wants to be user friendly.
///
/// ZZT has a wide variety of tile types, called elements. The precise meanings of some of the
/// fields (particularly `p1`, `p2`, and `p3`) will vary depending on which tile the stat is pointed
/// at. You may want to consult a reference source, such as [the wiki articles on
/// elements](https://wiki.zzt.org/wiki/Element), for more info.
#[derive(Debug, Clone)]
pub struct Stat {
    /// X coordinate of the stat's tile (1-based).
    pub x: u8,
    /// Y coordinate of the stat's tile (1-based).
    pub y: u8,
    /// Horizontal step delta.
    ///
    /// This is usually -1, 0, or 1. Paired with `y_step`, the step deltas are used to encode
    /// direction or movement. For example, for transporters, this encodes which direction the
    /// transporter is facing, whereas for pushers, it encodes the direction of movement.
    ///
    /// Step values of (0, 0) or the four cardinal directions are the most common. Other step values
    /// are possible and are useful for advanced techniques, but care must be taken to ensure that
    /// they are safe; ZZT's bounds checking code assumes step distances will never exceed 1, so
    /// steps larger than that may go off-board and cause memory corruption.
    pub x_step: i16,
    /// Vertical step delta.
    pub y_step: i16,
    /// How often this stat is updated, in game ticks.
    ///
    /// Roughly speaking, stats update every 1/cycle game ticks. A cycle 1 object will run its code
    /// on every tick, a cycle 2 object runs every other tick, etc.
    pub cycle: i16,
    /// Element-specific parameter 1.
    pub p1: u8,
    /// Element-specific parameter 2.
    pub p2: u8,
    /// Element-specific parameter 3.
    pub p3: u8,
    /// Index of the following stat in a centipede chain, or -1 if none.
    ///
    /// Leader and follower do not need to be explicitly set if you are creating a world from
    /// scratch; if there are no existing links, ZZT will automatically link adjacent segments into
    /// a chain when the board first loads. These pointers are used mainly for preserving the state
    /// of saved games---but a world author could still use them if they needed a specific centipede
    /// layout for some reason.
    pub follower: i16,
    /// Index of the leading stat in a centipede chain, or -1 if none.
    pub leader: i16,
    /// The tile that is underneath this stat.
    ///
    /// This is often an empty tile, but it may be something else if the stat was placed on top of
    /// another terrain element (such as a fake wall/floor). ZZT uses this field to fill in the gap
    /// if this stat ever moves; tracking the `under` type allows stats to move over terrain without
    /// erasing it.
    ///
    /// Note that this field only holds a tile, i.e., not a stat. In general, ZZT assumes that a
    /// given coordinate can only be occupied by one stat at a time. For example, bullets may pass
    /// over terrain, but they cannot pass by each other; they destroy each other on collision.
    ///
    /// (Stats can be manually made to occupy the same x/y coordinates, but although this technique
    /// is known as "stat stacking", strictly speaking ZZT has no way to represent one stat being
    /// above or below another. The combination of stats in this way does not behave like two
    /// overlapping items.)
    pub under: Tile,
    /// Current position in an object's ZZT-OOP program.
    ///
    /// This is a byte offset in the file format, which corresponds to a character offset in the
    /// `String` representation. 0 starts from the top of the program and is the typical default.
    ///
    /// ZZT uses -1 to indicate that execution has halted; this value can be set from within ZZT
    /// using the `#end` command, but it can also be set directly in the file format to keep an
    /// object from running its code when the board is first loaded.
    ///
    /// ZZT has no call stack, so this is the main piece of execution state.
    pub instruction_pointer: i16,
    /// The stat's ZZT-OOP program, if any.
    ///
    /// If a stat doesn't have a program, this is set to the empty string. This field is only ever
    /// used by scrolls and objects, so it's pretty common for this to be empty.
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
            keys: Keys {
                blue: keys[0],
                green: keys[1],
                cyan: keys[2],
                red: keys[3],
                purple: keys[4],
                yellow: keys[5],
                white: keys[6],
            },
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
        result.push_bool(self.keys.blue);
        result.push_bool(self.keys.green);
        result.push_bool(self.keys.cyan);
        result.push_bool(self.keys.red);
        result.push_bool(self.keys.purple);
        result.push_bool(self.keys.yellow);
        result.push_bool(self.keys.white);
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
    /// Get the tile at the specified coordinates.
    ///
    /// This uses ZZT's 1-based coordinates, so (1, 1) is the top-left corner of the board.
    /// Panics if the coordinates lie outside the Board's 60x25 terrain area.
    pub fn tile(&self, x: usize, y: usize) -> Tile {
        assert!(
            x >= 1 && x <= 60 && y >= 1 && y <= 25,
            "tile coords out of range: ({x}, {y})"
        );
        self.tiles[(y - 1) * 60 + (x - 1)]
    }

    /// Set the tile at the specified coordinates.
    ///
    /// This uses ZZT's 1-based coordinates, so (1, 1) is the top-left corner of the board.
    /// Panics if the coordinates lie outside the Board's 60x25 terrain area.
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        assert!(
            x >= 1 && x <= 60 && y >= 1 && y <= 25,
            "tile coords out of range: ({x}, {y})"
        );
        self.tiles[(y - 1) * 60 + (x - 1)] = tile;
    }

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

    /// Format board terrain as two hex grids (elements, then colors).
    fn format_tiles(board: &Board) -> String {
        let mut out = String::new();
        for y in 1..=25 {
            for x in 1..=60 {
                out.push_str(&format!("{:02x}", board.tile(x, y).element));
            }
            out.push('\n');
        }
        out.push('\n');
        for y in 1..=25 {
            for x in 1..=60 {
                out.push_str(&format!("{:02x}", board.tile(x, y).color));
            }
            out.push('\n');
        }
        out.push('\n');
        out
    }

    #[test]
    fn tiles_snapshot() {
        let world = World::from_bytes(BYTES).unwrap();
        let mut all_tiles = String::new();
        for (i, board) in world.boards.iter().enumerate() {
            all_tiles.push_str(&format!("// board {}: {}\n", i, board.name));
            all_tiles.push_str(&format_tiles(board));
        }
        insta::assert_snapshot!(all_tiles);
    }
}
