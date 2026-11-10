//! A small software renderer for checking the model without a simulator.
//!
//! Faces pointing away from the camera are not drawn, so an interior can be
//! looked into from outside. Each part is drawn in its finish: texture,
//! surface relief and shine. Creases and corners are darkened where light
//! would not reach, and parts are lightly outlined where they meet.

use crate::labels::{Atlas, Label};
use crate::mesh::{add, cross, dot, normalised, scaled, sub, Mesh, Vec3};
use crate::texture::Finish;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Vertical field of view in degrees.
    pub field_of_view: f32,
}

/// A mesh with the colour and finish to draw it in.
#[derive(Clone, Copy)]
pub struct Painted<'a> {
    pub mesh: &'a Mesh,
    pub colour: [u8; 3],
    pub finish: Finish,
}

pub struct Image {
    pub width: usize,
    pub height: usize,
    /// Rows from the top, three bytes a pixel.
    pub rgb: Vec<u8>,
}

impl Image {
    pub fn pixel(&self, x: usize, y: usize) -> [u8; 3] {
        let i = (y * self.width + x) * 3;
        [self.rgb[i], self.rgb[i + 1], self.rgb[i + 2]]
    }

    /// Half the size, each pixel the average of four; smooths edges.
    pub fn halved(&self) -> Image {
        let (width, height) = (self.width / 2, self.height / 2);
        let mut rgb = Vec::with_capacity(width * height * 3);
        for y in 0..height {
            for x in 0..width {
                for channel in 0..3 {
                    let sum: u32 = [(0, 0), (1, 0), (0, 1), (1, 1)].iter().map(|(dx, dy)| self.pixel(2 * x + dx, 2 * y + dy)[channel] as u32).sum();
                    rgb.push((sum / 4) as u8);
                }
            }
        }
        Image { width, height, rgb }
    }

    pub fn save_png(&self, path: &std::path::Path) -> Result<(), String> {
        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&self.rgb).map_err(|e| e.to_string())
    }
}

pub const BACKGROUND: [u8; 3] = [24, 26, 30];
const NEAR: f32 = 0.02;
const SKY_LIGHT: Vec3 = [0.29, 0.86, 0.42];
/// How far light is taken to be blocked by nearby surfaces, metres.
const SHADOW_REACH: f32 = 0.05;

/// A corner in camera space (x right, y up, z into the scene) with its
/// world-space normal.
#[derive(Clone, Copy)]
struct Corner {
    at: Vec3,
    normal: Vec3,
}

fn between(a: Corner, b: Corner, t: f32) -> Corner {
    Corner { at: add(a.at, scaled(sub(b.at, a.at), t)), normal: add(a.normal, scaled(sub(b.normal, a.normal), t)) }
}

/// Cuts away the part of a triangle that is behind the camera.
fn clip_near(triangle: [Corner; 3]) -> Vec<Corner> {
    let mut kept = Vec::with_capacity(4);
    for i in 0..3 {
        let (a, b) = (triangle[i], triangle[(i + 1) % 3]);
        let (a_in, b_in) = (a.at[2] >= NEAR, b.at[2] >= NEAR);
        if a_in {
            kept.push(a);
        }
        if a_in != b_in {
            kept.push(between(a, b, (NEAR - a.at[2]) / (b.at[2] - a.at[2])));
        }
    }
    kept
}

struct Target {
    width: usize,
    height: usize,
    focal: f32,
    eye: Vec3,
    axes: [Vec3; 3],
    rgb: Vec<f32>,
    depth: Vec<f32>,
    owner: Vec<u16>,
    night: bool,
}

/// The texture coordinates of a world point on a surface facing `normal`:
/// the surface is laid flat along whichever axis it faces most, and the
/// same rule gives the exported model its texture coordinates.
pub fn box_coordinates(position: Vec3, normal: Vec3, tile: f32) -> ([f32; 2], Vec3, Vec3) {
    let facing = (0..3).max_by(|a, b| normal[*a].abs().total_cmp(&normal[*b].abs())).unwrap_or(1);
    let (a, b) = match facing {
        0 => (2, 1),
        1 => (0, 2),
        _ => (0, 1),
    };
    let axis = |i: usize| {
        let mut v = [0.0; 3];
        v[i] = 1.0;
        v
    };
    ([position[a] / tile, position[b] / tile], axis(a), axis(b))
}

