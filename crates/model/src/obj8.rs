//! X-Plane OBJ8 export.

use std::fmt::Write;

use crate::mesh::Mesh;
use crate::part::{Manipulator, Motion, Moving};

/// The mesh as the text of an OBJ8 file. X-Plane wants triangles clockwise
/// when seen from the front, the reverse of how `Mesh` holds them.
pub fn write(mesh: &Mesh, texture: Option<&str>) -> String {
    let mut out = String::from("I\n800\nOBJ\n\n");
    if let Some(texture) = texture {
        let _ = writeln!(out, "TEXTURE {texture}");
    }
    let _ = writeln!(out, "POINT_COUNTS {} 0 0 {}\n", mesh.vertices.len(), mesh.indices.len());
    for v in &mesh.vertices {
        let [x, y, z] = v.position;
        let [nx, ny, nz] = v.normal;
        let _ = writeln!(out, "VT {x:.5} {y:.5} {z:.5} {nx:.4} {ny:.4} {nz:.4} {:.5} {:.5}", v.uv[0], v.uv[1]);
    }
    out.push('\n');
    let reversed: Vec<u32> = mesh.indices.chunks_exact(3).flat_map(|t| [t[0], t[2], t[1]]).collect();
    let mut tens = reversed.chunks_exact(10);
    for ten in &mut tens {
        out.push_str("IDX10");
        for index in ten {
            let _ = write!(out, " {index}");
        }
        out.push('\n');
    }
    for index in tens.remainder() {
        let _ = writeln!(out, "IDX {index}");
    }
    let _ = writeln!(out, "\nTRIS 0 {}", reversed.len());
    out
}

/// The mesh as an OBJ8 file that uses a colour texture and a normal map
/// carrying metalness and gloss. Texture coordinates are laid out by the
/// same rule the renderer uses, with one repeat of the texture every
/// `tile` metres.
pub fn write_textured(mesh: &Mesh, texture: &str, normal_map: &str, tile: f32) -> String {
    let mut laid_out = mesh.clone();
    for vertex in &mut laid_out.vertices {
        vertex.uv = crate::render::box_coordinates(vertex.position, vertex.normal, tile).0;
    }
    let plain = write(&laid_out, Some(texture));
    let header = format!("TEXTURE {texture}\n");
    plain.replacen(&header, &format!("{header}TEXTURE_NORMAL {normal_map}\nGLOBAL_specular 1.0\nNORMAL_METALNESS\n"), 1)
}

/// What a material object draws: the texture set, and optionally an image
/// of what glows with the dataref that sets how brightly.
#[derive(Clone, Copy)]
pub struct Skin<'a> {
    pub texture: &'a str,
    pub normal_map: &'a str,
    pub lit: Option<(&'a str, &'a str)>,
    /// Metres covered by one repeat of the texture.
    pub tile: f32,
    /// Use the texture coordinates the geometry already has, and do not
    /// lay it out by position: for geometry that points at a palette.
    pub fixed_uv: bool,
}

