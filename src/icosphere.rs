use nalgebra::Vector3;
use std::collections::HashMap;

type Vert = [f32; 3];
type Index = [usize; 3];
type Vec3 = Vector3<f32>;

pub struct IcoSphere {
    subdiv: usize,
    vertices: Vec<Vert>,
    indices: Vec<Index>,
}

const T: f32 = 0.85065080835204;
const X: f32 = 0.5257311121191336;

const VERT: [Vert; 12] = [
    [-X, T, 0.], [X, T, 0.], [-X, -T, 0.], [X, -T, 0.],
    [0., -X, T], [0., X, T], [0., -X, -T], [0., X, -T],
    [T, 0., -X], [T, 0., X], [-T, 0., -X], [-T, 0., X],
];

const IDX: [Index; 20] = [
    [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10],
    [0, 10, 11], [1, 5, 9], [5, 11, 4], [11, 10, 2],
    [10, 7, 6], [7, 1, 8], [3, 9, 4], [3, 4, 2],
    [3, 2, 6], [3, 6, 8], [3, 8, 9], [4, 9, 5],
    [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
];

impl IcoSphere {
    pub fn new(subdiv: usize) -> Self {
        let mut vertices = VERT.to_vec();
        let mut indices = IDX.to_vec();
        for _ in 0..subdiv {
            (vertices, indices) = subdivide(vertices, indices);
        }

        Self {subdiv, vertices, indices}
    }

    pub fn normals(&self) -> Vec<f32> {
        let n_faces = 20*(4 as usize).pow(self.subdiv as u32);
        let n_indices = 3*n_faces;
        let buf_len = 3*n_indices;
        let mut normals = Vec::<Vec3>::with_capacity(n_indices);
        let mut buf = Vec::<f32>::with_capacity(buf_len);
        for _ in 0..n_indices {
            normals.push(Vec3::zeros());
        }
        for i in 0..n_faces {
            let idx = self.indices[i];
            let a = self.vertices[idx[0]];
            let b = self.vertices[idx[1]];
            let c = self.vertices[idx[2]];
            let va = Vec3::new(a[0], a[1], a[2]);
            let vb = Vec3::new(b[0], b[1], b[2]);
            let vc = Vec3::new(c[0], c[1], c[2]);
            let norm = (vb-va).cross(&(vc-va)).normalize();
            for j in 0..3 {
                normals[idx[j]] += norm;
            }
        }
        for i in 0..n_indices {
            let t_idx = self.indices[i/3];
            let n_idx = t_idx[i%3];
            let norm: Vert = normals[n_idx].normalize().into();
            buf.push(norm[0]);
            buf.push(norm[1]);
            buf.push(norm[2]);
        }

        buf
    }

    pub fn buffer(&self) -> (i32, Vec<f32>) {
        let n_faces = 20*(4 as usize).pow(self.subdiv as u32);
        let n_indices = 3*n_faces;
        let buf_len = 3*n_indices;
        let mut buf = Vec::<f32>::with_capacity(buf_len);
        for i in 0..n_indices {
            let t_idx = self.indices[i/3];
            let v_idx = t_idx[i%3];
            let vert = self.vertices[v_idx];
            buf.push(vert[0]);
            buf.push(vert[1]);
            buf.push(vert[2]);
        }

        (n_indices as i32, buf)
    }
}

fn subdivide(
    mut vertices: Vec<Vert>,
    indices: Vec<Index>,
) -> (Vec<Vert>, Vec<Index>) {
    let mut cache = HashMap::<(usize, usize), usize>::default();
    let mut nindices = Vec::<Index>::default();
    for idx in &indices {
        let mut mid: Index = [0; 3];
        for i in 0..3 {
            let pair = (idx[i], idx[(i+1)%3]);
            mid[i] = match cache.get(&pair) {
                Some(i) => *i,
                None => vertices.len(),
            };
            if mid[i] == vertices.len() {
                cache.insert(pair, mid[i]);
                cache.insert((pair.1, pair.0), mid[i]);
                let start = vertices[pair.0];
                let end = vertices[pair.1];
                let new: Vert = Vec3::new(
                    start[0]+end[0],
                    start[1]+end[1],
                    start[2]+end[2],
                ).normalize().into();
                vertices.push(new);
            }
        }
        nindices.push([idx[0], mid[0], mid[2]]);
        nindices.push([idx[1], mid[1], mid[0]]);
        nindices.push([idx[2], mid[2], mid[1]]);
        nindices.push([mid[0], mid[1], mid[2]]);
    }
    (vertices, nindices)
}
