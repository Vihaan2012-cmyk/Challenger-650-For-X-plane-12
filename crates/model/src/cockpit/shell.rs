//! The inside of the fuselage around the flight compartment: wall, glazing,
//! floor and the bulkheads at each end.
//!
//! The back of the compartment is the documented constant section. Ahead of
//! that the nose narrows and the roof comes down. No drawing we hold gives
//! that shape, so the roof line and `width_scale` are estimates, fitted to
//! the documented lines of sight from the eye: 15.5 degrees down over the
//! nose and 27 degrees up under the top of the windshield. The glazing is
//! whatever wall lies inside the documented vision angles.

use crate::mesh::{normalised, Mesh, Vec3, Vertex};
use crate::mesh::{add, scaled};
use crate::part::Scene;
use crate::shapes::tube;
use crate::stations::{cabin_radius, floor_y, station_z, FS_FLIGHT_COMPARTMENT_BULKHEAD, FS_FORWARD_PRESSURE_BULKHEAD, METRES_PER_INCH};

const ARC_SEGMENTS: usize = 288;
const SLICES: usize = 120;

const WALL: [u8; 3] = [204, 204, 200];
const GLAZING: [u8; 3] = [150, 196, 226];
const FLOOR: [u8; 3] = [112, 110, 106];
const BULKHEAD: [u8; 3] = [190, 190, 186];

/// Documented lines of sight from the eye, degrees.
pub const VISION_DOWN_OVER_NOSE: f32 = 15.5;
pub const VISION_UP: f32 = 27.0;
pub const VISION_AFT_LIMIT: f32 = 135.0;

/// Station where the windshield meets the roof, from the three-view drawing.
pub const FS_WINDSHIELD_TOP: f32 = 230.0;
/// Station from which the section is full width. Estimated.
const FS_FULL_WIDTH: f32 = 269.0;

/// Where the eyes are, so the shell can be fitted around the documented
/// lines of sight.
#[derive(Clone, Copy)]
pub struct Eye {
    pub offset: f32,
    pub y: f32,
    pub station: f32,
}

fn station_of(z: f32) -> f32 {
    z / METRES_PER_INCH
}

/// The nose, traced by measuring the pixels of the side and plan views of
/// the manufacturer's three-view drawing. The drawing's scale is confirmed
/// by the fuselage measuring 107 inches across on it against a documented
/// 106. Its stations are found from the nose being at station 141, which
/// two separate features give: the side window ends at the bulkhead
/// documented at station 280, and the entrance door starts just aft of
/// station 295, where the training guide puts a junction box "forward of
/// the main entrance door".
///
/// Height of the inside of the roof above the centreline, metres, taken as
/// an inch and a half inside the outer line and no higher than the cabin.
const ROOF: [(f32, f32); 14] = [
    (202.75, 0.330), (209.0, 0.417), (215.0, 0.500), (221.0, 0.582), (227.0, 0.665), (233.0, 0.775), (239.0, 0.871),
    (245.0, 0.968), (251.0, 1.024), (257.0, 1.105), (263.0, 1.146), (269.0, 1.189), (275.0, 1.217), (280.0, 1.240),
];
/// Width as a fraction of the constant section.
const WIDTH: [(f32, f32); 14] = [
    (202.75, 0.737), (209.0, 0.766), (215.0, 0.791), (221.0, 0.832), (227.0, 0.852), (233.0, 0.873), (239.0, 0.892),
    (245.0, 0.933), (251.0, 0.953), (257.0, 0.964), (263.0, 0.974), (269.0, 0.994), (275.0, 0.998), (280.0, 1.0),
];

/// A traced line read off at a station, straight between its points.
fn along(line: &[(f32, f32)], station: f32) -> f32 {
    let last = line.len() - 1;
    let station = station.clamp(line[0].0, line[last].0);
    let i = (0..last).find(|i| station <= line[i + 1].0).unwrap_or(last - 1);
    let t = (station - line[i].0) / (line[i + 1].0 - line[i].0);
    line[i].1 + (line[i + 1].1 - line[i].1) * t
}

/// How wide the section is at a station, as a fraction of the constant
/// section.
pub fn width_scale(station: f32) -> f32 {
    along(&WIDTH, station)
}

