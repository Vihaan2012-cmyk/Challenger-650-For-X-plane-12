//! Placing controls from positions measured on a layout figure.
//!
//! Each panel's data is kept in the figure's own coordinates, so it can be
//! checked against the figure, with a scale that comes from something of
//! known size in the same figure.

use crate::part::{Scene, Surface};

use super::controls::{button, gauge, key, key_in, knob, knob_in, switchlight, toggle};

/// How a figure's coordinates map onto a surface.
#[derive(Clone, Copy)]
pub struct Figure {
    /// Figure coordinates of the point that sits at the surface's (u, v).
    pub origin: (f32, f32),
    /// Metres per figure unit.
    pub scale: f32,
}

impl Figure {
    /// Offset on the surface from the origin; figure y runs down.
    pub fn offset(&self, x: f32, y: f32) -> (f32, f32) {
        ((x - self.origin.0) * self.scale, (self.origin.1 - y) * self.scale)
    }
}

/// The controls on one panel, in figure coordinates. Knobs and gauges carry
/// a radius, keys a width and height, all in figure units.
#[derive(Default)]
pub struct Controls<'a> {
    pub toggles: &'a [(f32, f32)],
    pub switchlights: &'a [(f32, f32)],
    pub buttons: &'a [(f32, f32)],
    pub knobs: &'a [(f32, f32, f32)],
    pub gauges: &'a [(f32, f32, f32)],
    pub keys: &'a [(f32, f32, f32, f32)],
    /// Light grey keys and bright metal knobs, as on the flight control panel.
    pub pale: bool,
}

impl Controls<'_> {
    pub fn count(&self) -> usize {
        self.toggles.len() + self.switchlights.len() + self.buttons.len() + self.knobs.len() + self.gauges.len() + self.keys.len()
    }

    /// Adds the controls with the figure's origin at (u, v) on the surface.
    /// Each becomes a moving control with a dataref under `prefix`, numbered
    /// in the order listed.
    pub fn add(&self, scene: &mut Scene, surface: &Surface, figure: Figure, u: f32, v: f32, prefix: &str) {
        let at = |x: f32, y: f32| {
            let (du, dv) = figure.offset(x, y);
            (u + du, v + dv)
        };
        for (number, &(x, y)) in self.toggles.iter().enumerate() {
            scene.control(&format!("{prefix}/toggle_{:02}", number + 1));
            let (u, v) = at(x, y);
            toggle(scene, surface, u, v);
        }
        for (number, &(x, y)) in self.switchlights.iter().enumerate() {
            scene.control(&format!("{prefix}/switchlight_{:02}", number + 1));
            let (u, v) = at(x, y);
            switchlight(scene, surface, u, v);
        }
        for (number, &(x, y)) in self.buttons.iter().enumerate() {
            scene.control(&format!("{prefix}/button_{:02}", number + 1));
            let (u, v) = at(x, y);
            button(scene, surface, u, v, 0.006);
        }
        for (number, &(x, y, radius)) in self.knobs.iter().enumerate() {
            scene.control(&format!("{prefix}/knob_{:02}", number + 1));
            let (u, v) = at(x, y);
            if self.pale {
                knob_in(scene, surface, u, v, radius * figure.scale, [156, 158, 162]);
            } else {
                knob(scene, surface, u, v, radius * figure.scale);
            }
        }
        scene.naming = None;
        for &(x, y, radius) in self.gauges {
            let (u, v) = at(x, y);
            gauge(scene, surface, u, v, radius * figure.scale);
        }
        for (number, &(x, y, width, height)) in self.keys.iter().enumerate() {
            scene.control(&format!("{prefix}/key_{:02}", number + 1));
            let (u, v) = at(x, y);
            if self.pale {
                key_in(scene, surface, u, v, width * figure.scale, height * figure.scale, [178, 180, 184]);
            } else {
                key(scene, surface, u, v, width * figure.scale, height * figure.scale);
            }
        }
        scene.naming = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figure_y_runs_down_and_surface_v_runs_up() {
        let figure = Figure { origin: (100.0, 50.0), scale: 0.001 };
        let (u, v) = figure.offset(130.0, 70.0);
        assert!((u - 0.03).abs() < 1e-6 && (v + 0.02).abs() < 1e-6);
    }

    #[test]
    fn every_listed_control_is_built() {
        let controls = Controls { toggles: &[(0.0, 0.0), (40.0, 0.0)], knobs: &[(80.0, 0.0, 10.0)], ..Default::default() };
        assert_eq!(controls.count(), 3);
        let mut scene = Scene::default();
        let surface = Surface::new([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        controls.add(&mut scene, &surface, Figure { origin: (0.0, 0.0), scale: 0.001 }, 0.0, 0.0, "test/panel");
        let names: Vec<&str> = scene.moving.iter().map(|m| m.dataref.as_str()).collect();
        assert_eq!(names, ["test/panel/toggle_01", "test/panel/toggle_02", "test/panel/knob_01"]);
        assert!(scene.naming.is_none(), "nothing built afterwards is animated by accident");
        let (low, high) = scene.moving[2].pieces[0].mesh.bounds().unwrap();
        // The knob is fluted, so its widest point may fall just short of its radius.
        assert!((low[0] - 0.07).abs() < 0.0012 && (high[0] - 0.09).abs() < 0.0012);
    }
}
