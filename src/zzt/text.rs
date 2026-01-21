use std::fmt::Write;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, multispace1, not_line_ending},
    combinator::{map, map_res, opt, recognize, value},
    multi::many0,
    sequence::pair,
};

use super::elements::{Element, element_name, resolve_alias};
use super::error::ParseError;
use super::parse::{Board, Program, Stat, Tile, World};

/// Write key = value if value != default.
macro_rules! kv {
    ($out:expr, $key:expr, $val:expr, $default:expr) => {
        if $val != $default {
            writeln!($out, "{} = {}", $key, $val).unwrap();
        }
    };
}

/// Write key = true if value is true.
macro_rules! kv_bool {
    ($out:expr, $key:expr, $val:expr) => {
        if $val {
            writeln!($out, "{} = true", $key).unwrap();
        }
    };
}

/// Write key = "value" if value != default.
macro_rules! kv_str {
    ($out:expr, $key:expr, $val:expr, $default:expr) => {
        if $val != $default {
            writeln!($out, "{} = {:?}", $key, $val).unwrap();
        }
    };
}

/// Convert a World to its text representation.
pub fn world_to_text(world: &World) -> String {
    let mut output = String::new();
    write_world_header(&mut output, world);

    for (i, board) in world.boards.iter().enumerate() {
        output.push('\n');
        write_board(&mut output, i, board);
    }

    output
}

/// Convert a standalone Board to its text representation.
pub fn board_to_text(board: &Board) -> String {
    let mut output = String::new();
    write_board(&mut output, 0, board);
    output
}

fn write_world_header(output: &mut String, world: &World) {
    output.push_str("[world]\n");
    writeln!(output, "name = {:?}", world.name).unwrap();

    kv!(output, "health", world.health, 100);
    kv!(output, "ammo", world.ammo, 0);
    kv!(output, "gems", world.gems, 0);
    kv!(output, "torches", world.torches, 0);
    kv!(output, "score", world.score, 0);

    let key_str = keys_to_string(&world.keys);
    if !key_str.is_empty() {
        writeln!(output, "keys = {:?}", key_str).unwrap();
    }

    kv!(output, "starting_board", world.starting_board, 0);
    kv_bool!(output, "saved_game", world.locked);

    // Print flags until we've printed all the non-empty ones.
    // If there are empty ones in between, they will be printed, e.g. ["foo", "", "bar"],
    // but the empty ones at the end will be omitted.
    let last_non_empty = world.flags.iter().rposition(|f| !f.is_empty());
    if let Some(last_idx) = last_non_empty {
        let flags = &world.flags[..=last_idx];
        write!(output, "flags = [").unwrap();
        for (i, flag) in flags.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            write!(output, "{:?}", flag).unwrap();
        }
        output.push_str("]\n");
    }

    kv!(output, "torch_cycles", world.torch_cycles, 0);
    kv!(output, "energizer_cycles", world.energizer_cycles, 0);
    kv!(output, "time", world.time, 0);
    kv!(output, "time_ticks", world.time_ticks, 0);
}

fn keys_to_string(keys: &[bool; 7]) -> String {
    let key_chars = ['b', 'g', 'c', 'r', 'p', 'y', 'w'];
    keys.iter()
        .zip(key_chars.iter())
        .filter(|(has_key, _)| **has_key)
        .map(|(_, c)| *c)
        .collect()
}

fn write_board(output: &mut String, index: usize, board: &Board) {
    writeln!(output, "[board {}]", index).unwrap();
    writeln!(output, "title = {:?}", board.name).unwrap();
    output.push('\n');

    write_terrain(output, &board.tiles);
    output.push('\n');

    kv!(output, "shots", board.max_shots, 0);
    kv_bool!(output, "dark", board.is_dark);
    kv!(output, "exit_n", board.exit_north, 0);
    kv!(output, "exit_s", board.exit_south, 0);
    kv!(output, "exit_e", board.exit_east, 0);
    kv!(output, "exit_w", board.exit_west, 0);
    kv_bool!(output, "reenter", board.restart_on_zap);
    kv!(output, "time_limit", board.time_limit, 0);
    kv!(output, "enter_x", board.enter_x, 0);
    kv!(output, "enter_y", board.enter_y, 0);
    kv_str!(output, "message", &board.message, "");

    for (i, stat) in board.stats.iter().enumerate() {
        let element = get_element_at(board, stat.x, stat.y);
        output.push('\n');
        write_stat(output, i, stat, element);
    }
}