impl Eye {
    /// How high the roof is above the centreline at a station, as a fraction
    /// of the constant section.
    pub fn roof_scale(&self, station: f32) -> f32 {
        along(&ROOF, station) / cabin_radius()
    }

    /// Height of the roof above the centreline at a point across the cabin.
    pub fn roof_y(&self, x: f32, z: f32) -> f32 {
        let station = station_of(z);
        let across = (x / (width_scale(station) * cabin_radius())).clamp(-1.0, 1.0);
        self.roof_scale(station) * cabin_radius() * (1.0 - across * across).sqrt()
    }

    /// True when a point is inside the flight compartment.
    pub fn inside(&self, p: Vec3) -> bool {
        let station = station_of(p[2]);
        if !(FS_FORWARD_PRESSURE_BULKHEAD - 1e-3..=FS_FLIGHT_COMPARTMENT_BULKHEAD + 1e-3).contains(&station) || p[1] < floor_y() - 1e-4 {
            return false;
        }
        let x = p[0] / (width_scale(station) * cabin_radius());
        let y = p[1] / (cabin_radius() * if p[1] > 0.0 { self.roof_scale(station) } else { 1.0 });
        x * x + y * y <= 1.0 + 1e-4
    }

    /// Whether a point on the wall is window: inside the documented vision
    /// angles of the eye on that side.
    fn sees_out_through(&self, p: Vec3) -> bool {
        let outboard = p[0].abs() - self.offset;
        let ahead = station_z(self.station) - p[2];
        let level = (outboard * outboard + ahead * ahead).sqrt();
        let elevation = (p[1] - self.y).atan2(level).to_degrees();
        let bearing = outboard.atan2(ahead).to_degrees();
        // The centre post between the two windshield panes is never glass.
        p[0].abs() > 0.03 && (WINDSHIELD.holds(bearing, elevation) || SIDE_WINDOW.holds(bearing, elevation))
    }

    fn wall_point(&self, angle: f32, station: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        let up = if cos > 0.0 { self.roof_scale(station) } else { 1.0 };
        [width_scale(station) * cabin_radius() * sin, up * cabin_radius() * cos, station_z(station)]
    }

    fn wall_vertex(&self, angle: f32, station: f32, u: f32, v: f32) -> Vertex {
        let (sin, cos) = angle.sin_cos();
        let up = if cos > 0.0 { self.roof_scale(station) } else { 1.0 };
        let normal = normalised([-sin / width_scale(station), -cos / up, 0.0]);
        Vertex { position: self.wall_point(angle, station), normal, uv: [u, v] }
    }
}

/// A window, as the range of directions from the eye it is seen in: the
/// bearing outboard of straight ahead, and the elevation, in degrees. Its
/// outline is a rectangle in those terms with well rounded corners.
#[derive(Clone, Copy)]
struct Window {
    bearing: (f32, f32),
    elevation: (f32, f32),
}

const WINDSHIELD: Window = Window { bearing: (-16.0, 54.0), elevation: (-VISION_DOWN_OVER_NOSE, VISION_UP) };
const SIDE_WINDOW: Window = Window { bearing: (62.0, VISION_AFT_LIMIT - 10.0), elevation: (-25.0, 40.0) };
/// How square the corners are: 2 would make each window an ellipse.
const CORNER: f32 = 6.0;
const SEAL: [u8; 3] = [28, 28, 30];

impl Window {
    fn middle_and_half(range: (f32, f32)) -> (f32, f32) {
        ((range.0 + range.1) / 2.0, (range.1 - range.0) / 2.0)
    }

    fn holds(&self, bearing: f32, elevation: f32) -> bool {
        let (bm, bh) = Window::middle_and_half(self.bearing);
        let (em, eh) = Window::middle_and_half(self.elevation);
        ((bearing - bm) / bh).abs().powf(CORNER) + ((elevation - em) / eh).abs().powf(CORNER) < 1.0
    }

    /// A point of the outline, `t` running 0..1 round it.
    fn edge(&self, t: f32) -> (f32, f32) {
        let (bm, bh) = Window::middle_and_half(self.bearing);
        let (em, eh) = Window::middle_and_half(self.elevation);
        let angle = std::f32::consts::TAU * t;
        let rounded = |x: f32| x.signum() * x.abs().powf(2.0 / CORNER);
        (bm + bh * rounded(angle.cos()), em + eh * rounded(angle.sin()))
    }
}

