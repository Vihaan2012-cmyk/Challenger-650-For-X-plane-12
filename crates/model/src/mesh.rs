//! Triangle meshes. Triangles are counter-clockwise when seen from the side
//! their normals point to; exporters convert to what each simulator wants.

pub type Vec3 = [f32; 3];

pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn normalised(a: Vec3) -> Vec3 {
    let length = dot(a, a).sqrt();
    if length == 0.0 {
        a
    } else {
        [a[0] / length, a[1] / length, a[2] / length]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Adds a triangle, ordering its corners so it faces the way its normals
    /// point.
    pub fn triangle(&mut self, mut corners: [Vertex; 3]) {
        let face = cross(sub(corners[1].position, corners[0].position), sub(corners[2].position, corners[0].position));
        let wanted = add(add(corners[0].normal, corners[1].normal), corners[2].normal);
        if dot(face, wanted) < 0.0 {
            corners.swap(1, 2);
        }
        let base = self.vertices.len() as u32;
        self.vertices.extend(corners);
        self.indices.extend([base, base + 1, base + 2]);
    }

    /// Adds a four-sided face whose corners are given in order around its
    /// edge, either way round.
    pub fn quad(&mut self, mut corners: [Vertex; 4]) {
        let face = cross(sub(corners[1].position, corners[0].position), sub(corners[2].position, corners[0].position));
        let wanted = corners.iter().fold([0.0; 3], |sum, v| add(sum, v.normal));
        if dot(face, wanted) < 0.0 {
            corners.swap(1, 3);
        }
        let base = self.vertices.len() as u32;
        self.vertices.extend(corners);
        self.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Adds a flat convex face from its outline, given in order either way
    /// round.
    pub fn convex(&mut self, outline: &[Vec3], normal: Vec3, uv: impl Fn(Vec3) -> [f32; 2]) {
        let vertex = |position: Vec3| Vertex { position, normal, uv: uv(position) };
        for i in 1..outline.len().saturating_sub(1) {
            self.triangle([vertex(outline[0]), vertex(outline[i]), vertex(outline[i + 1])]);
        }
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    /// Lowest and highest corner of the box around the mesh.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let first = self.vertices.first()?.position;
        Some(self.vertices.iter().fold((first, first), |(mut low, mut high), v| {
            for axis in 0..3 {
                low[axis] = low[axis].min(v.position[axis]);
                high[axis] = high[axis].max(v.position[axis]);
            }
            (low, high)
        }))
    }

    /// True when every triangle faces the way its corners' normals point.
    pub fn faces_agree_with_normals(&self) -> bool {
        self.indices.chunks_exact(3).all(|t| {
            let [a, b, c] = [self.vertices[t[0] as usize], self.vertices[t[1] as usize], self.vertices[t[2] as usize]];
            let face = cross(sub(b.position, a.position), sub(c.position, a.position));
            dot(face, add(add(a.normal, b.normal), c.normal)) > 0.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn up(position: Vec3) -> Vertex {
        Vertex { position, normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] }
    }

    #[test]
    fn faces_point_the_way_their_normals_do_whichever_way_round_they_are_given() {
        let square = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let mut reversed = square;
        reversed.reverse();
        for corners in [square, reversed] {
            let mut mesh = Mesh::default();
            mesh.quad(corners.map(up));
            mesh.triangle([up(corners[0]), up(corners[1]), up(corners[2])]);
            mesh.convex(&corners, [0.0, 1.0, 0.0], |_| [0.0, 0.0]);
            assert_eq!(mesh.triangle_count(), 5);
            assert!(mesh.faces_agree_with_normals());
        }
    }

    #[test]
    fn appending_keeps_indices_pointing_at_the_right_vertices() {
        let mut a = Mesh::default();
        a.triangle([up([0.0, 0.0, 0.0]), up([0.0, 0.0, 1.0]), up([1.0, 0.0, 0.0])]);
        let b = a.clone();
        a.append(&b);
        assert_eq!(a.indices[3..], [3, 4, 5]);
        assert_eq!(a.bounds(), Some(([0.0, 0.0, 0.0], [1.0, 0.0, 1.0])));
    }
}