fn write_terrain(output: &mut String, tiles: &[Tile]) {
    // Elements: 60x25 grid, element bytes as 2-digit hex
    for row in 0..25 {
        for col in 0..60 {
            let tile = &tiles[row * 60 + col];
            write!(output, "{:02x}", tile.element).unwrap();
        }
        output.push('\n');
    }

    output.push('\n');

    // Colors: 60x25 grid, color bytes as 2-digit hex
    for row in 0..25 {
        for col in 0..60 {
            let tile = &tiles[row * 60 + col];
            write!(output, "{:02x}", tile.color).unwrap();
        }
        output.push('\n');
    }
}

fn get_element_at(board: &Board, x: u8, y: u8) -> Option<u8> {
    if x == 0 || y == 0 || x > 60 || y > 25 {
        return None;
    }
    let index = ((y as usize - 1) * 60) + (x as usize - 1);
    board.tiles.get(index).map(|t| t.element)
}

fn write_stat(output: &mut String, index: usize, stat: &Stat, element: Option<u8>) {
    // Stat header with element type comment
    let element_comment = match element {
        Some(id) => element_name(id),
        None => "off-board".to_string(),
    };
    writeln!(output, "[stat {}] # {}", index, element_comment).unwrap();

    writeln!(output, "x = {}", stat.x).unwrap();
    writeln!(output, "y = {}", stat.y).unwrap();

    kv!(output, "cycle", stat.cycle, 0);
    kv!(output, "x_step", stat.x_step, 0);
    kv!(output, "y_step", stat.y_step, 0);

    if stat.under.element != 0 || stat.under.color != 0 {
        writeln!(
            output,
            "under = ({}, {})",
            stat.under.element, stat.under.color
        )
        .unwrap();
    }

    kv!(output, "follower", stat.follower, -1);
    kv!(output, "leader", stat.leader, -1);
    kv!(output, "instruction_pointer", stat.instruction_pointer, 0);

    // Parameters with element-specific aliases
    let elem = element.and_then(Element::from_u8);
    write_param(output, stat.p1, "p1", elem.and_then(|e| e.p1_alias()));
    write_param(output, stat.p2, "p2", elem.and_then(|e| e.p2_alias()));
    write_param(output, stat.p3, "p3", elem.and_then(|e| e.p3_alias()));

    // Program/code
    match &stat.program {
        Program::Own(code) if !code.is_empty() => {
            output.push_str("code = \"\"\"\n");
            output.push_str(code);
            if !code.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("\"\"\"\n");
        }
        Program::Bound(idx) => {
            writeln!(output, "bind = {}", idx).unwrap();
        }
        _ => {}
    }
}

fn write_param(output: &mut String, value: u8, generic_name: &str, alias: Option<&str>) {
    if value == 0 {
        return;
    }
    let name = alias.unwrap_or(generic_name);
    writeln!(output, "{} = {}", name, value).unwrap();
}

// ============================================================================
// Text Parsing
// ============================================================================

/// Parsed value from a key-value pair.
#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    StringArray(Vec<String>),
    Tuple2(u8, u8),
    TripleQuotedString(String),
}

/// Skip a comment (# to end of line).
fn comment(input: &str) -> IResult<&str, ()> {
    value((), pair(char('#'), not_line_ending)).parse(input)
}

/// Skip whitespace and comments.
fn ws(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(alt((value((), multispace1), comment))),
    ).parse(input)
}

