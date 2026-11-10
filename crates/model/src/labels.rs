//! Legends: pieces of text lying on a surface. All the legends of a model
//! share one image, the atlas; each legend is a small rectangle that shows
//! its piece of it. A second image holds what glows when the panel
//! lighting is on.

use crate::font;
use crate::mesh::{add, scaled, Vec3};
use crate::part::Surface;

pub struct Label {
    /// Bottom left, bottom right, top right, top left.
    pub corners: [Vec3; 4],
    pub normal: Vec3,
    pub text: String,
    pub colour: [u8; 3],
    /// Whether it is lit from behind.
    pub lit: bool,
}

impl Label {
    /// Text `height` metres tall, centred at (u, v) on a surface and
    /// standing `proud` of it.
    pub fn on(surface: &Surface, u: f32, v: f32, height: f32, proud: f32, text: &str, colour: [u8; 3], lit: bool) -> Label {
        let half_width = height * font::aspect(text) / 2.0;
        let corner = |su: f32, sv: f32| add(surface.point(u + su * half_width, v + sv * height / 2.0), scaled(surface.normal(), proud));
        Label { corners: [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)], normal: surface.normal(), text: text.to_string(), colour, lit }
    }
}

pub struct Atlas {
    pub size: usize,
    /// How much lettering covers each pixel, rows from the top.
    pub cover: Vec<u8>,
    /// Each label's piece of the image as left, top, right, bottom, 0..1
    /// with v measured down from the top.
    pub pieces: Vec<[f32; 4]>,
}

impl Atlas {
    /// Lays every label's lettering out in rows in a square image.
    pub fn build(labels: &[Label], size: usize) -> Result<Atlas, String> {
        let letter_height = (size / 73).max(8);
        let gap = 2;
        let mut cover = vec![0u8; size * size];
        let mut pieces = Vec::with_capacity(labels.len());
        let (mut x, mut y) = (gap, gap);
        // The same text is only drawn once.
        let mut drawn: std::collections::HashMap<&str, [f32; 4]> = std::collections::HashMap::new();
        for label in labels {
            if let Some(piece) = drawn.get(label.text.as_str()) {
                pieces.push(*piece);
                continue;
            }
            let bitmap = font::draw(&label.text, letter_height);
            if bitmap.width + 2 * gap > size {
                return Err(format!("the legend {:?} is too long for a {size} pixel image", label.text));
            }
            if x + bitmap.width + gap > size {
                x = gap;
                y += letter_height + gap;
            }
            if y + letter_height + gap > size {
                return Err(format!("{} legends do not fit in a {size} pixel image", labels.len()));
            }
            for row in 0..bitmap.height {
                let from = row * bitmap.width;
                cover[(y + row) * size + x..(y + row) * size + x + bitmap.width].copy_from_slice(&bitmap.cover[from..from + bitmap.width]);
            }
            let scale = size as f32;
            let piece = [x as f32 / scale, y as f32 / scale, (x + bitmap.width) as f32 / scale, (y + letter_height) as f32 / scale];
            drawn.insert(label.text.as_str(), piece);
            pieces.push(piece);
            x += bitmap.width + gap;
        }
        Ok(Atlas { size, cover, pieces })
    }

    /// Cover at a point of the image, 0..1.
    pub fn cover_at(&self, u: f32, v: f32) -> f32 {
        let x = ((u * self.size as f32) as usize).min(self.size - 1);
        let y = ((v * self.size as f32) as usize).min(self.size - 1);
        self.cover[y * self.size + x] as f32 / 255.0
    }

    /// The image for a simulator: white lettering whose alpha is the cover.
    pub fn colour_image(&self) -> Vec<u8> {
        self.cover.iter().flat_map(|c| [255, 255, 255, *c]).collect()
    }

    /// The image of what glows: the lettering, on black.
    pub fn lit_image(&self) -> Vec<u8> {
        self.cover.iter().flat_map(|c| [*c, *c, *c]).collect()
    }
}

