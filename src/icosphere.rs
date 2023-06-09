use nalgebra::Vector3;
use rustc_hash::FxHashMap;

type Vert = [f32; 3];
type Index = [usize; 3];
type Vec3 = Vector3<f32>;

pub struct IcoSphere {
    pub subdiv: usize,
    pub n_face: usize,
    pub n_vert: usize,
    vertices: Vec<Vert>,
    indices: Vec<Index>,
    normals: Vec<Vec3>,
}

const T: f32 = 0.85065080835204;
const X: f32 = 0.52573111211913;

const VERT: [Vert; 12] = [
    [-X, T, 0.], [X, T, 0.], [-X, -T, 0.], [X, -T, 0.],
    [0., -X, T], [0., X, T], [0., -X, -T], [0., X, -T],
    [T, 0., -X], [T, 0., X], [-T, 0., -X], [-T, 0., X],
];

const INDEX: [Index; 20] = [
    [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10],
    [0, 10, 11], [1, 5, 9], [5, 11, 4], [11, 10, 2],
    [10, 7, 6], [7, 1, 8], [3, 9, 4], [3, 4, 2],
    [3, 2, 6], [3, 6, 8], [3, 8, 9], [4, 9, 5],
    [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
];

impl IcoSphere {
    pub fn new(subdiv: usize) -> Self {
        let n_face = 20*4usize.pow(subdiv as u32);
        let n_vert = 3*n_face;
        let mut vertices = VERT.to_vec();
        let mut indices = INDEX.to_vec();
        let mut normals = Vec::<Vec3>::with_capacity(n_vert);
        for _ in 0..subdiv {
            (vertices, indices) = subdivide(vertices, indices);
        }
        for _ in 0..n_vert {
            normals.push(Vec3::zeros());
        }
        for i in 0..n_face {
            let idx = indices[i];
            let a = vertices[idx[0]];
            let b = vertices[idx[1]];
            let c = vertices[idx[2]];
            let va = Vec3::new(a[0], a[1], a[2]);
            let vb = Vec3::new(b[0], b[1], b[2]);
            let vc = Vec3::new(c[0], c[1], c[2]);
            let norm = (vb-va).cross(&(vc-va)).normalize();
            for j in 0..3 {
                normals[idx[j]] += norm;
            }
        }
        for i in 0..n_vert {
            normals[i].normalize_mut();
        }

        Self {subdiv, n_face, n_vert, vertices, indices, normals}
    }

    pub fn normal_buf(&self) -> Vec<f32> {
        let buf_len = 3*self.n_vert;
        let mut buf = Vec::<f32>::with_capacity(buf_len);
        for i in 0..self.n_vert {
            let t_idx = self.indices[i/3];
            let n_idx = t_idx[i%3];
            let norm: Vert = self.normals[n_idx].into();
            buf.push(norm[0]);
            buf.push(norm[1]);
            buf.push(norm[2]);
        }

        buf
    }

    pub fn vertex_buf(&self) -> Vec<f32> {
        let buf_len = 3*self.n_vert;
        let mut buf = Vec::<f32>::with_capacity(buf_len);
        for i in 0..self.n_vert {
            let t_idx = self.indices[i/3];
            let v_idx = t_idx[i%3];
            let vert = self.vertices[v_idx];
            buf.push(vert[0]);
            buf.push(vert[1]);
            buf.push(vert[2]);
        }

        buf
    }
}

fn subdivide(
    mut vertices: Vec<Vert>,
    indices: Vec<Index>,
) -> (Vec<Vert>, Vec<Index>) {
    let mut cache = FxHashMap::<(usize, usize), usize>::default();
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