/// Parse a section header like [world], [board 0], [stat 1].
/// Returns the optional index.
fn section_header<'a>(name: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, Option<usize>> {
    move |input: &'a str| {
        let (input, _) = ws(input)?;
        let (input, _) = char('[').parse(input)?;
        let (input, _) = tag(name).parse(input)?;
        let (input, idx) = opt((multispace1, map_res(digit1, |s: &str| s.parse())))
            .map(|opt| opt.map(|(_, n)| n))
            .parse(input)?;
        let (input, _) = char(']').parse(input)?;
        // Consume rest of line (including any comment)
        let (input, _) = opt(pair(take_while(|c| c == ' ' || c == '\t'), comment)).parse(input)?;
        let (input, _) = opt(line_ending).parse(input)?;
        Ok((input, idx))
    }
}

/// Parse an identifier (key name).
fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)
}

/// Parse a quoted string value (handles escape sequences).
fn quoted_string(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"').parse(input)?;
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut consumed = 0;

    loop {
        match chars.next() {
            Some('"') => {
                consumed += 1;
                break;
            }
            Some('\\') => {
                consumed += 1;
                match chars.next() {
                    Some('n') => {
                        result.push('\n');
                        consumed += 1;
                    }
                    Some('r') => {
                        result.push('\r');
                        consumed += 1;
                    }
                    Some('t') => {
                        result.push('\t');
                        consumed += 1;
                    }
                    Some('\\') => {
                        result.push('\\');
                        consumed += 1;
                    }
                    Some('"') => {
                        result.push('"');
                        consumed += 1;
                    }
                    Some(c) => {
                        // Unknown escape, keep as-is
                        result.push('\\');
                        result.push(c);
                        consumed += 1;
                    }
                    None => break,
                }
            }
            Some(c) => {
                result.push(c);
                consumed += c.len_utf8();
            }
            None => break,
        }
    }

    Ok((&input[consumed..], result))
}

/// Parse a triple-quoted string ("""...""").
fn triple_quoted_string(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("\"\"\"").parse(input)?;
    let (input, _) = opt(line_ending).parse(input)?;
    let (input, content) = take_until("\"\"\"").parse(input)?;
    let (input, _) = tag("\"\"\"").parse(input)?;
    // Trim trailing newline from content if present
    let content = content.strip_suffix('\n').unwrap_or(content);
    Ok((input, content.to_string()))
}

/// Parse an integer (possibly negative).
fn integer(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize(pair(opt(char('-')), digit1)),
        |s: &str| s.parse(),
    ).parse(input)
}

/// Parse a boolean value.
fn boolean(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("true")), value(false, tag("false")))).parse(input)
}

/// Parse a tuple like (element, color).
fn tuple2(input: &str) -> IResult<&str, (u8, u8)> {
    let (input, _) = char('(').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, a) = map_res(digit1, |s: &str| s.parse::<u8>()).parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char(',').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, b) = map_res(digit1, |s: &str| s.parse::<u8>()).parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char(')').parse(input)?;
    Ok((input, (a, b)))
}

/// Parse a string array like ["foo", "bar", ""].
fn string_array(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = char('[').parse(input)?;
    let (input, _) = multispace0.parse(input)?;

    let mut items = Vec::new();
    let mut input = input;

    // Check for empty array
    if let Ok((next, _)) = char::<_, nom::error::Error<&str>>(']').parse(input) {
        return Ok((next, items));
    }

    // Parse first item
    let (next, first) = quoted_string(input)?;
    items.push(first);
    input = next;

    // Parse remaining items
    loop {
        let (next, _) = multispace0.parse(input)?;
        if let Ok((next, _)) = char::<_, nom::error::Error<&str>>(']').parse(next) {
            return Ok((next, items));
        }
        let (next, _) = char(',').parse(next)?;
        let (next, _) = multispace0.parse(next)?;
        let (next, item) = quoted_string(next)?;
        items.push(item);
        input = next;
    }
}

/// Parse a value (any type).
fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        map(triple_quoted_string, Value::TripleQuotedString),
        map(string_array, Value::StringArray),
        map(quoted_string, Value::String),
        map(tuple2, |(a, b)| Value::Tuple2(a, b)),
        map(boolean, Value::Bool),
        map(integer, Value::Int),
    )).parse(input)
}

