//! Lettering for panel legends, drawn to MS33558, the public standard for
//! numerals and letters on aircraft instrument panels: plain single-stroke
//! capitals with round bowls, the I and 1 bare strokes, and a stroke one
//! sixth of the letter height thick.
//!
//! Each character is drawn on a grid six units high. A stroke is a run of
//! x y pairs; one that starts with `~` is a curve through those points.

/// Height of a capital, in units.
pub const CAP: f32 = 6.0;
/// Half the stroke thickness: the standard's six to one height to stroke.
const STROKE: f32 = 0.5;
/// Space between one character and the next, in units.
const GAP: f32 = 1.5;

/// A character's width and its strokes, separated by `|`.
fn glyph(c: char) -> (f32, &'static str) {
    match c.to_ascii_uppercase() {
        'A' => (4.0, "0 0 2 6 4 0|0.8 2 3.2 2"),
        'B' => (4.0, "0 0 0 6|~0 6 3.6 6 3.6 3.1 0 3.1|~0 3.1 4 3.1 4 0 0 0"),
        'C' => (4.0, "~4 4.4 4 6 0 6 0 0 4 0 4 1.6"),
        'D' => (4.0, "0 0 0 6|~0 6 4 6 4 0 0 0"),
        'E' => (3.8, "3.8 6 0 6 0 0 3.8 0|0 3.1 3 3.1"),
        'F' => (3.8, "3.8 6 0 6 0 0|0 3.1 3 3.1"),
        'G' => (4.0, "~4 4.4 4 6 0 6 0 0 4 0 4 2.6|4 2.6 2.2 2.6"),
        'H' => (4.0, "0 0 0 6|4 0 4 6|0 3.1 4 3.1"),
        'I' => (0.0, "0 0 0 6"),
        'J' => (3.6, "~3.6 6 3.6 0 0 0 0 1.8"),
        'K' => (4.0, "0 0 0 6|4 6 0 2.4|1.4 3.6 4 0"),
        'L' => (3.6, "0 6 0 0 3.6 0"),
        'M' => (4.6, "0 0 0 6 2.3 2 4.6 6 4.6 0"),
        'N' => (4.0, "0 0 0 6 4 0 4 6"),
        'O' => (4.0, "~2 0 0 0 0 6 4 6 4 0 2 0"),
        'P' => (4.0, "0 0 0 6|~0 6 4 6 4 2.8 0 2.8"),
        'Q' => (4.0, "~2 0 0 0 0 6 4 6 4 0 2 0|2.4 1.6 4 0"),
        'R' => (4.0, "0 0 0 6|~0 6 4 6 4 2.8 0 2.8|2 2.8 4 0"),
        'S' => (4.0, "~4 4.6 4 6 0 6 0 3.1 4 3.1 4 0 0 0 0 1.4"),
        'T' => (4.0, "0 6 4 6|2 6 2 0"),
        'U' => (4.0, "~0 6 0 0 4 0 4 6"),
        'V' => (4.0, "0 6 2 0 4 6"),
        'W' => (5.0, "0 6 1.25 0 2.5 6 3.75 0 5 6"),
        'X' => (4.0, "0 0 4 6|0 6 4 0"),
        'Y' => (4.0, "0 6 2 3 4 6|2 3 2 0"),
        'Z' => (4.0, "0 6 4 6 0 0 4 0"),
        '0' => (3.4, "~1.7 0 0 0 0 6 3.4 6 3.4 0 1.7 0"),
        '1' => (0.0, "0 0 0 6"),
        '2' => (4.0, "~0 4.8 0 6 4 6 4 3.2 0 0|0 0 4 0"),
        '3' => (4.0, "0 6 4 6 1.8 3.6|~1.8 3.6 4 3.6 4 0 0 0 0 1.4"),
        '4' => (4.0, "3 0 3 6 0 1.8 4 1.8"),
        '5' => (4.0, "4 6 0.4 6 0.2 3.4|~0.2 3.4 2 3.9 4 3.6 4 0 0 0 0 1.4"),
        '6' => (4.0, "~3.4 5.9 1.2 5.6 0 3 0 0 4 0 4 3.7 1 3.7 0.1 2.4"),
        '7' => (4.0, "0 6 4 6 1.4 0"),
        '8' => (4.0, "~2 3.1 0.3 3.1 0.3 6 3.7 6 3.7 3.1 2 3.1|~2 3.1 0 3.1 0 0 4 0 4 3.1 2 3.1"),
        '9' => (4.0, "~0.6 0.1 2.8 0.4 4 3 4 6 0 6 0 2.3 3 2.3 3.9 3.6"),
        '-' => (2.4, "0 3 2.4 3"),
        '/' => (3.4, "0 0 3.4 6"),
        '.' => (0.0, "0 0 0 0.2"),
        ':' => (0.0, "0 1 0 1.2|0 4 0 4.2"),
        '+' => (3.6, "1.8 1.2 1.8 4.8|0 3 3.6 3"),
        '&' => (4.4, "~4.4 0 1 4 1 6 3 6 3 4 0 2 0 0 2.6 0 4.4 2.4"),
        '(' => (1.4, "~1.4 6 0 4.5 0 1.5 1.4 0"),
        ')' => (1.4, "~0 6 1.4 4.5 1.4 1.5 0 0"),
        // A rule: runs of these join into one unbroken line.
        '_' => (4.0, "-0.75 3 4.75 3"),
        ' ' => (2.2, ""),
        _ => (2.2, ""),
    }
}

