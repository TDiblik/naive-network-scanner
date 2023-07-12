// I want the window smaller for debugging
#[cfg(debug_assertions)]
pub const WORKSPACE_WINDOW_WIDTH: f32 = 880.0;
#[cfg(debug_assertions)]
pub const WORKSPACE_WINDOW_HEIGHT: f32 = 1080.0;

#[cfg(not(debug_assertions))]
pub const WORKSPACE_WINDOW_WIDTH: f32 = 1450.0;
#[cfg(not(debug_assertions))]
pub const WORKSPACE_WINDOW_HEIGHT: f32 = 1080.0;
