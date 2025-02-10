use nalgebra::Vector3;

use crate::mesh::Mesh;

#[derive(Clone, Copy, Debug)]
pub struct Bbox {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Bbox {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }

    pub fn from_mesh(mesh: &Mesh) -> Self {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for triangle in mesh.triangles() {
            min = min
                .inf(&triangle.position_a)
                .inf(&triangle.position_b)
                .inf(&triangle.position_c);

            max = max
                .sup(&triangle.position_a)
                .sup(&triangle.position_b)
                .sup(&triangle.position_c)
        }

        Bbox { min, max }
    }

    pub fn from_pcl(pcl: &[Vector3<f32>]) -> Self {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for p in pcl {
            min = min.inf(p);
            max = max.sup(p);
        }

        Self { min, max }
    }
}