/// Rounds a run of points into a curve by cutting its corners three times.
/// A run that ends where it starts is treated as a closed loop.
fn rounded(points: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    let closed = points.len() > 2 && points[0] == points[points.len() - 1];
    let mut current = if closed { points[..points.len() - 1].to_vec() } else { points };
    for _ in 0..3 {
        let count = current.len();
        let mut next = Vec::with_capacity(count * 2 + 2);
        let mix = |a: [f32; 2], b: [f32; 2], t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        if !closed {
            next.push(current[0]);
        }
        for i in 0..if closed { count } else { count - 1 } {
            let (a, b) = (current[i], current[(i + 1) % count]);
            next.push(mix(a, b, 0.25));
            next.push(mix(a, b, 0.75));
        }
        if !closed {
            next.push(current[count - 1]);
        }
        current = next;
    }
    if closed {
        current.push(current[0]);
    }
    current
}

/// The line segments of a piece of text, in units, starting at x = 0.
fn segments(text: &str) -> Vec<([f32; 2], [f32; 2])> {
    let mut all = Vec::new();
    let mut shift = 0.0;
    for c in text.chars() {
        let (width, strokes) = glyph(c);
        for stroke in strokes.split('|') {
            let curve = stroke.starts_with('~');
            let numbers: Vec<f32> = stroke.trim_start_matches('~').split_whitespace().filter_map(|n| n.parse().ok()).collect();
            let mut points: Vec<[f32; 2]> = numbers.chunks_exact(2).map(|p| [p[0] + shift, p[1]]).collect();
            if curve {
                points = rounded(points);
            }
            all.extend(points.windows(2).map(|pair| (pair[0], pair[1])));
        }
        shift += width + GAP;
    }
    all
}

/// Width of a piece of text, in units.
pub fn width(text: &str) -> f32 {
    let total: f32 = text.chars().map(|c| glyph(c).0 + GAP).sum();
    (total - GAP).max(0.0)
}

/// How wide a piece of text is for each unit of its height.
pub fn aspect(text: &str) -> f32 {
    (width(text) + 2.0 * STROKE) / (CAP + 2.0 * STROKE)
}

pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    /// How much of each pixel the lettering covers, rows from the top.
    pub cover: Vec<u8>,
}

