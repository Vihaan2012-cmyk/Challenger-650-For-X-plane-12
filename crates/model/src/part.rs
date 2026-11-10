//! Building blocks: coloured parts, boxes, and flat surfaces that units are
//! mounted on.

use crate::labels::Label;
use crate::mesh::{add, cross, normalised, scaled, Mesh, Vec3, Vertex};

pub struct Part {
    pub name: &'static str,
    pub colour: [u8; 3],
    pub mesh: Mesh,
}

/// How a control moves as its dataref goes from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    /// Turns about an axis through the pivot, between two angles in degrees.
    Rotate { axis: Vec3, degrees: (f32, f32) },
    /// Slides along a direction, between two distances in metres.
    Slide { along: Vec3, metres: (f32, f32) },
}

/// How the crew work a control with the mouse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Manipulator {
    /// Click to flip between off and on.
    Toggle,
    /// Held down while pressed.
    Push,
    /// Dragged along its travel.
    Drag,
}

/// A control that moves: its dataref, how it moves, and its pieces.
pub struct Moving {
    pub dataref: String,
    pub pivot: Vec3,
    pub motion: Motion,
    pub manipulator: Manipulator,
    pub pieces: Vec<Part>,
}

#[derive(Default)]
pub struct Scene {
    pub parts: Vec<Part>,
    pub labels: Vec<Label>,
    pub moving: Vec<Moving>,
    /// The dataref of the control being built, if it is to move.
    pub naming: Option<String>,
}

impl Scene {
    /// Adds geometry to the part of that name, creating it if needed.
    pub fn add(&mut self, name: &'static str, colour: [u8; 3], mesh: Mesh) {
        match self.parts.iter_mut().find(|p| p.name == name && p.colour == colour) {
            Some(part) => part.mesh.append(&mesh),
            None => self.parts.push(Part { name, colour, mesh }),
        }
    }

    /// Names the control about to be built, so that its moving pieces are
    /// kept apart and animated by this dataref.
    pub fn control(&mut self, dataref: &str) {
        self.naming = Some(dataref.to_string());
    }

    /// Adds a piece that moves with the control being built; if no control
    /// has been named, it is added as ordinary fixed geometry.
    pub fn add_moving(&mut self, name: &'static str, colour: [u8; 3], mesh: Mesh, pivot: Vec3, motion: Motion, manipulator: Manipulator) {
        let Some(dataref) = self.naming.clone() else {
            return self.add(name, colour, mesh);
        };
        if self.moving.last().map(|m| m.dataref != dataref).unwrap_or(true) {
            self.moving.push(Moving { dataref, pivot, motion, manipulator, pieces: Vec::new() });
        }
        if let Some(control) = self.moving.last_mut() {
            control.pieces.push(Part { name, colour, mesh });
        }
    }

    /// Every piece of geometry, fixed and moving.
    pub fn everything(&self) -> Vec<&Part> {
        self.parts.iter().chain(self.moving.iter().flat_map(|m| &m.pieces)).collect()
    }

    /// Adds a backlit white legend.
    pub fn legend(&mut self, surface: &Surface, u: f32, v: f32, height: f32, proud: f32, text: &str) {
        self.labels.push(Label::on(surface, u, v, height, proud, text, [238, 238, 232], true));
    }

    /// Adds a printed legend that does not light up.
    pub fn placard(&mut self, surface: &Surface, u: f32, v: f32, height: f32, proud: f32, text: &str, colour: [u8; 3]) {
        self.labels.push(Label::on(surface, u, v, height, proud, text, colour, false));
    }

    pub fn part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name == name)
    }

    pub fn merged(&self) -> Mesh {
        let mut all = Mesh::default();
        for part in self.everything() {
            all.append(&part.mesh);
        }
        all
    }
}

pub const UPRIGHT: [Vec3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Axes leaning about the left-right axis; a positive angle moves the top
/// towards the nose.
pub fn pitched(degrees: f32) -> [Vec3; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [[1.0, 0.0, 0.0], [0.0, cos, -sin], [0.0, sin, cos]]
}

/// A box with the given centre, axes and half sizes along those axes.
pub fn block(centre: Vec3, axes: [Vec3; 3], half: [f32; 3]) -> Mesh {
    let mut mesh = Mesh::default();
    for axis in 0..3 {
        let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
        for sign in [-1.0, 1.0] {
            let normal = scaled(axes[axis], sign);
            let middle = add(centre, scaled(axes[axis], sign * half[axis]));
            let corner = |sa: f32, sb: f32| Vertex {
                position: add(add(middle, scaled(axes[a], sa * half[a])), scaled(axes[b], sb * half[b])),
                normal,
                uv: [(sa + 1.0) / 2.0, (sb + 1.0) / 2.0],
            };
            mesh.quad([corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)]);
        }
    }
    mesh
}