/// Draws meshes, each in its own colour and finish, in daylight.
pub fn render(parts: &[Painted], camera: &Camera, width: usize, height: usize) -> Image {
    render_with(parts, None, camera, width, height, false)
}

/// Draws meshes and the legends on them. At night the cockpit is dark and
/// only what is lit shows: screens, and legends lit from behind.
pub fn render_with(parts: &[Painted], legends: Option<(&[Label], &Atlas)>, camera: &Camera, width: usize, height: usize, night: bool) -> Image {
    let forward = normalised(sub(camera.target, camera.eye));
    let right = normalised(cross(forward, camera.up));
    let up = cross(right, forward);
    let to_camera = |p: Vec3| {
        let d = sub(p, camera.eye);
        [dot(d, right), dot(d, up), dot(d, forward)]
    };
    let background = BACKGROUND.map(|c| c as f32);
    let mut target = Target {
        width,
        height,
        focal: height as f32 / 2.0 / (camera.field_of_view.to_radians() / 2.0).tan(),
        eye: camera.eye,
        axes: [right, up, forward],
        rgb: background.iter().copied().cycle().take(width * height * 3).collect(),
        depth: vec![f32::INFINITY; width * height],
        owner: vec![0; width * height],
        night,
    };

    for (number, part) in parts.iter().enumerate() {
        for t in part.mesh.indices.chunks_exact(3) {
            let [a, b, c] = [part.mesh.vertices[t[0] as usize], part.mesh.vertices[t[1] as usize], part.mesh.vertices[t[2] as usize]];
            let face = cross(sub(b.position, a.position), sub(c.position, a.position));
            if dot(face, sub(camera.eye, a.position)) <= 0.0 {
                continue;
            }
            let polygon = clip_near([a, b, c].map(|v| Corner { at: to_camera(v.position), normal: v.normal }));
            for i in 1..polygon.len().saturating_sub(1) {
                fill(&[polygon[0], polygon[i], polygon[i + 1]], part, number as u16 + 1, &mut target);
            }
        }
    }

    // Legends lie on top of what has been drawn.
    if let Some((labels, atlas)) = legends {
        for (label, piece) in labels.iter().zip(&atlas.pieces) {
            if dot(label.normal, sub(camera.eye, label.corners[0])) <= 0.0 {
                continue;
            }
            let uv = [[piece[0], piece[3]], [piece[2], piece[3]], [piece[2], piece[1]], [piece[0], piece[1]]];
            let strength = match (night, label.lit) {
                (false, _) => 0.94,
                (true, true) => 1.0,
                (true, false) => 0.12,
            };
            for corners in [[0, 1, 2], [0, 2, 3]] {
                let triangle = corners.map(|i| (to_camera(label.corners[i]), uv[i]));
                if triangle.iter().all(|c| c.0[2] >= NEAR) {
                    draw_legend(&triangle, label.colour, strength, atlas, &mut target);
                }
            }
        }
    }

    let shadow = contact_shadow(&target);
    let mut rgb = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        for x in 0..width {
            let slot = y * width + x;
            let own = target.owner[slot];
            let edge = (x + 1 < width && target.owner[slot + 1] != own) || (y + 1 < height && target.owner[slot + width] != own);
            let dim = shadow[slot] * if edge { 0.72 } else { 1.0 };
            for channel in 0..3 {
                rgb.push((target.rgb[slot * 3 + channel] * dim).clamp(0.0, 255.0) as u8);
            }
        }
    }
    Image { width, height, rgb }
}

