use std::fmt::Write;

use super::elements::{Element, element_name};
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
