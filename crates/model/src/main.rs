//! Writes the model's OBJ8 files into a folder.

use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let folder = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "target/model".into()));
    std::fs::create_dir_all(&folder)?;
    let parts = [("cockpit_envelope.obj", cl650_model::cockpit::envelope())];
    for (name, mesh) in parts {
        let path = folder.join(name);
        std::fs::write(&path, cl650_model::obj8::write(&mesh, None))?;
        println!("{} ({} triangles)", path.display(), mesh.triangle_count());
    }
    Ok(())
}