/// How much light reaches each pixel, 0..1: less in creases and inside
/// corners. A pixel is shaded when the surface either side of it, taken in
/// opposite pairs, is on average nearer the camera than it is; a flat
/// surface seen at any angle is left alone.
fn contact_shadow(target: &Target) -> Vec<f32> {
    const DIRECTIONS: [(f32, f32); 6] = [(1.0, 0.0), (0.5, 0.866), (-0.5, 0.866), (0.707, 0.707), (-0.707, 0.707), (0.0, 1.0)];
    let (width, height) = (target.width as i32, target.height as i32);
    let mut light = vec![1.0; target.depth.len()];
    for y in 0..height {
        for x in 0..width {
            let slot = (y * width + x) as usize;
            let depth = target.depth[slot];
            if !depth.is_finite() {
                continue;
            }
            let reach = (target.focal * SHADOW_REACH / depth).clamp(2.0, 48.0);
            let (mut blocked, mut pairs) = (0.0, 0.0);
            for (dx, dy) in DIRECTIONS {
                for step in [0.45, 1.0] {
                    let (ox, oy) = ((dx * reach * step) as i32, (dy * reach * step) as i32);
                    let at = |sx: i32, sy: i32| (sx >= 0 && sy >= 0 && sx < width && sy < height).then(|| target.depth[(sy * width + sx) as usize]).filter(|d| d.is_finite());
                    if let (Some(one), Some(other)) = (at(x + ox, y + oy), at(x - ox, y - oy)) {
                        let nearer = depth - (one + other) / 2.0;
                        if nearer > 0.0015 && nearer < 4.0 * SHADOW_REACH {
                            blocked += (nearer / (SHADOW_REACH * 0.5)).min(1.0);
                        }
                        pairs += 1.0;
                    }
                }
            }
            if pairs > 0.0 {
                light[slot] = 1.0 - 0.7 * blocked / pairs;
            }
        }
    }
    light
}

/// Blends one triangle of a legend over what is already drawn, wherever it
/// is not hidden behind something nearer.
fn draw_legend(tri: &[(Vec3, [f32; 2]); 3], colour: [u8; 3], strength: f32, atlas: &Atlas, target: &mut Target) {
    let (width, height) = (target.width, target.height);
    let screen = tri.map(|c| [width as f32 / 2.0 + c.0[0] / c.0[2] * target.focal, height as f32 / 2.0 - c.0[1] / c.0[2] * target.focal]);
    let edge = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    let area = edge(screen[0], screen[1], screen[2]);
    if area.abs() < 1e-6 {
        return;
    }
    let span = |axis: usize, limit: usize| {
        let low = screen.iter().map(|s| s[axis]).fold(f32::INFINITY, f32::min).floor().max(0.0) as usize;
        let high = (screen.iter().map(|s| s[axis]).fold(f32::NEG_INFINITY, f32::max).ceil().max(0.0) as usize).min(limit.saturating_sub(1));
        low..=high
    };
    for y in span(1, height) {
        for x in span(0, width) {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let weights = [edge(screen[1], screen[2], p) / area, edge(screen[2], screen[0], p) / area, edge(screen[0], screen[1], p) / area];
            if weights.iter().any(|w| *w < 0.0) {
                continue;
            }
            let over = [weights[0] / tri[0].0[2], weights[1] / tri[1].0[2], weights[2] / tri[2].0[2]];
            let z = 1.0 / (over[0] + over[1] + over[2]);
            let slot = y * width + x;
            if z > target.depth[slot] + 0.004 {
                continue;
            }
            let u = (0..3).map(|i| tri[i].1[0] * over[i] * z).sum::<f32>();
            let v = (0..3).map(|i| tri[i].1[1] * over[i] * z).sum::<f32>();
            let cover = atlas.cover_at(u, v);
            for channel in 0..3 {
                let ink = colour[channel] as f32 * strength;
                target.rgb[slot * 3 + channel] += (ink - target.rgb[slot * 3 + channel]) * cover;
            }
        }
    }
}