impl Eye {
    /// Where the outline of a window meets the wall, as seen from the eye
    /// on one side: the line its seal runs along.
    fn seal_line(&self, side: f32, window: Window) -> Vec<Vec3> {
        let start = [side * self.offset, self.y, station_z(self.station)];
        let mut line: Vec<Vec3> = (0..=144)
            .map(|i| {
                let (bearing, elevation) = window.edge(i as f32 / 144.0);
                let (b, e) = (bearing.to_radians(), elevation.to_radians());
                let direction = [side * b.sin() * e.cos(), e.sin(), -b.cos() * e.cos()];
                let (mut near, mut far) = (0.05, 0.05);
                while far < 4.0 && self.inside(add(start, scaled(direction, far))) {
                    near = far;
                    far += 0.02;
                }
                for _ in 0..12 {
                    let middle = (near + far) / 2.0;
                    if self.inside(add(start, scaled(direction, middle))) { near = middle } else { far = middle }
                }
                add(start, scaled(direction, near - 0.014))
            })
            .collect();
        line.dedup_by(|a, b| crate::mesh::dot(crate::mesh::sub(*a, *b), crate::mesh::sub(*a, *b)) < 1e-8);
        line
    }
}

/// How many times a piece of wall that straddles a window edge is split.
const EDGE_SPLITS: u32 = 4;

/// Half the width of the frame round each window, metres. Estimated.
const FRAME_HALF_WIDTH: f32 = 0.009;
const FRAME: [u8; 3] = [46, 46, 48];

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Wall,
    Glass,
    Frame,
}

#[derive(Default)]
struct Skin {
    wall: Mesh,
    glazing: Mesh,
    frames: Mesh,
}

/// A piece of wall between two angles and two stations.
#[derive(Clone, Copy)]
struct Patch {
    angles: (f32, f32),
    stations: (f32, f32),
    u: (f32, f32),
    v: (f32, f32),
}

impl Eye {
    /// Adds a piece of wall as wall or as glass. A piece that is partly
    /// window is split in four and each quarter decided again, so window
    /// edges come out smooth without making the whole wall fine.
    fn wall_patch(&self, patch: Patch, splits: u32, skin: &mut Skin) {
        let Patch { angles: (a0, a1), stations: (s0, s1), u: (u0, u1), v: (v0, v1) } = patch;
        let (am, sm) = ((a0 + a1) / 2.0, (s0 + s1) / 2.0);
        let middle = self.kind_at(am, sm);
        let mixed = [(a0, s0), (a1, s0), (a1, s1), (a0, s1)].iter().any(|(a, s)| self.kind_at(*a, *s) != middle);
        if mixed && splits > 0 {
            let (um, vm) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
            for (angles, u) in [((a0, am), (u0, um)), ((am, a1), (um, u1))] {
                for (stations, v) in [((s0, sm), (v0, vm)), ((sm, s1), (vm, v1))] {
                    self.wall_patch(Patch { angles, stations, u, v }, splits - 1, skin);
                }
            }
            return;
        }
        let corners = [self.wall_vertex(a0, s0, u0, v0), self.wall_vertex(a1, s0, u1, v0), self.wall_vertex(a1, s1, u1, v1), self.wall_vertex(a0, s1, u0, v1)];
        match middle {
            Kind::Wall => skin.wall.quad(corners),
            Kind::Glass => skin.glazing.quad(corners),
            Kind::Frame => skin.frames.quad(corners),
        }
    }

    /// What the wall is at a point: glass, plain wall, or the frame that
    /// runs round the edge of each window.
    fn kind_at(&self, angle: f32, station: f32) -> Kind {
        let glass_at = |a: f32, s: f32| self.sees_out_through(self.wall_point(a, s));
        let here = glass_at(angle, station);
        let (da, ds) = (FRAME_HALF_WIDTH / cabin_radius(), FRAME_HALF_WIDTH / METRES_PER_INCH);
        let edge_near = [(da, 0.0), (-da, 0.0), (0.0, ds), (0.0, -ds)].iter().any(|(a, s)| glass_at(angle + a, station + s) != here);
        match (edge_near, here) {
            (true, _) => Kind::Frame,
            (false, true) => Kind::Glass,
            (false, false) => Kind::Wall,
        }
    }
}

