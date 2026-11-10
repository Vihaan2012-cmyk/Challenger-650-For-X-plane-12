//! Curved shapes: cylinders, bodies with rounded corners, and tubes swept
//! along a path.

use std::f32::consts::TAU;

use crate::mesh::{add, cross, dot, normalised, scaled, sub, Mesh, Vec3, Vertex};

/// Two directions square to an axis and to each other.
fn perpendiculars(axis: Vec3) -> (Vec3, Vec3) {
    let reference = if axis[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let first = normalised(cross(axis, reference));
    (first, cross(axis, first))
}

fn ring_direction(first: Vec3, second: Vec3, i: usize, segments: usize) -> Vec3 {
    let angle = TAU * i as f32 / segments as f32;
    add(scaled(first, angle.cos()), scaled(second, angle.sin()))
}

/// A closed cylinder standing on `base` along `axis`.
pub fn cylinder(base: Vec3, axis: Vec3, radius: f32, height: f32, segments: usize) -> Mesh {
    let axis = normalised(axis);
    let (first, second) = perpendiculars(axis);
    let top = add(base, scaled(axis, height));
    let mut mesh = Mesh::default();
    let (mut low, mut high) = (Vec::new(), Vec::new());
    for i in 0..segments {
        let (d0, d1) = (ring_direction(first, second, i, segments), ring_direction(first, second, i + 1, segments));
        let (u0, u1) = (i as f32 / segments as f32, (i + 1) as f32 / segments as f32);
        let v = |centre: Vec3, d: Vec3, u: f32, t: f32| Vertex { position: add(centre, scaled(d, radius)), normal: d, uv: [u, t] };
        mesh.quad([v(base, d0, u0, 0.0), v(base, d1, u1, 0.0), v(top, d1, u1, 1.0), v(top, d0, u0, 1.0)]);
        low.push(add(base, scaled(d0, radius)));
        high.push(add(top, scaled(d0, radius)));
    }
    mesh.convex(&high, axis, |_| [0.5, 0.5]);
    mesh.convex(&low, scaled(axis, -1.0), |_| [0.5, 0.5]);
    mesh
}

/// A cylinder with shallow flutes down its side, like the grip of a knob.
pub fn fluted(base: Vec3, axis: Vec3, radius: f32, height: f32, flutes: usize, segments: usize) -> Mesh {
    let axis = normalised(axis);
    let (first, second) = perpendiculars(axis);
    let top = add(base, scaled(axis, height));
    let reach = |i: usize| radius * (1.0 - 0.07 * (0.5 + 0.5 * (TAU * flutes as f32 * i as f32 / segments as f32).cos()));
    let mut mesh = Mesh::default();
    for i in 0..segments {
        let (d0, d1) = (ring_direction(first, second, i, segments), ring_direction(first, second, i + 1, segments));
        let (r0, r1) = (reach(i), reach(i + 1));
        let v = |centre: Vec3, d: Vec3, r: f32, t: f32| Vertex { position: add(centre, scaled(d, r)), normal: d, uv: [i as f32 / segments as f32, t] };
        mesh.quad([v(base, d0, r0, 0.0), v(base, d1, r1, 0.0), v(top, d1, r1, 1.0), v(top, d0, r0, 1.0)]);
        // The ends are not convex, so each is a fan from its middle.
        for (centre, normal) in [(top, axis), (base, scaled(axis, -1.0))] {
            let flat = |position: Vec3| Vertex { position, normal, uv: [0.5, 0.5] };
            mesh.triangle([flat(centre), flat(add(centre, scaled(d0, r0))), flat(add(centre, scaled(d1, r1)))]);
        }
    }
    mesh
}

/// A slab whose outline in its first two axes is a rectangle with rounded
/// corners; flat on the two faces across its third axis.
pub fn rounded_slab(centre: Vec3, axes: [Vec3; 3], half: [f32; 3], radius: f32, corner_segments: usize) -> Mesh {
    let radius = radius.min(half[0]).min(half[1]);
    let mut outline: Vec<([f32; 2], [f32; 2])> = Vec::new();
    for (corner, (sx, sy)) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)].into_iter().enumerate() {
        for k in 0..=corner_segments {
            let angle = TAU / 4.0 * (corner as f32 + k as f32 / corner_segments as f32);
            let direction = [angle.cos(), angle.sin()];
            outline.push(([sx * (half[0] - radius) + radius * direction[0], sy * (half[1] - radius) + radius * direction[1]], direction));
        }
    }
    let place = |p: [f32; 2], w: f32| add(add(add(centre, scaled(axes[0], p[0])), scaled(axes[1], p[1])), scaled(axes[2], w));
    let mut mesh = Mesh::default();
    let count = outline.len();
    for i in 0..count {
        let ((p0, n0), (p1, n1)) = (outline[i], outline[(i + 1) % count]);
        let normal = |n: [f32; 2]| add(scaled(axes[0], n[0]), scaled(axes[1], n[1]));
        let v = |p: [f32; 2], n: [f32; 2], w: f32, t: f32| Vertex { position: place(p, w), normal: normal(n), uv: [i as f32 / count as f32, t] };
        mesh.quad([v(p0, n0, -half[2], 0.0), v(p1, n1, -half[2], 0.0), v(p1, n1, half[2], 1.0), v(p0, n0, half[2], 1.0)]);
    }
    for sign in [-1.0, 1.0] {
        let face: Vec<Vec3> = outline.iter().map(|(p, _)| place(*p, sign * half[2])).collect();
        mesh.convex(&face, scaled(axes[2], sign), |_| [0.5, 0.5]);
    }
    mesh
}