/// One material's geometry as an OBJ8 file: the fixed part, then each
/// moving control's pieces inside its own animation, with the click region
/// that works it. A control turns or slides as its dataref goes 0 to 1.
pub fn write_material(fixed: &Mesh, moving: &[(&Moving, Mesh)], skin: &Skin) -> String {
    let mut all = fixed.clone();
    let mut ranges = Vec::with_capacity(moving.len());
    for (_, mesh) in moving {
        ranges.push((all.indices.len(), mesh.indices.len()));
        all.append(mesh);
    }
    let mut out = String::from("I\n800\nOBJ\n\n");
    let _ = writeln!(out, "TEXTURE {}\nTEXTURE_NORMAL {}", skin.texture, skin.normal_map);
    if let Some((lit, _)) = skin.lit {
        let _ = writeln!(out, "TEXTURE_LIT {lit}");
    }
    let _ = writeln!(out, "GLOBAL_specular 1.0\nNORMAL_METALNESS\nPOINT_COUNTS {} 0 0 {}\n", all.vertices.len(), all.indices.len());
    for v in &all.vertices {
        let [x, y, z] = v.position;
        let [nx, ny, nz] = v.normal;
        let uv = if skin.fixed_uv { v.uv } else { crate::render::box_coordinates(v.position, v.normal, skin.tile).0 };
        let _ = writeln!(out, "VT {x:.5} {y:.5} {z:.5} {nx:.4} {ny:.4} {nz:.4} {:.5} {:.5}", uv[0], uv[1]);
    }
    out.push('\n');
    // Clockwise, as X-Plane wants.
    for t in all.indices.chunks_exact(3) {
        let _ = writeln!(out, "IDX {}\nIDX {}\nIDX {}", t[0], t[2], t[1]);
    }
    out.push('\n');
    if let Some((_, level)) = skin.lit {
        let _ = writeln!(out, "ATTR_light_level 0 1 {level}");
    }
    let _ = writeln!(out, "TRIS 0 {}", fixed.indices.len());
    for ((control, _), (start, count)) in moving.iter().zip(ranges) {
        let [px, py, pz] = control.pivot;
        let tip = control.dataref.rsplit('/').next().unwrap_or("control");
        let _ = writeln!(out, "ANIM_begin");
        let drag = match control.motion {
            Motion::Rotate { axis, degrees } => {
                let _ = writeln!(out, "ANIM_trans {px:.5} {py:.5} {pz:.5} {px:.5} {py:.5} {pz:.5} 0 0 none");
                let _ = writeln!(out, "ANIM_rotate {:.4} {:.4} {:.4} {:.2} {:.2} 0 1 {}", axis[0], axis[1], axis[2], degrees.0, degrees.1, control.dataref);
                let _ = writeln!(out, "ANIM_trans {:.5} {:.5} {:.5} {:.5} {:.5} {:.5} 0 0 none", -px, -py, -pz, -px, -py, -pz);
                [0.0, 0.05, 0.0]
            }
            Motion::Slide { along, metres } => {
                let (a, b) = (along.map(|c| c * metres.0 + 0.0), along.map(|c| c * metres.1 + 0.0));
                let _ = writeln!(out, "ANIM_trans {:.5} {:.5} {:.5} {:.5} {:.5} {:.5} 0 1 {}", a[0], a[1], a[2], b[0], b[1], b[2], control.dataref);
                along.map(|c| c * (metres.1 - metres.0).abs().max(0.03))
            }
        };
        let _ = match control.manipulator {
            Manipulator::Toggle => writeln!(out, "ATTR_manip_toggle hand 1 0 {} {tip}", control.dataref),
            Manipulator::Push => writeln!(out, "ATTR_manip_push hand 1 0 {} {tip}", control.dataref),
            Manipulator::Drag => writeln!(out, "ATTR_manip_drag_axis hand {:.4} {:.4} {:.4} 0 1 {} {tip}", drag[0], drag[1], drag[2], control.dataref),
        };
        let _ = writeln!(out, "TRIS {start} {count}\nATTR_manip_none\nANIM_end");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Vertex;

    fn four_triangles() -> Mesh {
        let up = |position| Vertex { position, normal: [0.0, 1.0, 0.0], uv: [0.25, 0.75] };
        let mut mesh = Mesh::default();
        for i in 0..4 {
            let z = i as f32;
            mesh.triangle([up([0.0, 0.0, z]), up([0.0, 0.0, z + 1.0]), up([1.0, 0.0, z])]);
        }
        mesh
    }

    #[test]
    fn the_file_has_the_header_counts_and_every_index() {
        let text = write(&four_triangles(), Some("cockpit.png"));
        assert!(text.starts_with("I\n800\nOBJ\n"));
        assert!(text.contains("TEXTURE cockpit.png\n"));
        assert!(text.contains("POINT_COUNTS 12 0 0 12\n"));
        assert_eq!(text.lines().filter(|l| l.starts_with("VT ")).count(), 12);
        assert_eq!(text.lines().filter(|l| l.starts_with("IDX10 ")).count(), 1);
        assert_eq!(text.lines().filter(|l| l.starts_with("IDX ")).count(), 2);
        assert!(text.trim_end().ends_with("TRIS 0 12"));
    }

    #[test]
    fn triangles_are_written_clockwise() {
        let mesh = four_triangles();
        let text = write(&mesh, None);
        let first = text.lines().find(|l| l.starts_with("IDX10 ")).unwrap();
        let written: Vec<u32> = first.split_whitespace().skip(1).take(3).map(|n| n.parse().unwrap()).collect();
        assert_eq!(written, [mesh.indices[0], mesh.indices[2], mesh.indices[1]]);
        assert!(!text.contains("TEXTURE"));
    }

    #[test]
    fn a_textured_file_names_both_images_and_lays_out_coordinates() {
        let text = write_textured(&four_triangles(), "leather.png", "leather_NML.png", 0.5);
        assert!(text.contains("TEXTURE leather.png
TEXTURE_NORMAL leather_NML.png
GLOBAL_specular 1.0
NORMAL_METALNESS
"));
        // The vertex at x = 1, z = 0 on a face pointing up is two repeats along u.
        assert!(text.lines().any(|l| l.starts_with("VT 1.00000 0.00000 0.00000") && l.ends_with("2.00000 0.00000")), "{text}");
    }

    #[test]
    fn a_moving_control_is_written_inside_its_animation_with_its_click_region() {
        let knob = Moving { dataref: "test/knob".into(), pivot: [1.0, 2.0, 3.0], motion: Motion::Rotate { axis: [0.0, 1.0, 0.0], degrees: (-150.0, 150.0) }, manipulator: Manipulator::Drag, pieces: vec![] };
        let button = Moving { dataref: "test/button".into(), pivot: [0.0; 3], motion: Motion::Slide { along: [0.0, -1.0, 0.0], metres: (0.0, 0.002) }, manipulator: Manipulator::Push, pieces: vec![] };
        let fixed = four_triangles();
        let moving = [(&knob, four_triangles()), (&button, four_triangles())];
        let skin = Skin { texture: "a.png", normal_map: "a_NML.png", lit: Some(("a_LIT.png", "test/brightness")), tile: 0.5, fixed_uv: false };
        let text = write_material(&fixed, &moving, &skin);
        assert!(text.contains("POINT_COUNTS 36 0 0 36"));
        assert!(text.contains("TEXTURE_LIT a_LIT.png") && text.contains("ATTR_light_level 0 1 test/brightness
TRIS 0 12"));
        assert!(text.contains("ANIM_rotate 0.0000 1.0000 0.0000 -150.00 150.00 0 1 test/knob"));
        assert!(text.contains("ANIM_trans 1.00000 2.00000 3.00000 1.00000 2.00000 3.00000 0 0 none"));
        assert!(text.contains("ATTR_manip_drag_axis hand") && text.contains("test/knob knob
TRIS 12 12"));
        assert!(text.contains("ANIM_trans 0.00000 0.00000 0.00000 0.00000 -0.00200 0.00000 0 1 test/button"));
        assert!(text.contains("ATTR_manip_push hand 1 0 test/button button
TRIS 24 12"));
        assert_eq!(text.matches("ANIM_begin").count(), 2);
        assert_eq!(text.matches("ANIM_end").count(), 2);
        // Geometry that points at a palette keeps the coordinates it was given.
        let pointed = write_material(&fixed, &[], &Skin { fixed_uv: true, ..skin });
        assert!(pointed.lines().filter(|l| l.starts_with("VT ")).all(|l| l.ends_with("0.25000 0.75000")));
    }
}