fn fill(tri: &[Corner; 3], part: &Painted, owner: u16, target: &mut Target) {
    let (width, height) = (target.width, target.height);
    let screen = tri.map(|c| [width as f32 / 2.0 + c.at[0] / c.at[2] * target.focal, height as f32 / 2.0 - c.at[1] / c.at[2] * target.focal]);
    let edge = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    let area = edge(screen[0], screen[1], screen[2]);
    if area.abs() < 1e-6 {
        return;
    }
    let span = |axis: usize, limit: usize| {
        let low = screen.iter().map(|s| s[axis]).fold(f32::INFINITY, f32::min).floor().max(0.0) as usize;
        let high = (screen.iter().map(|s| s[axis]).fold(f32::NEG_INFINITY, f32::max).ceil().max(0.0) as usize).min(limit.saturating_sub(1));
        low..=high
    };
    let finish = part.finish;
    let (shine, tightness) = finish.shine();
    for y in span(1, height) {
        for x in span(0, width) {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let weights = [edge(screen[1], screen[2], p) / area, edge(screen[2], screen[0], p) / area, edge(screen[0], screen[1], p) / area];
            if weights.iter().any(|w| *w < 0.0) {
                continue;
            }
            // Perspective-correct weights.
            let over = [weights[0] / tri[0].at[2], weights[1] / tri[1].at[2], weights[2] / tri[2].at[2]];
            let z = 1.0 / (over[0] + over[1] + over[2]);
            let slot = y * width + x;
            if z >= target.depth[slot] {
                continue;
            }
            // A clear surface is laid over what is behind it and hides nothing.
            let opacity = finish.opacity();
            if opacity == 1.0 {
                target.depth[slot] = z;
                target.owner[slot] = owner;
            }
            let mix = |f: &dyn Fn(&Corner) -> Vec3| (0..3).fold([0.0; 3], |sum, i| add(sum, scaled(f(&tri[i]), over[i] * z)));
            let smooth = normalised(mix(&|c| c.normal));
            let at = mix(&|c| c.at);
            let offset = add(add(scaled(target.axes[0], at[0]), scaled(target.axes[1], at[1])), scaled(target.axes[2], at[2]));
            let world = add(target.eye, offset);
            let towards_eye = normalised(scaled(offset, -1.0));

            let ([u, v], along_u, along_v) = box_coordinates(world, smooth, finish.tile_size());
            let (shade, _) = finish.sample(u, v);
            let (slope_u, slope_v) = finish.slope(u, v);
            let normal = normalised(sub(sub(smooth, scaled(along_u, slope_u)), scaled(along_v, slope_v)));

            let diffuse = 0.40 + 0.36 * dot(normal, towards_eye).max(0.0) + 0.24 * dot(normal, SKY_LIGHT).max(0.0);
            let halfway = normalised(add(towards_eye, SKY_LIGHT));
            let glint = shine * (dot(normal, halfway).max(0.0).powf(tightness) + 0.35 * dot(normal, towards_eye).max(0.0).powf(tightness * 2.0));
            let sheen = shine * 0.55 * (1.0 - dot(normal, towards_eye).max(0.0)).powi(4);
            let glint = glint + sheen;
            let (diffuse, glint) = if target.night { (diffuse * 0.14, glint * 0.25) } else { (diffuse, glint) };
            // What a display draws shines by itself, by day and by night.
            let glow = match finish {
                Finish::Lit => part.colour.map(|c| c as f32 * if target.night { 0.92 } else { 0.5 }),
                Finish::Screen if target.night => [14.0, 30.0, 54.0],
                _ => [0.0; 3],
            };
            for channel in 0..3 {
                let lit = part.colour[channel] as f32 * shade * diffuse + glow[channel];
                let behind = target.rgb[slot * 3 + channel];
                target.rgb[slot * 3 + channel] = behind + (lit - behind) * opacity + 255.0 * glint;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Vertex;

    const GREY: [u8; 3] = [180, 180, 180];
    const UP: Vec3 = [0.0, 1.0, 0.0];

    fn plain(mesh: &Mesh, colour: [u8; 3]) -> Painted<'_> {
        Painted { mesh, colour, finish: Finish::Screen }
    }

    fn wall(normal: Vec3) -> Mesh {
        let v = |x: f32, y: f32| Vertex { position: [x, y, 0.0], normal, uv: [0.0, 0.0] };
        let mut mesh = Mesh::default();
        mesh.triangle([v(-1.0, -1.0), v(1.0, -1.0), v(0.0, 1.0)]);
        mesh
    }

    fn sheet(z: f32, half: f32) -> Mesh {
        let v = |x: f32, y: f32| Vertex { position: [x, y, z], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] };
        let mut mesh = Mesh::default();
        mesh.quad([v(-half, -half), v(half, -half), v(half, half), v(-half, half)]);
        mesh
    }

    fn looking_aft() -> Camera {
        Camera { eye: [0.0, 0.0, -4.0], target: [0.0, 0.0, 0.0], up: UP, field_of_view: 60.0 }
    }

    #[test]
    fn a_face_towards_the_camera_is_drawn_where_it_should_be() {
        let mesh = wall([0.0, 0.0, -1.0]);
        let image = render(&[plain(&mesh, GREY)], &looking_aft(), 80, 60);
        assert_ne!(image.pixel(40, 32), BACKGROUND, "middle");
        assert_eq!(image.pixel(2, 2), BACKGROUND, "corner");
        assert_ne!(image.pixel(40, 20), BACKGROUND, "the apex is up");
        assert_eq!(image.pixel(40, 55), BACKGROUND, "below the base");
    }

    #[test]
    fn a_face_pointing_away_is_not_drawn() {
        let mesh = wall([0.0, 0.0, 1.0]);
        let image = render(&[plain(&mesh, GREY)], &looking_aft(), 80, 60);
        assert!(image.rgb.chunks_exact(3).all(|p| p == BACKGROUND));
    }

    #[test]
    fn a_face_that_passes_behind_the_camera_is_still_drawn_in_front() {
        let v = |x: f32, z: f32| Vertex { position: [x, -1.0, z], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] };
        let mut floor = Mesh::default();
        floor.quad([v(-5.0, -10.0), v(5.0, -10.0), v(5.0, 10.0), v(-5.0, 10.0)]);
        let camera = Camera { eye: [0.0, 0.0, 0.0], target: [0.0, 0.0, 1.0], up: UP, field_of_view: 60.0 };
        let image = render(&[plain(&floor, GREY)], &camera, 80, 60);
        assert_ne!(image.pixel(40, 55), BACKGROUND, "floor below the horizon");
        assert_eq!(image.pixel(40, 5), BACKGROUND, "nothing above it");
    }

    #[test]
    fn the_nearer_part_wins_and_is_outlined_against_the_other() {
        let (far, near) = (sheet(1.0, 3.0), sheet(0.0, 0.5));
        let image = render(&[plain(&far, [200, 0, 0]), plain(&near, [0, 200, 0])], &looking_aft(), 80, 60);
        let middle = image.pixel(40, 30);
        assert!(middle[1] > middle[0], "the near sheet is in front: {middle:?}");
        let across: Vec<[u8; 3]> = (0..80).map(|x| image.pixel(x, 30)).collect();
        let darkest = across.iter().map(|p| p[0].max(p[1])).min().unwrap();
        assert!(darkest < across[5][0], "darker where they meet");
        let small = image.halved();
        assert_eq!((small.width, small.height), (40, 30));
    }

    #[test]
    fn a_clear_cover_lets_what_is_behind_it_show() {
        let (behind, cover) = (sheet(1.0, 3.0), sheet(0.0, 3.0));
        let through = |finish: Finish| {
            let parts = [plain(&behind, [0, 200, 0]), Painted { mesh: &cover, colour: [200, 200, 200], finish }];
            render(&parts, &looking_aft(), 80, 60).pixel(40, 30)
        };
        let (clear, solid) = (through(Finish::Clear), through(Finish::Plastic));
        assert!(clear[1] > clear[0] + 30, "green shows through: {clear:?}");
        assert!((solid[1] as i32 - solid[0] as i32).abs() < 12, "a solid cover hides it: {solid:?}");
    }

    #[test]
    fn a_rough_finish_shows_its_texture_and_a_plain_one_does_not() {
        let mesh = sheet(0.0, 3.0);
        // Roughness: how much each pixel differs from the one beside it.
        let spread = |finish: Finish| {
            let image = render(&[Painted { mesh: &mesh, colour: GREY, finish }], &looking_aft(), 80, 60);
            (10..70).map(|x| (image.pixel(x, 30)[0] as i32 - image.pixel(x + 1, 30)[0] as i32).abs()).sum::<i32>()
        };
        assert!(spread(Finish::Sheepskin) > 4 * spread(Finish::Screen) + 60, "{} against {}", spread(Finish::Sheepskin), spread(Finish::Screen));
    }

    #[test]
    fn an_inside_corner_is_darker_than_the_open_surface_beside_it() {
        // A floor meeting a back wall, seen from above and in front.
        let v = |p: Vec3, normal: Vec3| Vertex { position: p, normal, uv: [0.0, 0.0] };
        let mut room = Mesh::default();
        let up = [0.0, 1.0, 0.0];
        room.quad([v([-2.0, 0.0, -2.0], up), v([2.0, 0.0, -2.0], up), v([2.0, 0.0, 0.0], up), v([-2.0, 0.0, 0.0], up)]);
        let back = [0.0, 0.0, -1.0];
        room.quad([v([-2.0, 0.0, 0.0], back), v([2.0, 0.0, 0.0], back), v([2.0, 2.0, 0.0], back), v([-2.0, 2.0, 0.0], back)]);
        let camera = Camera { eye: [0.0, 1.0, -1.2], target: [0.0, 0.0, 0.0], up: UP, field_of_view: 60.0 };
        let (width, height) = (120, 120);
        let forward = normalised(sub(camera.target, camera.eye));
        let right = normalised(cross(forward, camera.up));
        let target_for = |depths: Vec<f32>| Target { width, height, focal: 100.0, eye: camera.eye, axes: [right, cross(right, forward), forward], rgb: vec![], depth: depths, owner: vec![], night: false };
        // Depth of a flat floor alone, then of the floor and wall together.
        let image = render(&[plain(&room, GREY)], &camera, width, height);
        let column: Vec<u8> = (0..height).map(|y| image.pixel(60, y)[0]).collect();
        let corner = *column.iter().filter(|c| **c != BACKGROUND[0]).min().unwrap();
        let open = column[height - 8];
        assert!(corner + 12 < open, "corner {corner} against open floor {open}");
        // And a single flat sheet, seen at a slant, is not shaded at all.
        let slanted: Vec<f32> = (0..width * height).map(|i| 1.0 + (i / width) as f32 * 0.01).collect();
        assert!(contact_shadow(&target_for(slanted)).iter().all(|l| *l > 0.999));
    }

    #[test]
    fn a_legend_shows_on_its_surface_and_glows_at_night_only_if_lit() {
        use crate::part::Surface;
        let mesh = sheet(0.0, 3.0);
        // A surface facing the camera, which looks along +z.
        let surface = Surface::new([0.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let brightest = |lit: bool, night: bool| {
            let labels = [Label::on(&surface, 0.0, 0.0, 1.2, 0.001, "H", [255, 255, 255], lit)];
            let atlas = Atlas::build(&labels, 256).unwrap();
            let parts = [Painted { mesh: &mesh, colour: [40, 40, 40], finish: Finish::PanelFace }];
            let image = render_with(&parts, Some((&labels, &atlas)), &looking_aft(), 160, 120, night);
            image.rgb.iter().copied().max().unwrap()
        };
        assert!(brightest(true, false) > 200, "white lettering by day");
        assert!(brightest(true, true) > 200, "lit lettering at night");
        assert!(brightest(false, true) < 80, "unlit lettering is dark at night");
    }
}