/// The legends as an OBJ8 file drawing from the atlas images. Legends that
/// are not lit from behind are left out of the lit image by X-Plane's
/// per-object lighting level, so they are written as a second object.
pub fn obj8(labels: &[Label], atlas: &Atlas, texture: &str, lit: Option<&str>, only_lit: bool) -> String {
    use std::fmt::Write;
    let chosen: Vec<(&Label, &[f32; 4])> = labels.iter().zip(&atlas.pieces).filter(|(l, _)| l.lit == only_lit).collect();
    let mut out = String::from("I\n800\nOBJ\n\n");
    let _ = writeln!(out, "TEXTURE {texture}");
    if let Some(lit) = lit.filter(|_| only_lit) {
        let _ = writeln!(out, "TEXTURE_LIT {lit}");
    }
    let _ = writeln!(out, "POINT_COUNTS {} 0 0 {}\n", chosen.len() * 4, chosen.len() * 6);
    for (label, piece) in &chosen {
        // Texture v runs up in an OBJ8 file; the image's rows run down.
        let uv = [[piece[0], 1.0 - piece[3]], [piece[2], 1.0 - piece[3]], [piece[2], 1.0 - piece[1]], [piece[0], 1.0 - piece[1]]];
        for (corner, uv) in label.corners.iter().zip(uv) {
            let _ = writeln!(out, "VT {:.5} {:.5} {:.5} {:.4} {:.4} {:.4} {:.5} {:.5}", corner[0], corner[1], corner[2], label.normal[0], label.normal[1], label.normal[2], uv[0], uv[1]);
        }
    }
    out.push('\n');
    for i in 0..chosen.len() as u32 {
        // Clockwise, as X-Plane wants.
        let b = i * 4;
        let _ = writeln!(out, "IDX {}\nIDX {}\nIDX {}\nIDX {}\nIDX {}\nIDX {}", b, b + 2, b + 1, b, b + 3, b + 2);
    }
    let _ = writeln!(out, "\nATTR_poly_os 2\nTRIS 0 {}", chosen.len() * 6);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> Surface {
        Surface::new([0.0, 0.0, 5.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn a_label_is_the_height_asked_for_and_as_wide_as_its_text() {
        let label = Label::on(&panel(), 0.1, 0.2, 0.004, 0.001, "FUEL", [255; 3], true);
        let [bl, br, tr, _] = label.corners;
        assert!((tr[1] - br[1] - 0.004).abs() < 1e-6);
        assert!(((br[0] - bl[0]) - 0.004 * font::aspect("FUEL")).abs() < 1e-6);
        assert!(((bl[0] + br[0]) / 2.0 - 0.1).abs() < 1e-6 && (bl[2] - 5.001).abs() < 1e-6);
    }

    #[test]
    fn every_label_gets_a_piece_with_its_lettering_in_it() {
        let labels: Vec<Label> = ["FUEL", "APU", "FUEL", "HYDRAULIC"].iter().map(|t| Label::on(&panel(), 0.0, 0.0, 0.004, 0.0, t, [255; 3], true)).collect();
        let atlas = Atlas::build(&labels, 512).unwrap();
        assert_eq!(atlas.pieces.len(), 4);
        assert_eq!(atlas.pieces[0], atlas.pieces[2], "the same text shares a piece");
        for piece in &atlas.pieces {
            let inked = (0..40).any(|i| atlas.cover_at(piece[0] + (piece[2] - piece[0]) * i as f32 / 40.0, (piece[1] + piece[3]) / 2.0) > 0.5);
            assert!(inked);
        }
        assert_eq!(atlas.colour_image().len(), 512 * 512 * 4);
        assert!(Atlas::build(&labels, 16).is_err(), "too small an image is refused");
    }

    #[test]
    fn the_file_holds_only_the_legends_asked_for() {
        let mut labels: Vec<Label> = ["A", "B", "C"].iter().map(|t| Label::on(&panel(), 0.0, 0.0, 0.004, 0.0, t, [255; 3], true)).collect();
        labels[1].lit = false;
        let atlas = Atlas::build(&labels, 256).unwrap();
        let lit = obj8(&labels, &atlas, "legends.png", Some("legends_LIT.png"), true);
        assert!(lit.contains("TEXTURE_LIT legends_LIT.png") && lit.contains("POINT_COUNTS 8 0 0 12"));
        let unlit = obj8(&labels, &atlas, "legends.png", Some("legends_LIT.png"), false);
        assert!(!unlit.contains("TEXTURE_LIT") && unlit.contains("POINT_COUNTS 4 0 0 6"));
    }
}
