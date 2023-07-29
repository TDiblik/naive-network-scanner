use super::ip::Port;

// I want the window smaller for debugging
#[cfg(debug_assertions)]
pub const WORKSPACE_WINDOW_WIDTH: f32 = 880.0;
#[cfg(debug_assertions)]
pub const WORKSPACE_WINDOW_HEIGHT: f32 = 1080.0;

#[cfg(not(debug_assertions))]
pub const WORKSPACE_WINDOW_WIDTH: f32 = 1450.0;
#[cfg(not(debug_assertions))]
pub const WORKSPACE_WINDOW_HEIGHT: f32 = 1080.0;

pub const DEFAULT_WINDOW_STARTING_POS: eframe::epaint::Pos2 = eframe::epaint::Pos2 {
    x: WORKSPACE_WINDOW_WIDTH / 2.0 - 150.0,
    y: WORKSPACE_WINDOW_HEIGHT / 2.0 - 150.0,
};

#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";

pub const DEFAULT_SPACER: f32 = 5.0;
pub const ACTION_SPACER: f32 = 10.0;

pub const TRASH_ICON: &str = "🗑";

pub const MOST_COMMON_PORTS: [Port; 3] = [20, 21, 22];
// Fill with data from https://www.stationx.net/common-ports-cheat-sheet/
pub const ALL_COMMON_PORTS_LENGHT: usize = 43;
pub const ALL_COMMON_PORTS: [(Port, &str); ALL_COMMON_PORTS_LENGHT] = [
    (0, "telnet"),
    (0, "ssh"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (999, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
    (0, "your_mom"),
];

pub const PORT_FUZZING_COMMANDS: [&[u8]; 3] = [b"info\n", b"help\n", b"version\n"];
