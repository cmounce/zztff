use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Officially supported ZZT element types.
///
/// These are all the elements for which ZZT 3.2 has handler code. In general,
/// though, the file format allows any `u8` as a tile type. And sometimes
/// out-of-range values see actual use:
///
/// - Some values don't crash ZZT right away, but instead trigger a different
///   bug where they appear as text colors besides the official seven.
/// - Some worlds intentionally crash ZZT for whatever reason.
/// - Forks of ZZT may take unused element IDs and repurpose them.
///
/// In order to provide flexibility around these cases, the `Board` struct
/// represents tile types with raw `u8`s, but this enum is still provided for
/// convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Element {
    Empty = 0,
    BoardEdge = 1,
    Messenger = 2,
    Monitor = 3,
    Player = 4,
    Ammo = 5,
    Torch = 6,
    Gem = 7,
    Key = 8,
    Door = 9,
    Scroll = 10,
    Passage = 11,
    Duplicator = 12,
    Bomb = 13,
    Energizer = 14,
    Star = 15,
    Clockwise = 16,
    Counter = 17,
    Bullet = 18,
    Water = 19,
    Forest = 20,
    Solid = 21,
    Normal = 22,
    Breakable = 23,
    Boulder = 24,
    SliderNs = 25,
    SliderEw = 26,
    Fake = 27,
    Invisible = 28,
    BlinkWall = 29,
    Transporter = 30,
    Line = 31,
    Ricochet = 32,
    BlinkRayH = 33,
    Bear = 34,
    Ruffian = 35,
    Object = 36,
    Slime = 37,
    Shark = 38,
    SpinningGun = 39,
    Pusher = 40,
    Lion = 41,
    Tiger = 42,
    BlinkRayV = 43,
    Head = 44,
    Segment = 45,
    // 46 is unused
    TextBlue = 47,
    TextGreen = 48,
    TextCyan = 49,
    TextRed = 50,
    TextPurple = 51,
    TextBrown = 52,
    TextBlack = 53,
}
