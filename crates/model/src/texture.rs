//! Surface finishes: procedural textures that repeat seamlessly, with the
//! relief and shine that go with them. The renderer samples them directly;
//! `bake` turns them into colour and normal images for a simulator.

use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Finish {
    Paint,
    PanelFace,
    Plastic,
    Ivory,
    Metal,
    Leather,
    Sheepskin,
    Carpet,
    Lining,
    Rubber,
    Glass,
    Screen,
    /// Clear plastic that what is behind shows through.
    Clear,
    /// Drawn in light: what a display shows.
    Lit,
}

fn lattice(x: i32, y: i32, period_x: i32, period_y: i32, seed: u32) -> f32 {
    let mut h = (x.rem_euclid(period_x) as u32).wrapping_mul(374_761_393) ^ (y.rem_euclid(period_y) as u32).wrapping_mul(668_265_263) ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

/// Smooth noise in 0..1 that repeats once per unit of u and v, with the
/// given number of lumps across each.
fn noise(u: f32, v: f32, lumps_u: i32, lumps_v: i32, seed: u32) -> f32 {
    let (x, y) = (u * lumps_u as f32, v * lumps_v as f32);
    let (x0, y0) = (x.floor(), y.floor());
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let (tx, ty) = (ease(x - x0), ease(y - y0));
    let at = |dx: i32, dy: i32| lattice(x0 as i32 + dx, y0 as i32 + dy, lumps_u, lumps_v, seed);
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * tx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * tx;
    top + (bottom - top) * ty
}

/// Several layers of noise, each twice as fine and half as strong.
fn layered(u: f32, v: f32, lumps: i32, layers: u32, seed: u32) -> f32 {
    let (mut total, mut weight, mut strength) = (0.0, 0.0, 1.0);
    for layer in 0..layers {
        let scale = lumps << layer;
        total += strength * noise(u, v, scale, scale, seed + layer);
        weight += strength;
        strength *= 0.5;
    }
    total / weight
}

impl Finish {
    pub const ALL: [Finish; 14] = [
        Finish::Paint,
        Finish::PanelFace,
        Finish::Plastic,
        Finish::Ivory,
        Finish::Metal,
        Finish::Leather,
        Finish::Sheepskin,
        Finish::Carpet,
        Finish::Lining,
        Finish::Rubber,
        Finish::Glass,
        Finish::Screen,
        Finish::Clear,
        Finish::Lit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Finish::Paint => "paint",
            Finish::PanelFace => "panel_face",
            Finish::Plastic => "plastic",
            Finish::Ivory => "ivory",
            Finish::Metal => "metal",
            Finish::Leather => "leather",
            Finish::Sheepskin => "sheepskin",
            Finish::Carpet => "carpet",
            Finish::Lining => "lining",
            Finish::Rubber => "rubber",
            Finish::Glass => "glass",
            Finish::Screen => "screen",
            Finish::Clear => "clear",
            Finish::Lit => "lit",
        }
    }

    /// Metres covered by one repeat of the texture.
    pub fn tile_size(self) -> f32 {
        match self {
            Finish::Carpet => 0.40,
            Finish::Leather | Finish::Sheepskin | Finish::Lining => 0.30,
            Finish::Paint | Finish::PanelFace => 0.25,
            Finish::Glass | Finish::Screen | Finish::Clear | Finish::Lit => 1.0,
            _ => 0.20,
        }
    }

    /// Depth of the surface relief, metres.
    pub fn relief(self) -> f32 {
        match self {
            Finish::Sheepskin => 0.006,
            Finish::Carpet => 0.0025,
            Finish::Leather => 0.0007,
            Finish::Lining => 0.0003,
            Finish::Rubber => 0.0002,
            Finish::Paint | Finish::PanelFace | Finish::Metal => 0.00006,
            _ => 0.0,
        }
    }

    /// How strong and how tight the shine is.
    pub fn shine(self) -> (f32, f32) {
        match self {
            Finish::Glass | Finish::Clear => (0.55, 120.0),
            Finish::Screen | Finish::Lit => (0.45, 90.0),
            Finish::Metal => (0.45, 45.0),
            Finish::Plastic => (0.42, 55.0),
            Finish::Ivory => (0.40, 40.0),
            Finish::Leather => (0.20, 18.0),
            Finish::Paint => (0.18, 24.0),
            Finish::Rubber => (0.12, 12.0),
            Finish::PanelFace => (0.22, 30.0),
            Finish::Lining => (0.03, 6.0),
            Finish::Sheepskin | Finish::Carpet => (0.0, 1.0),
        }
    }

    /// The texture at a point of the tile: how much lighter or darker than
    /// the base colour (1 is unchanged), and the height of the relief, 0..1.
    pub fn sample(self, u: f32, v: f32) -> (f32, f32) {
        let (u, v) = (u.rem_euclid(1.0), v.rem_euclid(1.0));
        match self {
            Finish::Paint => {
                let n = layered(u, v, 64, 3, 11);
                (1.0 + 0.05 * (n - 0.5), n)
            }
            Finish::PanelFace => {
                let n = layered(u, v, 96, 2, 23);
                (1.0 + 0.04 * (n - 0.5), n)
            }
            Finish::Plastic | Finish::Ivory | Finish::Rubber => {
                let n = layered(u, v, 48, 2, 37);
                (1.0 + 0.03 * (n - 0.5), n)
            }
            Finish::Metal => {
                // Brushed: fine across, long along.
                let n = 0.7 * noise(u, v, 256, 4, 41) + 0.3 * noise(u, v, 128, 8, 43);
                (1.0 + 0.10 * (n - 0.5), n)
            }
            Finish::Leather => {
                // Grain: ridges where the noise crosses its middle.
                let n = layered(u, v, 28, 4, 53);
                let ridge = 1.0 - (2.0 * n - 1.0).abs();
                (0.93 + 0.11 * ridge, ridge)
            }
            Finish::Sheepskin => {
                // Clumps of wool with finer fibres over them.
                let clumps = layered(u, v, 12, 2, 61);
                let fibres = layered(u, v, 96, 3, 67);
                let height = 0.6 * clumps + 0.4 * fibres;
                (0.66 + 0.68 * height, height)
            }
            Finish::Carpet => {
                let n = layered(u, v, 128, 2, 71);
                (0.84 + 0.32 * n, n)
            }
            Finish::Lining => {
                let weave = 0.5 + 0.5 * (TAU * 60.0 * u).sin() * (TAU * 60.0 * v).sin();
                let n = layered(u, v, 32, 2, 79);
                (0.97 + 0.03 * weave + 0.03 * (n - 0.5), weave)
            }
            Finish::Glass | Finish::Screen | Finish::Clear | Finish::Lit => (1.0, 0.0),
        }
    }

    /// The slope of the relief at a point, as metres of height per metre
    /// along u and along v.
    pub fn slope(self, u: f32, v: f32) -> (f32, f32) {
        let relief = self.relief();
        if relief == 0.0 {
            return (0.0, 0.0);
        }
        let step = 1.0 / 1024.0;
        let here = self.sample(u, v).1;
        let scale = relief / (step * self.tile_size());
        ((self.sample(u + step, v).1 - here) * scale, (self.sample(u, v + step).1 - here) * scale)
    }
}