pub fn build(scene: &mut Scene, eye: Eye) {
    let radius = cabin_radius();
    let edge = (floor_y() / radius).acos();
    let angle = |i: usize| -edge + 2.0 * edge * i as f32 / ARC_SEGMENTS as f32;
    let length = FS_FLIGHT_COMPARTMENT_BULKHEAD - FS_FORWARD_PRESSURE_BULKHEAD;
    let mut stations: Vec<f32> = (0..=SLICES).map(|j| FS_FORWARD_PRESSURE_BULKHEAD + length * j as f32 / SLICES as f32).collect();
    stations.extend([FS_WINDSHIELD_TOP, eye.station, FS_FULL_WIDTH]);
    stations.sort_by(|a, b| a.total_cmp(b));
    stations.dedup_by(|a, b| (*a - *b).abs() < 0.3);

    let (mut skin, mut floor) = (Skin::default(), Mesh::default());
    for pair in stations.windows(2) {
        let (s0, s1) = (pair[0], pair[1]);
        let (v0, v1) = ((s0 - FS_FORWARD_PRESSURE_BULKHEAD) / length, (s1 - FS_FORWARD_PRESSURE_BULKHEAD) / length);
        for i in 0..ARC_SEGMENTS {
            let patch = Patch { angles: (angle(i), angle(i + 1)), stations: (s0, s1), u: (i as f32 / ARC_SEGMENTS as f32, (i + 1) as f32 / ARC_SEGMENTS as f32), v: (v0, v1) };
            eye.wall_patch(patch, EDGE_SPLITS, &mut skin);
        }
        let outline = [eye.wall_point(-edge, s0), eye.wall_point(edge, s0), eye.wall_point(edge, s1), eye.wall_point(-edge, s1)];
        floor.convex(&outline, [0.0, 1.0, 0.0], |p| [p[0] / (2.0 * radius) + 0.5, (station_of(p[2]) - FS_FORWARD_PRESSURE_BULKHEAD) / length]);
    }
    scene.add("wall", WALL, skin.wall);
    scene.add("glazing", GLAZING, skin.glazing);
    scene.add("window frames", FRAME, skin.frames);
    for side in [-1.0, 1.0] {
        for window in [WINDSHIELD, SIDE_WINDOW] {
            scene.add("window seals", SEAL, tube(&eye.seal_line(side, window), 0.010, 24));
        }
    }
    scene.add("floor", FLOOR, floor);

    let mut bulkheads = Mesh::default();
    for (at, normal) in [(FS_FORWARD_PRESSURE_BULKHEAD, [0.0, 0.0, 1.0]), (FS_FLIGHT_COMPARTMENT_BULKHEAD, [0.0, 0.0, -1.0])] {
        let outline: Vec<Vec3> = (0..=ARC_SEGMENTS).map(|i| eye.wall_point(angle(i), at)).collect();
        bulkheads.convex(&outline, normal, |p| [p[0] / (2.0 * radius) + 0.5, p[1] / (2.0 * radius) + 0.5]);
    }
    scene.add("bulkheads", BULKHEAD, bulkheads);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stations::{inches, FLOOR_WIDTH_IN};

    const EYE: Eye = Eye { offset: 0.48, y: 0.517, station: 248.0 };

    fn shell() -> Scene {
        let mut scene = Scene::default();
        build(&mut scene, EYE);
        scene
    }

    #[test]
    fn the_back_of_the_compartment_is_the_documented_section() {
        let scene = shell();
        // The seals stand on the wall, so they are left out of its measurements.
        let mut skin = crate::mesh::Mesh::default();
        for part in scene.parts.iter().filter(|p| p.name != "window seals") {
            skin.append(&part.mesh);
        }
        let (low, high) = skin.bounds().unwrap();
        let close = |a: f32, b: f32| (a - b).abs() < 0.003;
        assert!(close(high[0] - low[0], 2.0 * cabin_radius()), "width {}", high[0] - low[0]);
        assert!(close(low[1], floor_y()) && close(high[1], cabin_radius()), "height {}", high[1]);
        assert!(close(low[2], station_z(FS_FORWARD_PRESSURE_BULKHEAD)) && close(high[2], station_z(FS_FLIGHT_COMPARTMENT_BULKHEAD)), "length");
        let widest = scene.part("floor").unwrap().mesh.vertices.iter().map(|v| v.position[0].abs()).fold(0.0, f32::max) * 2.0;
        assert!(close(widest, inches(FLOOR_WIDTH_IN)), "floor {widest}");
    }

    #[test]
    fn everything_is_seen_from_inside() {
        let scene = shell();
        let middle = [0.0, -0.2, station_z(241.0)];
        for part in &scene.parts {
            assert!(part.mesh.faces_agree_with_normals(), "{}", part.name);
            if part.name == "window seals" {
                continue;
            }
            for v in &part.mesh.vertices {
                assert!(crate::mesh::dot(crate::mesh::sub(middle, v.position), v.normal) > 0.0, "{} {v:?}", part.name);
            }
        }
    }

    #[test]
    fn the_traced_nose_agrees_with_the_documented_line_of_sight() {
        // The top of the windshield, seen from the eye, is at the documented
        // 27 degrees up. The outer line is an inch and a half above the
        // inside one that is traced here.
        let top = along(&ROOF, FS_WINDSHIELD_TOP) + 1.5 * METRES_PER_INCH;
        let ahead = (EYE.station - FS_WINDSHIELD_TOP) * METRES_PER_INCH;
        let up = ((top - EYE.y) / ahead).atan().to_degrees();
        assert!((up - VISION_UP).abs() < 2.0, "{up}");
        // The nose narrows and comes down towards the front, and never
        // exceeds the constant section.
        for pair in ROOF.windows(2).chain(WIDTH.windows(2)) {
            assert!(pair[1].0 > pair[0].0 && pair[1].1 > pair[0].1);
        }
        assert!(along(&ROOF, 280.0) <= cabin_radius() && (width_scale(280.0) - 1.0).abs() < 1e-6);
        assert!((width_scale(202.75) - 0.737).abs() < 1e-6 && width_scale(150.0) == width_scale(202.75));
    }

    #[test]
    fn there_is_glass_ahead_and_to_both_sides_but_not_on_the_centre_post() {
        let scene = shell();
        let glass = &scene.part("glazing").unwrap().mesh;
        let eye_z = station_z(EYE.station);
        for side in [-1.0, 1.0] {
            assert!(glass.vertices.iter().any(|v| v.position[0] * side > 0.2 && v.position[2] < eye_z - 0.5), "windshield");
            assert!(glass.vertices.iter().any(|v| v.position[0] * side > 0.9 && v.position[2] > eye_z - 0.2), "side window");
        }
        assert!(glass.vertices.iter().all(|v| v.position[0].abs() > 0.02), "centre post");
    }

    #[test]
    fn the_wall_is_the_edge_of_inside() {
        assert!(EYE.inside([0.0, 0.0, station_z(240.0)]));
        assert!(!EYE.inside([0.0, 0.0, station_z(300.0)]));
        assert!(!EYE.inside([0.0, floor_y() - 0.01, station_z(240.0)]));
        assert!(EYE.inside([0.0, cabin_radius() - 0.08, station_z(279.0)]));
        assert!(!EYE.inside([0.0, cabin_radius() - 0.02, station_z(210.0)]), "the roof is lower at the front");
    }

    #[test]
    fn each_window_has_a_seal_that_runs_round_it_on_the_wall() {
        for side in [-1.0, 1.0] {
            for window in [WINDSHIELD, SIDE_WINDOW] {
                let line = EYE.seal_line(side, window);
                assert!(line.len() > 100);
                let (first, last) = (line[0], line[line.len() - 1]);
                assert!(crate::mesh::dot(crate::mesh::sub(first, last), crate::mesh::sub(first, last)).sqrt() < 0.01, "the seal closes on itself");
                for p in &line {
                    assert!(EYE.inside(*p), "just inside the wall");
                    assert!(p[0] * side > 0.0, "on its own side");
                }
            }
        }
        assert!(WINDSHIELD.holds(10.0, 0.0) && !WINDSHIELD.holds(53.9, 26.9), "corners are rounded off");
    }
}
