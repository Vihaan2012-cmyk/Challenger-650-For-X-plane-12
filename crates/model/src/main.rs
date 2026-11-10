//! Writes the model, its textures, its dataref list, and pictures of it
//! into a folder.
//!
//!     cl650-model [FOLDER] [--textures SIZE] [--no-pictures] [--always-lit]
//!
//! SIZE is the texture size in pixels: 1024, 2048, 4096 or 8192. With
//! --always-lit, screens and legends glow at full brightness whatever the
//! lighting datarefs say, for looking at the model before a plugin exists.

use std::fmt::Write;
use std::path::PathBuf;

use cl650_model::cockpit;
use cl650_model::labels::Atlas;
use cl650_model::materials::finish_for;
use cl650_model::mesh::Mesh;
use cl650_model::obj8::{write_material, Skin};
use cl650_model::part::{Manipulator, Motion, Moving, Part};
use cl650_model::render::{render_with, Camera, Painted};
use cl650_model::texture::{bake, save_png, Finish};

const UP: [f32; 3] = [0.0, 1.0, 0.0];
/// Datarefs that set how brightly things glow, 0 to 1.
const DISPLAY_BRIGHTNESS: &str = "cl650/lighting/displays";
const INTEGRAL_LIGHTING: &str = "cl650/lighting/integral";
const MAP_LIGHTS: [&str; 2] = ["cl650/lighting/map_left", "cl650/lighting/map_right"];
const FLOOD_LIGHTING: &str = "cl650/lighting/flood";
/// The shared palette texture is this many swatches across and down.
const SWATCHES: usize = 16;

/// Geometry that shares one texture set: the fixed part, and the pieces of
/// each moving control that are made of it.
#[derive(Default)]
struct Object<'a> {
    fixed: Mesh,
    moving: Vec<(&'a Moving, Mesh)>,
}

impl<'a> Object<'a> {
    fn add(&mut self, owner: Option<&'a Moving>, mesh: &Mesh) {
        match owner {
            None => self.fixed.append(mesh),
            Some(control) => match self.moving.iter_mut().find(|(o, _)| std::ptr::eq(*o, control)) {
                Some((_, held)) => held.append(mesh),
                None => self.moving.push((control, mesh.clone())),
            },
        }
    }
}

/// Finishes whose grain repeats across a surface need a texture of their
/// own. Every other finish is close to a flat colour, so all of those share
/// one palette texture, each colour a swatch in it.
fn has_grain(finish: Finish) -> bool {
    matches!(finish, Finish::Leather | Finish::Sheepskin | Finish::Carpet | Finish::Lining)
}