/// A smooth curve through the given points, `steps` pieces between each.
pub fn smooth(points: &[Vec3], steps: usize) -> Vec<Vec3> {
    let at = |i: isize| points[i.clamp(0, points.len() as isize - 1) as usize];
    let mut curve = Vec::new();
    for i in 0..points.len() as isize - 1 {
        let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
        for step in 0..steps {
            let t = step as f32 / steps as f32;
            let mut point = [0.0; 3];
            for axis in 0..3 {
                point[axis] = 0.5
                    * (2.0 * p1[axis]
                        + (p2[axis] - p0[axis]) * t
                        + (2.0 * p0[axis] - 5.0 * p1[axis] + 4.0 * p2[axis] - p3[axis]) * t * t
                        + (3.0 * p1[axis] - p0[axis] - 3.0 * p2[axis] + p3[axis]) * t * t * t);
            }
            curve.push(point);
        }
    }
    curve.push(points[points.len() - 1]);
    curve
}

/// A round tube along a path, closed at both ends.
pub fn tube(path: &[Vec3], radius: f32, segments: usize) -> Mesh {
    let mut mesh = Mesh::default();
    if path.len() < 2 {
        return mesh;
    }
    let tangent = |i: usize| normalised(sub(path[(i + 1).min(path.len() - 1)], path[i.saturating_sub(1)]));
    let (mut side, _) = perpendiculars(tangent(0));
    let mut rings: Vec<Vec<(Vec3, Vec3)>> = Vec::new();
    for i in 0..path.len() {
        let along = tangent(i);
        // Carry the side direction along the path so the tube does not twist.
        side = normalised(sub(side, scaled(along, dot(side, along))));
        let other = cross(along, side);
        rings.push((0..segments).map(|k| ring_direction(side, other, k, segments)).map(|d| (add(path[i], scaled(d, radius)), d)).collect());
    }
    for (i, pair) in rings.windows(2).enumerate() {
        for k in 0..segments {
            let next = (k + 1) % segments;
            let v = |(position, normal): (Vec3, Vec3), u: usize, t: usize| Vertex { position, normal, uv: [u as f32 / segments as f32, t as f32 / path.len() as f32] };
            mesh.quad([v(pair[0][k], k, i), v(pair[0][next], k + 1, i), v(pair[1][next], k + 1, i + 1), v(pair[1][k], k, i + 1)]);
        }
    }
    let end = |ring: &Vec<(Vec3, Vec3)>| ring.iter().map(|(p, _)| *p).collect::<Vec<Vec3>>();
    mesh.convex(&end(&rings[0]), scaled(tangent(0), -1.0), |_| [0.5, 0.5]);
    mesh.convex(&end(&rings[rings.len() - 1]), tangent(path.len() - 1), |_| [0.5, 0.5]);
    mesh
}

