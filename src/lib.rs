//! # Tools for reading/writing ZZT's file formats
//!
//! zztff is a library for working with [ZZT](https://en.wikipedia.org/wiki/ZZT) world and board files.
//! It is somewhat low-level; it allows control over every data field used by ZZT,
//! but creating new files requires some familiarity with the file format.
//!
//! zztff currently only supports the format used by ZZT 3.2, the last official release.
//! Super ZZT (the sequel) is not currently supported.
//!
//! ## String encoding
//!
//! zztff decodes all text data from extended ASCII into Unicode `String`s, because those are usually more convenient
//! for examining text and performing string manipuation. "Extended ASCII" is ambiguous as an encoding, so zztff assumes
//! [CP437](https://en.wikipedia.org/wiki/Code_page_437) for all conversions.
//! This is the de facto standard for most ZZT worlds and is what you want most of the time.
//!
//! If you have text that is not CP437, zztff will still read and write it just fine, and it is guaranteed to round-trip
//! back to disk unchanged. However, depending on the author's original encoding, foreign characters and dingbats
//! may display incorrectly.
//! (This is a difficult problem to solve in general, because ZZT formats do not specify their encodings, and if the
//! ZZT world was using a custom font, it can have characters without any Unicode equivalent.)
//!
//! Serialization will fail with [`EncodeError::EncodingError`] if you add a character to a String that doesn't have a
//! CP437 equivalent.
//! Serialization can also fail with [`EncodeError::StringTooLong`] if a String would exceed the maximum length of the
//! given text field in the file format.
//! (You can check the encoded length of a String by counting its codepoints, e.g., `s.chars().count()`;
//! zztff guarantees a 1:1 correspondence between codepoints in the String and bytes in the file format.)
//!
//! ## Examples
//!
//! ### Reading an existing .ZZT file
//!
//! ```rust,no_run
//! let bytes = std::fs::read("TOWN.ZZT")?;
//! let world = zztff::World::from_bytes(&bytes)?;
//! let first3: Vec<&str> = world.boards.iter().take(3).map(|b| b.name.as_str()).collect();
//! println!("Loaded world {:?}, {} boards.", world.name, world.boards.len());
//! println!("First 3 boards: {:?}", first3);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Assuming you have TOWN.ZZT, this loads the file and prints a brief summary:
//!
//! ```text
//! Loaded world "TOWN", 34 boards.
//! First 3 boards: ["Introduction Screen", "Room One", "Armory"]
//! ```
//!
//! ### Creating a new .ZZT file
//!
//! Creating a valid world file from scratch is more involved.
//! Worlds must contain at least one board, and we must populate that board with a player.
//! And to create a functioning player, we must create a corresponding stat to allow the player tile to move around.
//!
//! ```rust,no_run
//! use zztff::{World, Board, Tile, Stat, Element};
//!
//! let mut world = World::default();
//! world.name = "HELLO".into();    // should match stem of filename
//!
//! let mut board = Board::default();
//! board.set_tile(30, 12, Tile {   // place a player tile in the center, at (30, 12)
//!     element: Element::Player as u8,
//!     color: 0x1f,
//! });
//! board.stats.push(Stat {
//!     x: 30,                      // associate a stat with the player tile at (30, 12)
//!     y: 12,
//!     cycle: 1,
//!     ..Stat::default()
//! });
//! world.boards.push(board);
//!
//! let bytes = world.to_bytes()?;
//! std::fs::write("HELLO.ZZT", &bytes)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod elements;
mod errors;
mod text;
mod world;

pub use elements::Element;
pub use errors::{DecodeError, EncodeError};
pub use text::{decode_multiline, decode_oneline, encode_multiline, encode_oneline};
pub use world::{Board, Keys, Program, Stat, Tile, World};