impl Finish {
    /// How much of what is behind a surface it hides: 1 unless it is clear.
    pub fn opacity(self) -> f32 {
        if self == Finish::Clear { 0.2 } else { 1.0 }
    }

    /// 1 for bare metal, 0 for everything else.
    pub fn metalness(self) -> f32 {
        if self == Finish::Metal { 1.0 } else { 0.0 }
    }

    /// How smooth the surface is, 0..1, for a simulator's gloss channel.
    pub fn gloss(self) -> f32 {
        let (strength, tightness) = self.shine();
        (strength * 0.9 + (tightness / 120.0) * 0.5).clamp(0.0, 1.0)
    }
}

/// A baked set of images for one material: colour as RGB, and a normal map
/// as RGBA in X-Plane's layout (red and green the normal, blue the
/// metalness, alpha the gloss). Rows from the top.
pub struct Baked {
    pub size: usize,
    pub albedo: Vec<u8>,
    pub normal: Vec<u8>,
}

/// Bakes a finish in a base colour, `size` pixels square, using every
/// processor: an 8192 pixel set is 67 million texels.
pub fn bake(finish: Finish, colour: [u8; 3], size: usize) -> Baked {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(size.max(1));
    let rows_each = size.div_ceil(threads);
    let (metal, gloss) = ((finish.metalness() * 255.0) as u8, (finish.gloss() * 255.0) as u8);
    let bands: Vec<(Vec<u8>, Vec<u8>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|band| {
                scope.spawn(move || {
                    let rows = (band * rows_each).min(size)..((band + 1) * rows_each).min(size);
                    let (mut albedo, mut normal) = (Vec::with_capacity(rows.len() * size * 3), Vec::with_capacity(rows.len() * size * 4));
                    for y in rows {
                        for x in 0..size {
                            let (u, v) = ((x as f32 + 0.5) / size as f32, 1.0 - (y as f32 + 0.5) / size as f32);
                            let (shade, _) = finish.sample(u, v);
                            albedo.extend(colour.map(|c| (c as f32 * shade).clamp(0.0, 255.0) as u8));
                            let (du, dv) = finish.slope(u, v);
                            let length = (du * du + dv * dv + 1.0).sqrt();
                            normal.extend([-du / length, -dv / length].map(|c| ((c * 0.5 + 0.5) * 255.0) as u8));
                            normal.extend([metal, gloss]);
                        }
                    }
                    (albedo, normal)
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().expect("bake worker")).collect()
    });
    let (mut albedo, mut normal) = (Vec::with_capacity(size * size * 3), Vec::with_capacity(size * size * 4));
    for (a, n) in bands {
        albedo.extend(a);
        normal.extend(n);
    }
    Baked { size, albedo, normal }
}

/// Writes an image with three (RGB) or four (RGBA) bytes a pixel.
pub fn save_png(path: &std::path::Path, pixels: &[u8], size: usize) -> Result<(), String> {
    let colour = if pixels.len() == size * size * 4 { png::ColorType::Rgba } else { png::ColorType::Rgb };
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), size as u32, size as u32);
    encoder.set_color(colour);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(pixels).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_finish_repeats_without_a_seam() {
        for finish in Finish::ALL {
            for t in [0.0, 0.13, 0.5, 0.87] {
                let (a, b) = (finish.sample(0.0, t), finish.sample(1.0, t));
                assert!((a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4, "{finish:?} across u");
                let (a, b) = (finish.sample(t, 0.0), finish.sample(t, 1.0));
                assert!((a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4, "{finish:?} across v");
            }
        }
    }

    #[test]
    fn textures_stay_near_their_base_colour_and_fabric_is_rougher_than_paint() {
        let spread = |finish: Finish| {
            let shades: Vec<f32> = (0..400).map(|i| finish.sample(i as f32 * 0.0137, i as f32 * 0.0291).0).collect();
            let (low, high) = (shades.iter().cloned().fold(f32::MAX, f32::min), shades.iter().cloned().fold(f32::MIN, f32::max));
            assert!(low > 0.5 && high < 1.5, "{finish:?} {low} {high}");
            high - low
        };
        assert!(spread(Finish::Sheepskin) > 4.0 * spread(Finish::Paint));
        assert!(Finish::Sheepskin.relief() > Finish::Leather.relief() && Finish::Leather.relief() > Finish::Paint.relief());
        assert_eq!(Finish::Screen.slope(0.3, 0.3), (0.0, 0.0));
    }

    #[test]
    fn a_baked_set_has_colour_normal_metalness_and_gloss() {
        let baked = bake(Finish::Leather, [100, 80, 60], 37);
        assert_eq!((baked.albedo.len(), baked.normal.len()), (37 * 37 * 3, 37 * 37 * 4));
        assert!(baked.albedo.chunks_exact(3).any(|c| c[0] != 100), "the grain shows");
        assert!(baked.normal.chunks_exact(4).any(|n| n[0] != 127 || n[1] != 127), "the relief shows");
        assert!(baked.normal.chunks_exact(4).all(|n| n[2] == 0), "leather is not metal");
        let metal = bake(Finish::Metal, [150, 150, 150], 8);
        assert!(metal.normal.chunks_exact(4).all(|n| n[2] == 255 && n[3] > baked.normal[3]), "metal is metal, and glossier");
        // The same whether baked in one band or many.
        let again = bake(Finish::Leather, [100, 80, 60], 37);
        assert_eq!(baked.albedo, again.albedo);
    }
}