/// A body swept along a path with an oval section that changes along it:
/// for sculpted parts like a control wheel's arms. Each point of the path
/// has a half width across the path and a half depth along `depth_axis`.
/// The path and its sections are smoothed, `steps` pieces between each.
pub fn swept(path: &[Vec3], radii: &[(f32, f32)], depth_axis: Vec3, segments: usize, steps: usize) -> Mesh {
    let mut mesh = Mesh::default();
    if path.len() < 2 || path.len() != radii.len() {
        return mesh;
    }
    let line = smooth(path, steps);
    let sizes = smooth(&radii.iter().map(|r| [r.0, r.1, 0.0]).collect::<Vec<Vec3>>(), steps);
    let tangent = |i: usize| normalised(sub(line[(i + 1).min(line.len() - 1)], line[i.saturating_sub(1)]));
    let mut rings: Vec<Vec<(Vec3, Vec3)>> = Vec::new();
    for i in 0..line.len() {
        let along = tangent(i);
        let deep = normalised(sub(depth_axis, scaled(along, dot(depth_axis, along))));
        let across = cross(along, deep);
        let (width, depth) = (sizes[i][0].max(1e-4), sizes[i][1].max(1e-4));
        rings.push(
            (0..segments)
                .map(|k| {
                    let (sin, cos) = (TAU * k as f32 / segments as f32).sin_cos();
                    let position = add(line[i], add(scaled(across, width * cos), scaled(deep, depth * sin)));
                    (position, normalised(add(scaled(across, cos / width), scaled(deep, sin / depth))))
                })
                .collect(),
        );
    }
    for (i, pair) in rings.windows(2).enumerate() {
        for k in 0..segments {
            let next = (k + 1) % segments;
            let v = |(position, normal): (Vec3, Vec3), u: usize, t: usize| Vertex { position, normal, uv: [u as f32 / segments as f32, t as f32 / line.len() as f32] };
            mesh.quad([v(pair[0][k], k, i), v(pair[0][next], k + 1, i), v(pair[1][next], k + 1, i + 1), v(pair[1][k], k, i + 1)]);
        }
    }
    let end = |ring: &Vec<(Vec3, Vec3)>| ring.iter().map(|(p, _)| *p).collect::<Vec<Vec3>>();
    mesh.convex(&end(&rings[0]), scaled(tangent(0), -1.0), |_| [0.5, 0.5]);
    mesh.convex(&end(&rings[rings.len() - 1]), tangent(line.len() - 1), |_| [0.5, 0.5]);
    mesh
}