/// Parse a key = value pair.
fn parse_key_value(input: &str) -> IResult<&str, (&str, Value)> {
    let (input, _) = ws(input)?;
    let (input, key) = identifier(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, value) = parse_value(input)?;
    // Consume rest of line (including any comment)
    let (input, _) = opt(pair(take_while(|c| c == ' ' || c == '\t'), comment)).parse(input)?;
    let (input, _) = opt(line_ending).parse(input)?;
    Ok((input, (key, value)))
}

/// Parse 25 rows of hex data (60 columns each, 2 hex digits per value).
fn parse_hex_grid(input: &str) -> Result<(&str, Vec<u8>), ParseError> {
    let mut result = Vec::with_capacity(1500);
    let mut input = input;

    for _ in 0..25 {
        // Skip whitespace/comments before the row
        let (next, _) = ws(input).map_err(|e| ParseError::TextParseError {
            message: format!("hex grid whitespace: {:?}", e),
        })?;
        input = next;

        // Parse 60 hex pairs
        for _ in 0..60 {
            if input.len() < 2 {
                return Err(ParseError::InvalidHex("unexpected end of hex data".to_string()));
            }
            let hex_str = &input[..2];
            let byte = u8::from_str_radix(hex_str, 16)
                .map_err(|_| ParseError::InvalidHex(hex_str.to_string()))?;
            result.push(byte);
            input = &input[2..];
        }

        // Consume newline if present
        if input.starts_with('\n') {
            input = &input[1..];
        } else if input.starts_with("\r\n") {
            input = &input[2..];
        }
    }

    Ok((input, result))
}

/// Parse terrain: element grid followed by color grid.
fn parse_terrain(input: &str) -> Result<(&str, Vec<Tile>), ParseError> {
    let (input, elements) = parse_hex_grid(input)?;
    let (input, colors) = parse_hex_grid(input)?;

    let tiles: Vec<Tile> = elements
        .into_iter()
        .zip(colors)
        .map(|(element, color)| Tile { element, color })
        .collect();

    Ok((input, tiles))
}

/// Parse keys string like "bgcr" into bool array.
fn parse_keys(s: &str) -> [bool; 7] {
    let key_chars = ['b', 'g', 'c', 'r', 'p', 'y', 'w'];
    let mut keys = [false; 7];
    for (i, c) in key_chars.iter().enumerate() {
        keys[i] = s.contains(*c);
    }
    keys
}

/// Check if the next non-whitespace is a section header.
fn peek_section(input: &str) -> bool {
    let trimmed = input.trim_start();
    trimmed.starts_with('[')
}

/// Check if input is at end or only has whitespace/comments.
fn at_end(input: &str) -> bool {
    match ws(input) {
        Ok((rest, _)) => rest.is_empty(),
        Err(_) => false,
    }
}

/// Parse the [world] section properties.
fn parse_world_section(input: &str) -> Result<(&str, World), ParseError> {
    let (input, _) = section_header("world")(input).map_err(|e| ParseError::TextParseError {
        message: format!("expected [world] section: {:?}", e),
    })?;

    let mut world = World {
        health: 100,
        ..Default::default()
    };

    let mut input = input;
    while !at_end(input) && !peek_section(input) {
        let (next, (key, value)) =
            parse_key_value(input).map_err(|e| ParseError::TextParseError {
                message: format!("world property: {:?}", e),
            })?;
        input = next;

        match key {
            "name" => {
                if let Value::String(s) = value {
                    world.name = s;
                }
            }
            "health" => {
                if let Value::Int(n) = value {
                    world.health = n as i16;
                }
            }
            "ammo" => {
                if let Value::Int(n) = value {
                    world.ammo = n as i16;
                }
            }
            "gems" => {
                if let Value::Int(n) = value {
                    world.gems = n as i16;
                }
            }
            "torches" => {
                if let Value::Int(n) = value {
                    world.torches = n as i16;
                }
            }
            "score" => {
                if let Value::Int(n) = value {
                    world.score = n as i16;
                }
            }
            "keys" => {
                if let Value::String(s) = value {
                    world.keys = parse_keys(&s);
                }
            }
            "starting_board" => {
                if let Value::Int(n) = value {
                    world.starting_board = n as i16;
                }
            }
            "saved_game" => {
                if let Value::Bool(b) = value {
                    world.locked = b;
                }
            }
            "flags" => {
                if let Value::StringArray(arr) = value {
                    for (i, flag) in arr.into_iter().take(10).enumerate() {
                        world.flags[i] = flag;
                    }
                }
            }
            "torch_cycles" => {
                if let Value::Int(n) = value {
                    world.torch_cycles = n as i16;
                }
            }
            "energizer_cycles" => {
                if let Value::Int(n) = value {
                    world.energizer_cycles = n as i16;
                }
            }
            "time" => {
                if let Value::Int(n) = value {
                    world.time = n as i16;
                }
            }
            "time_ticks" => {
                if let Value::Int(n) = value {
                    world.time_ticks = n as i16;
                }
            }
            _ => {} // Ignore unknown keys
        }
    }

    Ok((input, world))
}

