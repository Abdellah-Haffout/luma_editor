use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasTransform {
    pub pan: Vec2,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CanvasTransform {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            min_zoom: 0.1,
            max_zoom: 5.0,
        }
    }
}

impl CanvasTransform {
    pub fn new(pan: Vec2, zoom: f32) -> Self {
        Self {
            pan,
            zoom,
            ..Default::default()
        }
    }

    /// Converts a point from window screen pixels to virtual infinite Canvas space.
    pub fn screen_to_canvas(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos - self.pan) / self.zoom
    }

    /// Converts a point from virtual Canvas space to window screen pixels.
    pub fn canvas_to_screen(&self, canvas_pos: Vec2) -> Vec2 {
        (canvas_pos * self.zoom) + self.pan
    }

    /// Pans the canvas by a screen-space delta.
    pub fn pan_by(&mut self, screen_delta: Vec2) {
        self.pan += screen_delta;
    }

    /// Zooms into/out of the canvas centered on a specific screen pivot (e.g. mouse cursor).
    /// Keeps the canvas coordinate under the mouse cursor stationary.
    pub fn zoom_at(&mut self, screen_pivot: Vec2, zoom_factor: f32) {
        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * zoom_factor).clamp(self.min_zoom, self.max_zoom);

        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        // pan_new = pivot - (pivot - pan_old) * (zoom_new / zoom_old)
        self.pan = screen_pivot - (screen_pivot - self.pan) * (new_zoom / old_zoom);
        self.zoom = new_zoom;
    }

    pub fn reset(&mut self) {
        self.pan = Vec2::ZERO;
        self.zoom = 1.0;
    }
}