/// A soft, cushion-like body: a box of the given half sizes whose faces
/// bulge and whose edges are rounded, lumpier the higher `lumps` is (metres).
/// `squareness` runs from 1 (an ellipsoid) towards 0 (a sharp box). The two
/// flat-ish faces are across the third axis.
pub fn pillow(centre: Vec3, axes: [Vec3; 3], half: [f32; 3], squareness: f32, lumps: f32, rings: usize, around: usize, seed: u32) -> Mesh {
    let power = |x: f32, e: f32| x.signum() * x.abs().powf(e);
    let lump = |i: usize, j: usize| {
        // A cheap repeatable bump per vertex, smoothed by its neighbours.
        let hash = |i: usize, j: usize| {
            let mut h = (i as u32 % around as u32).wrapping_mul(374_761_393) ^ (j as u32).wrapping_mul(668_265_263) ^ seed.wrapping_mul(2_246_822_519);
            h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
            (h ^ (h >> 16)) as f32 / u32::MAX as f32 - 0.5
        };
        (hash(i, j) * 2.0 + hash(i + 1, j) + hash(i + around - 1, j) + hash(i, j + 1) + hash(i, j.saturating_sub(1))) / 6.0
    };
    let vertex = |i: usize, j: usize| {
        let latitude = -TAU / 4.0 + TAU / 2.0 * j as f32 / rings as f32;
        let longitude = TAU * (i % around) as f32 / around as f32;
        let (cl, sl) = (latitude.cos().max(0.0), latitude.sin());
        let (co, so) = (longitude.cos(), longitude.sin());
        let e = squareness;
        let local = [half[0] * power(cl, e) * power(co, e), half[1] * power(cl, e) * power(so, e), half[2] * power(sl, e)];
        let n = [power(cl, 2.0 - e) * power(co, 2.0 - e) / half[0], power(cl, 2.0 - e) * power(so, 2.0 - e) / half[1], power(sl, 2.0 - e) / half[2]];
        let normal = normalised(add(add(scaled(axes[0], n[0]), scaled(axes[1], n[1])), scaled(axes[2], n[2])));
        let flat = add(add(add(centre, scaled(axes[0], local[0])), scaled(axes[1], local[1])), scaled(axes[2], local[2]));
        // Lumps fade to nothing at the two poles, where the rings close up.
        let position = add(flat, scaled(normal, lumps * 2.0 * lump(i % around, j) * cl.sqrt()));
        Vertex { position, normal, uv: [i as f32 / around as f32, j as f32 / rings as f32] }
    };
    let mut mesh = Mesh::default();
    for j in 0..rings {
        for i in 0..around {
            if j == 0 {
                mesh.triangle([vertex(i, 0), vertex(i, 1), vertex(i + 1, 1)]);
            } else if j == rings - 1 {
                mesh.triangle([vertex(i, j), vertex(i + 1, j), vertex(i, j + 1)]);
            } else {
                mesh.quad([vertex(i, j), vertex(i + 1, j), vertex(i + 1, j + 1), vertex(i, j + 1)]);
            }
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::part::UPRIGHT;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn a_cylinder_has_its_radius_and_height_and_faces_outwards() {
        let mesh = cylinder([1.0, 0.0, 0.0], [0.0, 0.0, 2.0], 0.5, 3.0, 16);
        let (low, high) = mesh.bounds().unwrap();
        assert!(close(low[2], 0.0) && close(high[2], 3.0));
        assert!(close(low[0], 0.5) && close(high[0], 1.5));
        assert!(mesh.faces_agree_with_normals());
        assert_eq!(mesh.triangle_count(), 16 * 2 + 14 * 2);
    }

    #[test]
    fn a_fluted_cylinder_is_closed_and_never_wider_than_its_radius() {
        let mesh = fluted([0.0; 3], [0.0, 0.0, 1.0], 0.5, 1.0, 12, 96);
        assert!(mesh.faces_agree_with_normals());
        assert_eq!(mesh.triangle_count(), 96 * 4);
        let widest = mesh.vertices.iter().map(|v| (v.position[0].powi(2) + v.position[1].powi(2)).sqrt()).fold(0.0, f32::max);
        let narrowest = mesh.vertices.iter().map(|v| (v.position[0].powi(2) + v.position[1].powi(2)).sqrt()).filter(|r| *r > 0.1).fold(f32::MAX, f32::min);
        assert!(close(widest, 0.5) && narrowest < 0.47, "{widest} {narrowest}");
    }

    #[test]
    fn a_rounded_slab_fills_its_box_but_not_its_corners() {
        let mesh = rounded_slab([0.0; 3], UPRIGHT, [0.3, 0.2, 0.1], 0.05, 6);
        let (low, high) = mesh.bounds().unwrap();
        assert!(close(high[0], 0.3) && close(high[1], 0.2) && close(high[2], 0.1));
        assert!(close(low[0], -0.3) && close(low[1], -0.2) && close(low[2], -0.1));
        assert!(mesh.faces_agree_with_normals());
        assert!(mesh.vertices.iter().all(|v| !(v.position[0] > 0.299 && v.position[1] > 0.199)));
    }

    #[test]
    fn a_tube_stays_its_radius_from_its_path_round_a_bend() {
        let path = smooth(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]], 8);
        assert_eq!(path.len(), 17);
        assert_eq!(path[8], [1.0, 0.0, 0.0]);
        let mesh = tube(&path, 0.1, 10);
        assert!(mesh.faces_agree_with_normals());
        for v in &mesh.vertices {
            let nearest = path.iter().map(|p| dot(sub(v.position, *p), sub(v.position, *p)).sqrt()).fold(f32::MAX, f32::min);
            assert!(nearest < 0.1001, "{nearest}");
        }
    }

    #[test]
    fn a_pillow_fills_its_box_and_is_closed() {
        let mesh = pillow([0.0; 3], UPRIGHT, [0.3, 0.2, 0.1], 0.5, 0.0, 16, 24, 1);
        let (low, high) = mesh.bounds().unwrap();
        assert!(close(high[0], 0.3) && close(high[1], 0.2) && close(high[2], 0.1), "{high:?}");
        assert!(close(low[0], -0.3) && close(low[1], -0.2) && close(low[2], -0.1));
        assert!(mesh.faces_agree_with_normals());
        assert_eq!(mesh.triangle_count(), 24 * 2 + 24 * 14 * 2);
        // Lumps move the surface, but only by about as much as asked.
        let lumpy = pillow([0.0; 3], UPRIGHT, [0.3, 0.2, 0.1], 0.5, 0.01, 16, 24, 1);
        let (_, lumpy_high) = lumpy.bounds().unwrap();
        assert!(lumpy_high[0] < 0.32 && lumpy.vertices != mesh.vertices);
    }

    #[test]
    fn a_swept_body_takes_the_width_and_depth_given_at_each_point() {
        let path = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]];
        let mesh = swept(&path, &[(0.3, 0.1), (0.2, 0.1), (0.1, 0.05)], [0.0, 0.0, 1.0], 32, 4);
        assert!(mesh.faces_agree_with_normals());
        let (low, high) = mesh.bounds().unwrap();
        assert!(close(high[0], 0.3) && close(low[0], -0.3), "widest at the start: {high:?}");
        // Smoothing lets the depth swell a little between two equal sections.
        assert!((0.0999..0.108).contains(&high[2]) && (low[2] + high[2]).abs() < 1e-4, "depth lies along the axis given: {high:?}");
        assert!(close(low[1], 0.0) && close(high[1], 2.0));
        let at_the_end = mesh.vertices.iter().filter(|v| v.position[1] > 1.999).map(|v| v.position[0].abs()).fold(0.0, f32::max);
        assert!(close(at_the_end, 0.1), "{at_the_end}");
        assert_eq!(swept(&path, &[(0.1, 0.1)], [0.0, 0.0, 1.0], 8, 2).triangle_count(), 0, "mismatched lists make nothing");
    }
}