/// A flat face that units are mounted on. `right` and `up` are as the crew
/// see them when reading the face; it faces along `right` x `up`.
#[derive(Clone, Copy)]
pub struct Surface {
    pub origin: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Surface {
    pub fn new(origin: Vec3, right: Vec3, up: Vec3) -> Surface {
        Surface { origin, right: normalised(right), up: normalised(up) }
    }

    pub fn normal(&self) -> Vec3 {
        cross(self.right, self.up)
    }

    pub fn point(&self, u: f32, v: f32) -> Vec3 {
        add(add(self.origin, scaled(self.right, u)), scaled(self.up, v))
    }

    fn axes(&self) -> [Vec3; 3] {
        [self.right, self.up, self.normal()]
    }

    /// A unit of the given face size centred at (u, v), standing `proud` of
    /// the surface.
    pub fn unit(&self, u: f32, v: f32, width: f32, height: f32, proud: f32) -> Mesh {
        let centre = add(self.point(u, v), scaled(self.normal(), proud / 2.0));
        block(centre, self.axes(), [width / 2.0, height / 2.0, proud / 2.0])
    }

    /// The body behind the surface: a slab of the given size centred at
    /// (u, v) with its front face on the surface.
    pub fn slab(&self, u: f32, v: f32, width: f32, height: f32, thickness: f32) -> Mesh {
        let centre = add(self.point(u, v), scaled(self.normal(), -thickness / 2.0));
        block(centre, self.axes(), [width / 2.0, height / 2.0, thickness / 2.0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_is_closed_and_faces_outwards() {
        let mesh = block([1.0, 2.0, 3.0], pitched(20.0), [0.5, 0.25, 0.125]);
        assert_eq!(mesh.triangle_count(), 12);
        assert!(mesh.faces_agree_with_normals());
        for v in &mesh.vertices {
            let outwards = crate::mesh::sub(v.position, [1.0, 2.0, 3.0]);
            assert!(crate::mesh::dot(outwards, v.normal) > 0.0);
        }
    }

    #[test]
    fn a_unit_sits_on_the_side_the_surface_faces() {
        // A panel facing aft, like the instrument panel.
        let panel = Surface::new([0.0, 0.0, 5.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(panel.normal(), [0.0, 0.0, 1.0]);
        let (low, high) = panel.unit(0.1, 0.2, 0.3, 0.2, 0.01).bounds().unwrap();
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        assert!(close(low[0], -0.05) && close(high[0], 0.25));
        assert!(close(low[1], 0.1) && close(high[1], 0.3));
        assert!(close(low[2], 5.0) && close(high[2], 5.01));
        let (low, high) = panel.slab(0.0, 0.0, 1.0, 1.0, 0.05).bounds().unwrap();
        assert!(close(low[2], 4.95) && close(high[2], 5.0));
    }

    #[test]
    fn parts_of_the_same_name_are_merged() {
        let mut scene = Scene::default();
        scene.add("a", [1, 2, 3], block([0.0; 3], UPRIGHT, [1.0; 3]));
        scene.add("a", [1, 2, 3], block([5.0; 3], UPRIGHT, [1.0; 3]));
        scene.add("b", [1, 2, 3], block([0.0; 3], UPRIGHT, [1.0; 3]));
        assert_eq!(scene.parts.len(), 2);
        assert_eq!(scene.part("a").unwrap().mesh.triangle_count(), 24);
        assert_eq!(scene.merged().triangle_count(), 36);
    }

    #[test]
    fn pieces_added_under_a_name_move_together_and_others_stay_fixed() {
        let turn = Motion::Rotate { axis: [0.0, 0.0, 1.0], degrees: (0.0, 300.0) };
        let mut scene = Scene::default();
        scene.add_moving("knobs", [1, 2, 3], block([0.0; 3], UPRIGHT, [1.0; 3]), [0.0; 3], turn, Manipulator::Drag);
        assert_eq!((scene.parts.len(), scene.moving.len()), (1, 0), "unnamed pieces are fixed");
        scene.control("test/knob_a");
        scene.add_moving("knobs", [1, 2, 3], block([0.0; 3], UPRIGHT, [1.0; 3]), [0.0; 3], turn, Manipulator::Drag);
        scene.add_moving("knob pointers", [9, 9, 9], block([0.0; 3], UPRIGHT, [1.0; 3]), [0.0; 3], turn, Manipulator::Drag);
        scene.control("test/knob_b");
        scene.add_moving("knobs", [1, 2, 3], block([0.0; 3], UPRIGHT, [1.0; 3]), [0.0; 3], turn, Manipulator::Drag);
        assert_eq!(scene.moving.len(), 2);
        assert_eq!(scene.moving[0].pieces.len(), 2);
        assert_eq!(scene.everything().len(), 4);
        assert_eq!(scene.merged().triangle_count(), 48);
    }
}
