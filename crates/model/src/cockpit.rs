//! The flight compartment.

use crate::mesh::{Mesh, Vec3, Vertex};
use crate::stations::{cabin_radius, floor_y, station_z, FS_FLIGHT_COMPARTMENT_BULKHEAD, FS_FORWARD_PRESSURE_BULKHEAD};

const ARC_SEGMENTS: usize = 48;

/// The space the flight compartment occupies, seen from inside: the fuselage
/// wall, the floor, and the bulkheads in front and behind.
///
/// The wall is the constant section carried forward unchanged. The real nose
/// narrows ahead of the seats; that taper has to be lofted from the drawings
/// and replaces this wall when it is done.
pub fn envelope() -> Mesh {
    let radius = cabin_radius();
    let floor = floor_y();
    let front = station_z(FS_FORWARD_PRESSURE_BULKHEAD);
    let back = station_z(FS_FLIGHT_COMPARTMENT_BULKHEAD);
    let length = back - front;

    // Angle from straight up; the wall runs from the left floor edge, over
    // the top, to the right floor edge.
    let edge = (floor / radius).acos();
    let angle = |i: usize| -edge + 2.0 * edge * i as f32 / ARC_SEGMENTS as f32;
    let around = |a: f32| [a.sin(), a.cos()];

    let mut mesh = Mesh::default();
    for i in 0..ARC_SEGMENTS {
        let corner = |a: f32, z: f32, u: f32| {
            let [x, y] = around(a);
            Vertex { position: [radius * x, radius * y, z], normal: [-x, -y, 0.0], uv: [u, (z - front) / length] }
        };
        let (a0, a1) = (angle(i), angle(i + 1));
        let (u0, u1) = (i as f32 / ARC_SEGMENTS as f32, (i + 1) as f32 / ARC_SEGMENTS as f32);
        mesh.quad([corner(a0, front, u0), corner(a1, front, u1), corner(a1, back, u1), corner(a0, back, u0)]);
    }

    let half_width = radius * edge.sin();
    let flat = |position: Vec3| [(position[0] + radius) / (2.0 * radius), (position[2] - front) / length];
    mesh.convex(
        &[[-half_width, floor, front], [half_width, floor, front], [half_width, floor, back], [-half_width, floor, back]],
        [0.0, 1.0, 0.0],
        flat,
    );

    let upright = |position: Vec3| [(position[0] + radius) / (2.0 * radius), (position[1] + radius) / (2.0 * radius)];
    for (z, normal) in [(front, [0.0, 0.0, 1.0]), (back, [0.0, 0.0, -1.0])] {
        let outline: Vec<Vec3> = (0..=ARC_SEGMENTS)
            .map(|i| {
                let [x, y] = around(angle(i));
                [radius * x, radius * y, z]
            })
            .collect();
        mesh.convex(&outline, normal, upright);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stations::{inches, FLOOR_WIDTH_IN};

    #[test]
    fn the_envelope_has_the_documented_size() {
        let (low, high) = envelope().bounds().unwrap();
        let close = |a: f32, b: f32| (a - b).abs() < 0.002;
        assert!(close(high[0] - low[0], 2.0 * cabin_radius()), "width");
        assert!(close(low[1], floor_y()) && close(high[1], cabin_radius()), "height");
        assert!(close(low[2], station_z(FS_FORWARD_PRESSURE_BULKHEAD)), "front");
        assert!(close(high[2], station_z(FS_FLIGHT_COMPARTMENT_BULKHEAD)), "back");
    }

    #[test]
    fn the_floor_is_as_wide_as_published() {
        let mesh = envelope();
        let on_floor = mesh.vertices.iter().filter(|v| v.normal == [0.0, 1.0, 0.0]);
        let width = on_floor.map(|v| v.position[0].abs()).fold(0.0, f32::max) * 2.0;
        assert!((width - inches(FLOOR_WIDTH_IN)).abs() < 0.002, "{width}");
    }

    #[test]
    fn every_face_is_seen_from_inside() {
        let mesh = envelope();
        assert!(mesh.faces_agree_with_normals());
        let middle = [0.0, 0.0, station_z(241.0)];
        for v in &mesh.vertices {
            let towards_middle = crate::mesh::sub(middle, v.position);
            assert!(crate::mesh::dot(towards_middle, v.normal) > 0.0, "{v:?}");
        }
    }
}