/// Draws text `height` pixels tall, with smooth edges.
pub fn draw(text: &str, height: usize) -> Bitmap {
    let scale = height as f32 / (CAP + 2.0 * STROKE);
    let width_px = ((width(text) + 2.0 * STROKE) * scale).ceil().max(1.0) as usize;
    let mut cover = vec![0u8; width_px * height];
    for (a, b) in segments(text) {
        let low_x = (a[0].min(b[0]) * scale).floor().max(0.0) as usize;
        let high_x = (((a[0].max(b[0]) + 2.0 * STROKE) * scale).ceil() as usize).min(width_px - 1);
        let low_y = (((CAP - a[1].max(b[1])) * scale).floor().max(0.0) as usize).min(height - 1);
        let high_y = ((((CAP - a[1].min(b[1])) + 2.0 * STROKE) * scale).ceil() as usize).min(height - 1);
        for y in low_y..=high_y {
            for x in low_x..=high_x {
                // Pixel centre, in units, with y up from the baseline.
                let p = [(x as f32 + 0.5) / scale - STROKE, (height as f32 - y as f32 - 0.5) / scale - STROKE];
                let along = [b[0] - a[0], b[1] - a[1]];
                let length_squared = along[0] * along[0] + along[1] * along[1];
                let t = if length_squared == 0.0 { 0.0 } else { (((p[0] - a[0]) * along[0] + (p[1] - a[1]) * along[1]) / length_squared).clamp(0.0, 1.0) };
                let (dx, dy) = (p[0] - a[0] - along[0] * t, p[1] - a[1] - along[1] * t);
                let amount = ((STROKE - (dx * dx + dy * dy).sqrt()) * scale + 0.5).clamp(0.0, 1.0);
                let slot = y * width_px + x;
                cover[slot] = cover[slot].max((amount * 255.0) as u8);
            }
        }
    }
    Bitmap { width: width_px, height, cover }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-/.:+&()";

    #[test]
    fn every_capital_and_digit_has_strokes() {
        for c in ALL.chars() {
            assert!(!segments(&c.to_string()).is_empty(), "{c}");
        }
        assert!(segments(" ").is_empty());
        assert_eq!(segments("a"), segments("A"), "lower case is drawn as capitals");
    }

    #[test]
    fn strokes_stay_inside_each_character() {
        for c in ALL.chars() {
            let wide = glyph(c).0;
            for (a, b) in segments(&c.to_string()) {
                for p in [a, b] {
                    assert!((-0.001..=wide + 0.001).contains(&p[0]) && (-0.001..=6.001).contains(&p[1]), "{c} {p:?}");
                }
            }
        }
    }

    #[test]
    fn the_stroke_is_a_sixth_of_the_height_as_the_standard_asks() {
        assert!((CAP / (2.0 * STROKE) - 6.0).abs() < 1e-6);
        // Measured on the drawing: the stem of an I, halfway up.
        let one = draw("I", 70);
        let thick = one.cover[35 * one.width..36 * one.width].iter().filter(|c| **c > 127).count();
        assert!((9..=11).contains(&thick), "{thick} pixels of a 70 pixel letter");
    }

    #[test]
    fn round_letters_are_round_and_the_i_and_1_are_bare_strokes() {
        // An O has no corners: nothing is inked at its bounding corner.
        let o = draw("O", 70);
        assert_eq!(o.cover[0], 0);
        assert!(o.cover[35 * o.width + 5] > 127, "the left side of the O, halfway up");
        // An I is as wide as its stroke and no wider.
        assert_eq!(draw("I", 70).width, draw("1", 70).width);
        assert!(draw("I", 70).width <= 11);
        // Curves really curve: an O has many more segments than a square would.
        assert!(segments("O").len() > 24);
    }

    #[test]
    fn drawn_text_has_ink_and_longer_text_is_wider() {
        let one = draw("I", 28);
        let four = draw("FUEL", 28);
        assert_eq!(one.height, 28);
        assert!(four.width > 3 * one.width);
        assert!(four.cover.iter().any(|c| *c == 255) && four.cover.iter().any(|c| *c == 0));
        assert_eq!(draw(" ", 28).cover.iter().copied().max(), Some(0));
        assert!((aspect("FUEL") - four.width as f32 / 28.0).abs() < 0.1);
    }
}
