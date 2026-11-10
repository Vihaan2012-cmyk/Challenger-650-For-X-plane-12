//! X-Plane OBJ8 export.

use std::fmt::Write;

use crate::mesh::Mesh;

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
}