fn main() -> Result<(), String> {
    let mut folder = PathBuf::from("target/model");
    let mut texture_size = 1024;
    let mut pictures = true;
    let mut always_lit = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--textures" => texture_size = arguments.next().and_then(|s| s.parse().ok()).ok_or("--textures needs a size in pixels")?,
            "--no-pictures" => pictures = false,
            "--always-lit" => always_lit = true,
            other => folder = PathBuf::from(other),
        }
    }
    let export = folder.join("xplane");
    std::fs::create_dir_all(&export).map_err(|e| e.to_string())?;
    // A light level runs from the first value to the second as its dataref
    // does; from -1 to 0, a dataref that reads zero gives full brightness.
    let write = |name: &str, text: &str| {
        let text = if always_lit { text.replace("ATTR_light_level 0 1 ", "ATTR_light_level -1 0 ") } else { text.to_string() };
        std::fs::write(export.join(name), text).map_err(|e| e.to_string())
    };

    let scene = cockpit::build();
    println!("{} triangles, {} moving controls, {} legends", scene.merged().triangle_count(), scene.moving.len(), scene.labels.len());

    // Sort every piece into the object it belongs to.
    let mut grained: Vec<(Finish, [u8; 3], Object)> = Vec::new();
    let mut swatches: Vec<(Finish, [u8; 3])> = Vec::new();
    let mut palette = Object::default();
    let pieces: Vec<(Option<&Moving>, &Part)> = scene.parts.iter().map(|p| (None, p)).chain(scene.moving.iter().flat_map(|m| m.pieces.iter().map(move |p| (Some(m), p)))).collect();
    for (owner, part) in pieces {
        let finish = finish_for(part.name);
        if has_grain(finish) {
            let index = match grained.iter().position(|g| g.0 == finish && g.1 == part.colour) {
                Some(index) => index,
                None => {
                    grained.push((finish, part.colour, Object::default()));
                    grained.len() - 1
                }
            };
            grained[index].2.add(owner, &part.mesh);
        } else {
            let swatch = match swatches.iter().position(|s| *s == (finish, part.colour)) {
                Some(index) => index,
                None => {
                    swatches.push((finish, part.colour));
                    swatches.len() - 1
                }
            };
            if swatch >= SWATCHES * SWATCHES {
                return Err(format!("more than {} flat colours; the palette is full", SWATCHES * SWATCHES));
            }
            // Every corner of the piece points at the middle of its swatch.
            let mut mesh = part.mesh.clone();
            let centre = [((swatch % SWATCHES) as f32 + 0.5) / SWATCHES as f32, 1.0 - ((swatch / SWATCHES) as f32 + 0.5) / SWATCHES as f32];
            for vertex in &mut mesh.vertices {
                vertex.uv = centre;
            }
            palette.add(owner, &mesh);
        }
    }

    // The palette: colour, a normal map carrying each finish's metalness
    // and gloss, and what glows.
    let palette_size = texture_size.min(2048);
    let cell = palette_size / SWATCHES;
    let (mut colour, mut normal, mut lit) = (vec![255u8; palette_size * palette_size * 4], vec![0u8; palette_size * palette_size * 4], vec![0u8; palette_size * palette_size * 3]);
    for pixel in normal.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[127, 127, 0, 0]);
    }
    for (index, (finish, shade)) in swatches.iter().enumerate() {
        let glow = match finish {
            Finish::Lit => *shade,
            Finish::Screen => [16, 34, 60],
            _ => [0, 0, 0],
        };
        for y in (index / SWATCHES) * cell..(index / SWATCHES + 1) * cell {
            for x in (index % SWATCHES) * cell..(index % SWATCHES + 1) * cell {
                let at = y * palette_size + x;
                // Colour, and in the fourth channel how much it hides: clear covers are see-through.
                colour[at * 4..at * 4 + 4].copy_from_slice(&[shade[0], shade[1], shade[2], (finish.opacity() * 255.0) as u8]);
                lit[at * 3..at * 3 + 3].copy_from_slice(&glow);
                normal[at * 4..at * 4 + 4].copy_from_slice(&[127, 127, (finish.metalness() * 255.0) as u8, (finish.gloss() * 255.0) as u8]);
            }
        }
    }
    save_png(&export.join("cockpit_palette.png"), &colour, palette_size)?;
    save_png(&export.join("cockpit_palette_NML.png"), &normal, palette_size)?;
    save_png(&export.join("cockpit_palette_LIT.png"), &lit, palette_size)?;
    let skin = Skin { texture: "cockpit_palette.png", normal_map: "cockpit_palette_NML.png", lit: Some(("cockpit_palette_LIT.png", DISPLAY_BRIGHTNESS)), tile: 1.0, fixed_uv: true };
    let text = write_material(&palette.fixed, &palette.moving, &skin);
    let mut animations = text.matches("ANIM_begin").count();
    write("cockpit_palette.obj", &text)?;

    for (number, (finish, shade, object)) in grained.iter().enumerate() {
        let name = format!("cockpit_{number:02}_{}", finish.name());
        let (texture, normal_map) = (format!("{name}.png"), format!("{name}_NML.png"));
        let baked = bake(*finish, *shade, texture_size);
        save_png(&export.join(&texture), &baked.albedo, baked.size)?;
        save_png(&export.join(&normal_map), &baked.normal, baked.size)?;
        let skin = Skin { texture: &texture, normal_map: &normal_map, lit: None, tile: finish.tile_size(), fixed_uv: false };
        let text = write_material(&object.fixed, &object.moving, &skin);
        animations += text.matches("ANIM_begin").count();
        write(&format!("{name}.obj"), &text)?;
    }
    println!("{} texture sets ({} flat colours in one palette, {} grained at {texture_size} px), {animations} animated draws, in {}", grained.len() + 1, swatches.len(), grained.len(), export.display());

    // Legends: one image for all the lettering, and one of what glows.
    let atlas = Atlas::build(&scene.labels, texture_size.max(2048))?;
    save_png(&export.join("cockpit_legends.png"), &atlas.colour_image(), atlas.size)?;
    save_png(&export.join("cockpit_legends_LIT.png"), &atlas.lit_image(), atlas.size)?;
    let lit = cl650_model::labels::obj8(&scene.labels, &atlas, "cockpit_legends.png", Some("cockpit_legends_LIT.png"), true);
    write("cockpit_legends_lit.obj", &lit.replacen("\nATTR_poly_os 2\n", &format!("\nATTR_light_level 0 1 {INTEGRAL_LIGHTING}\nATTR_poly_os 2\n"), 1))?;
    write("cockpit_legends_printed.obj", &cl650_model::labels::obj8(&scene.labels, &atlas, "cockpit_legends.png", None, false))?;

    // Lamps that light the cockpit: the two map lights, and a flood light
    // under the glareshield and overhead. Position, colour, size, direction,
    // spread and the dataref that sets its brightness.
    let [_, eye_y, eye_z] = cockpit::eye(-1.0);
    let mut lights = String::from("I\n800\nOBJ\n\nPOINT_COUNTS 0 0 0 0\n\n");
    for (side, dataref) in [(-1.0, MAP_LIGHTS[0]), (1.0, MAP_LIGHTS[1])] {
        let _ = writeln!(lights, "LIGHT_SPILL_CUSTOM {:.3} {:.3} {:.3} 1.0 0.95 0.85 1.0 1.2 {:.2} -0.90 0.35 0.75 {dataref}", side * 0.26, eye_y + 0.33, eye_z - 0.30, side * 0.25);
    }
    let _ = writeln!(lights, "LIGHT_SPILL_CUSTOM 0.000 {:.3} {:.3} 1.0 0.97 0.92 1.0 2.0 0.0 -1.0 0.0 0.0 {FLOOD_LIGHTING}", eye_y + 0.30, eye_z - 0.20);
    write("cockpit_lights.obj", &lights)?;

    // Every dataref the model reads, for the plugin to create and drive.
    let mut list = String::from("dataref\ttype\trange\tworks as\tmoves\n");
    for control in &scene.moving {
        let works = match control.manipulator {
            Manipulator::Toggle => "click to flip",
            Manipulator::Push => "press and hold",
            Manipulator::Drag => "drag",
        };
        let moves = match control.motion {
            Motion::Rotate { degrees, .. } => format!("turns {:.0} to {:.0} degrees", degrees.0, degrees.1),
            Motion::Slide { metres, .. } => format!("slides {:.1} to {:.1} mm", metres.0 * 1000.0, metres.1 * 1000.0),
        };
        let _ = writeln!(list, "{}\tfloat\t0..1\t{works}\t{moves}", control.dataref);
    }
    for dataref in [DISPLAY_BRIGHTNESS, INTEGRAL_LIGHTING, MAP_LIGHTS[0], MAP_LIGHTS[1], FLOOD_LIGHTING] {
        let _ = writeln!(list, "{dataref}\tfloat\t0..1\tlighting level\t");
    }
    std::fs::write(folder.join("datarefs.tsv"), list).map_err(|e| e.to_string())?;

    if !pictures {
        return Ok(());
    }
    let (y, z) = (eye_y, eye_z);
    let views = [
        ("cockpit_from_behind.png", Camera { eye: [0.0, y + 0.42, z + 0.47], target: [0.0, y - 0.30, z - 0.60], up: UP, field_of_view: 86.0 }, false),
        ("cockpit_night.png", Camera { eye: [0.0, y + 0.42, z + 0.47], target: [0.0, y - 0.30, z - 0.60], up: UP, field_of_view: 86.0 }, true),
        ("cockpit_pilot.png", Camera { eye: cockpit::eye(-1.0), target: [-0.30, y - 0.22, z - 0.80], up: UP, field_of_view: 86.0 }, false),
        ("cockpit_looking_aft.png", Camera { eye: [0.0, y + 0.05, z - 0.55], target: [0.0, y - 0.15, z + 0.9], up: UP, field_of_view: 78.0 }, false),
        ("cockpit_side.png", Camera { eye: [-9.0, 0.15, z - 0.45], target: [0.0, 0.15, z - 0.45], up: UP, field_of_view: 15.0 }, false),
        ("closeup_overhead.png", Camera { eye: [0.0, y - 0.05, z - 0.02], target: [0.0, y + 0.27, z - 0.25], up: UP, field_of_view: 52.0 }, false),
        ("closeup_overhead_night.png", Camera { eye: [0.0, y - 0.05, z - 0.02], target: [0.0, y + 0.27, z - 0.25], up: UP, field_of_view: 52.0 }, true),
        ("closeup_panel.png", Camera { eye: [0.0, y, z - 0.18], target: [0.0, y - 0.38, z - 0.74], up: UP, field_of_view: 64.0 }, false),
        ("closeup_glareshield.png", Camera { eye: [0.0, y - 0.02, z - 0.12], target: [0.0, y - 0.19, z - 0.60], up: UP, field_of_view: 58.0 }, false),
        ("closeup_pedestal.png", Camera { eye: [0.0, y + 0.12, z - 0.02], target: [0.0, y - 0.70, z - 0.42], up: UP, field_of_view: 56.0 }, false),
        ("closeup_quadrant.png", Camera { eye: [0.22, y - 0.25, z - 0.12], target: [0.0, y - 0.68, z - 0.42], up: UP, field_of_view: 44.0 }, false),
        ("closeup_captain.png", Camera { eye: [0.15, y + 0.12, z + 0.40], target: [-0.48, y - 0.36, z - 0.45], up: UP, field_of_view: 60.0 }, false),
        ("closeup_breakers.png", Camera { eye: [-0.30, y - 0.05, z + 0.05], target: [-0.63, y + 0.08, z + 0.50], up: UP, field_of_view: 70.0 }, false),
    ];
    let everything = scene.everything();
    let parts: Vec<Painted> = everything.iter().map(|p| Painted { mesh: &p.mesh, colour: p.colour, finish: finish_for(p.name) }).collect();
    for (name, camera, night) in views {
        let path = folder.join(name);
        render_with(&parts, Some((&scene.labels, &atlas)), &camera, 2560, 1600, night).halved().save_png(&path)?;
        println!("{}", path.display());
    }
    Ok(())
}