/// Get element at (x, y) from tiles array.
fn element_at(tiles: &[Tile], x: u8, y: u8) -> Option<u8> {
    if x == 0 || y == 0 || x > 60 || y > 25 {
        return None;
    }
    let index = ((y as usize - 1) * 60) + (x as usize - 1);
    tiles.get(index).map(|t| t.element)
}

/// Parse a [stat N] section.
fn parse_stat_section<'a>(input: &'a str, tiles: &[Tile]) -> Result<(&'a str, Stat), ParseError> {
    let (input, _) = section_header("stat")(input).map_err(|e| ParseError::TextParseError {
        message: format!("expected [stat] section: {:?}", e),
    })?;

    // First pass: collect all key-value pairs
    let mut pairs: Vec<(&str, Value)> = Vec::new();
    let mut input = input;
    while !at_end(input) && !peek_section(input) {
        let (next, pair) = parse_key_value(input).map_err(|e| ParseError::TextParseError {
            message: format!("stat property: {:?}", e),
        })?;
        pairs.push(pair);
        input = next;
    }

    // Extract x, y first to determine element type
    let mut x: u8 = 0;
    let mut y: u8 = 0;
    for (key, value) in &pairs {
        match *key {
            "x" => {
                if let Value::Int(n) = value {
                    x = *n as u8;
                }
            }
            "y" => {
                if let Value::Int(n) = value {
                    y = *n as u8;
                }
            }
            _ => {}
        }
    }

    let element = element_at(tiles, x, y).and_then(Element::from_u8);

    // Build stat with defaults
    let mut stat = Stat {
        x,
        y,
        x_step: 0,
        y_step: 0,
        cycle: 0,
        p1: 0,
        p2: 0,
        p3: 0,
        follower: -1,
        leader: -1,
        under: Tile {
            element: 0,
            color: 0,
        },
        instruction_pointer: 0,
        program: Program::Own(String::new()),
    };

    // Second pass: apply all properties
    for (key, value) in pairs {
        match key {
            "x" | "y" => {} // Already handled
            "cycle" => {
                if let Value::Int(n) = value {
                    stat.cycle = n as i16;
                }
            }
            "x_step" => {
                if let Value::Int(n) = value {
                    stat.x_step = n as i16;
                }
            }
            "y_step" => {
                if let Value::Int(n) = value {
                    stat.y_step = n as i16;
                }
            }
            "under" => {
                if let Value::Tuple2(e, c) = value {
                    stat.under = Tile {
                        element: e,
                        color: c,
                    };
                }
            }
            "follower" => {
                if let Value::Int(n) = value {
                    stat.follower = n as i16;
                }
            }
            "leader" => {
                if let Value::Int(n) = value {
                    stat.leader = n as i16;
                }
            }
            "instruction_pointer" => {
                if let Value::Int(n) = value {
                    stat.instruction_pointer = n as i16;
                }
            }
            "p1" => {
                if let Value::Int(n) = value {
                    stat.p1 = n as u8;
                }
            }
            "p2" => {
                if let Value::Int(n) = value {
                    stat.p2 = n as u8;
                }
            }
            "p3" => {
                if let Value::Int(n) = value {
                    stat.p3 = n as u8;
                }
            }
            "code" => {
                if let Value::TripleQuotedString(s) = value {
                    stat.program = Program::Own(s);
                }
            }
            "bind" => {
                if let Value::Int(n) = value {
                    stat.program = Program::Bound(n as u16);
                }
            }
            other => {
                // Check if it's a parameter alias
                if let Some(param_num) = resolve_alias(other, element) {
                    if let Value::Int(n) = value {
                        match param_num {
                            1 => stat.p1 = n as u8,
                            2 => stat.p2 = n as u8,
                            3 => stat.p3 = n as u8,
                            _ => {}
                        }
                    }
                }
                // Ignore unknown keys
            }
        }
    }

    Ok((input, stat))
}

