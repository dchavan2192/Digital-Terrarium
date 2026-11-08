//! Camera with pan (right-click drag or WASD) and zoom (scroll wheel).
//!
//! The camera maps world coordinates to screen coordinates. The simulation
//! world is 1000×1000; the camera lets you zoom in to watch individual
//! creatures or zoom out to see the full terrarium.

use macroquad::prelude::*;

pub struct Camera {
    /// Center of the camera in world coordinates.
    pub center_x: f64,
    pub center_y: f64,
    /// Zoom level: pixels per world unit. Higher = more zoomed in.
    pub zoom: f64,
    /// Whether the user is currently dragging to pan.
    dragging: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl Camera {
    pub fn new(world_width: f64, world_height: f64) -> Self {
        Camera {
            center_x: world_width / 2.0,
            center_y: world_height / 2.0,
            zoom: 0.7, // start slightly zoomed out to see the full world
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }

    /// Process input: scroll to zoom, right-click drag or WASD to pan.
    /// `ui_width` is the width of the egui side panel so we don't pan when clicking on UI.
    pub fn update(&mut self, ui_width: f32) {
        let (mx, my) = mouse_position();

        // Scroll wheel zoom (only when mouse is over the viewport, not the UI panel)
        if mx > ui_width {
            let scroll = mouse_wheel().1;
            if scroll != 0.0 {
                let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
                self.zoom = (self.zoom * factor).clamp(0.1, 20.0);
            }
        }

        // Right-click drag to pan
        if is_mouse_button_pressed(MouseButton::Right) && mx > ui_width {
            self.dragging = true;
            self.last_mouse_x = mx;
            self.last_mouse_y = my;
        }
        if is_mouse_button_released(MouseButton::Right) {
            self.dragging = false;
        }
        if self.dragging {
            let dx = (mx - self.last_mouse_x) as f64 / self.zoom;
            let dy = (my - self.last_mouse_y) as f64 / self.zoom;
            self.center_x -= dx;
            self.center_y -= dy;
            self.last_mouse_x = mx;
            self.last_mouse_y = my;
        }

        // WASD keyboard pan
        let pan_speed = 5.0 / self.zoom;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            self.center_y -= pan_speed;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            self.center_y += pan_speed;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            self.center_x -= pan_speed;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            self.center_x += pan_speed;
        }
    }

    /// Convert world coordinates to screen pixel coordinates.
    pub fn world_to_screen(&self, wx: f64, wy: f64) -> (f32, f32) {
        let sw = screen_width() as f64;
        let sh = screen_height() as f64;
        let sx = (wx - self.center_x) * self.zoom + sw / 2.0;
        let sy = (wy - self.center_y) * self.zoom + sh / 2.0;
        (sx as f32, sy as f32)
    }

    /// Convert screen pixel coordinates to world coordinates.
    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f64, f64) {
        let sw = screen_width() as f64;
        let sh = screen_height() as f64;
        let wx = (sx as f64 - sw / 2.0) / self.zoom + self.center_x;
        let wy = (sy as f64 - sh / 2.0) / self.zoom + self.center_y;
        (wx, wy)
    }
}
