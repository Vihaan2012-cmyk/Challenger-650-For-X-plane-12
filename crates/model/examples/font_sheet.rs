//! Draws every character of the panel lettering on one sheet, for checking
//! it against the standard.

use cl650_model::font::draw;

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "target/model/font_sheet.png".into());
    let rows = ["ABCDEFG", "HIJKLMN", "OPQRSTU", "VWXYZ", "012345", "6789", "FUEL BOOST PUMP", "A/COLL 1/2 BNK & (+) -.:"];
    let (letter, gap, width) = (96usize, 34usize, 1300usize);
    let height = rows.len() * (letter + gap) + gap;
    let mut pixels = vec![255u8; width * height];
    for (index, row) in rows.iter().enumerate() {
        let spaced: String = if index < 6 { row.chars().flat_map(|c| [c, ' ']).collect() } else { row.to_string() };
        let bitmap = draw(&spaced, letter);
        let top = gap + index * (letter + gap);
        for y in 0..bitmap.height {
            for x in 0..bitmap.width.min(width - 2 * gap) {
                pixels[(top + y) * width + gap + x] = 255 - bitmap.cover[y * bitmap.width + x];
            }
        }
    }
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().map_err(|e| e.to_string())?.write_image_data(&pixels).map_err(|e| e.to_string())?;
    println!("{path}");
    Ok(())
}