/// Parse a [board N] section.
fn parse_board_section(input: &str) -> Result<(&str, Board), ParseError> {
    let (input, _) = section_header("board")(input).map_err(|e| ParseError::TextParseError {
        message: format!("expected [board] section: {:?}", e),
    })?;

    // Parse title first (required before terrain)
    let (input, (_, title_value)) =
        parse_key_value(input).map_err(|e| ParseError::TextParseError {
            message: format!("expected board title: {:?}", e),
        })?;

    let title = match title_value {
        Value::String(s) => s,
        _ => String::new(),
    };

    // Parse terrain
    let (input, tiles) = parse_terrain(input)?;

    // Parse board properties
    let mut board = Board {
        name: title,
        tiles,
        max_shots: 0,
        is_dark: false,
        exit_north: 0,
        exit_south: 0,
        exit_west: 0,
        exit_east: 0,
        restart_on_zap: false,
        message: String::new(),
        enter_x: 0,
        enter_y: 0,
        time_limit: 0,
        stats: Vec::new(),
    };

    let mut input = input;

    // Parse remaining properties until we hit a stat section or end
    while !at_end(input) && !peek_section(input) {
        let (next, (key, value)) =
            parse_key_value(input).map_err(|e| ParseError::TextParseError {
                message: format!("board property: {:?}", e),
            })?;
        input = next;

        match key {
            "shots" => {
                if let Value::Int(n) = value {
                    board.max_shots = n as u8;
                }
            }
            "dark" => {
                if let Value::Bool(b) = value {
                    board.is_dark = b;
                }
            }
            "exit_n" => {
                if let Value::Int(n) = value {
                    board.exit_north = n as u8;
                }
            }
            "exit_s" => {
                if let Value::Int(n) = value {
                    board.exit_south = n as u8;
                }
            }
            "exit_e" => {
                if let Value::Int(n) = value {
                    board.exit_east = n as u8;
                }
            }
            "exit_w" => {
                if let Value::Int(n) = value {
                    board.exit_west = n as u8;
                }
            }
            "reenter" => {
                if let Value::Bool(b) = value {
                    board.restart_on_zap = b;
                }
            }
            "time_limit" => {
                if let Value::Int(n) = value {
                    board.time_limit = n as i16;
                }
            }
            "enter_x" => {
                if let Value::Int(n) = value {
                    board.enter_x = n as u8;
                }
            }
            "enter_y" => {
                if let Value::Int(n) = value {
                    board.enter_y = n as u8;
                }
            }
            "message" => {
                if let Value::String(s) = value {
                    board.message = s;
                }
            }
            _ => {} // Ignore unknown keys
        }
    }

    // Parse stats
    while !at_end(input) {
        // Check if next section is a stat
        let trimmed = input.trim_start();
        if !trimmed.starts_with("[stat") {
            break;
        }
        let (next, stat) = parse_stat_section(input, &board.tiles)?;
        board.stats.push(stat);
        input = next;
    }

    Ok((input, board))
}

/// Convert text to a World.
pub fn text_to_world(text: &str) -> Result<World, ParseError> {
    let (input, mut world) = parse_world_section(text)?;

    let mut input = input;
    while !at_end(input) {
        let (next, board) = parse_board_section(input)?;
        world.boards.push(board);
        input = next;
    }

    Ok(world)
}

/// Convert text to a Board.
pub fn text_to_board(text: &str) -> Result<Board, ParseError> {
    let (_, board) = parse_board_section(text)?;
    Ok(board)
}
