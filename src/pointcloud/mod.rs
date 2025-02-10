use ahash::AHashMap;
use indicatif::ProgressBar;
use nalgebra::Vector3;

pub struct PointCloud {
    points: Vec<(Vector3<f32>, [u8; 4])>,
}

impl PointCloud {
    pub fn new(points: Vec<(Vector3<f32>, [u8; 4])>) -> Self {
        Self { points }
    }

    pub fn voxelize(&self, resolution: f32, bar: &ProgressBar) -> Vec<(Vector3<i32>, [u8; 4])> {
        let mut set: AHashMap<Vector3<i32>, [u8; 4]> = AHashMap::new();

        bar.set_length(self.points.len() as u64);

        for (point, color) in &self.points {
            let pos = Vector3::new(
                (point.x / resolution).round() as i32,
                (point.y / resolution).round() as i32,
                (point.z / resolution).round() as i32,
            );

            bar.inc(1);

            set.insert(pos, *color);
        }

        set.into_iter().collect::<Vec<(Vector3<i32>, [u8; 4])>>()
    }

    pub fn rotate(&mut self, rotation: Vector3<f32>) {
        let rotation_matrix =
            nalgebra::Rotation3::from_euler_angles(rotation.x, rotation.y, rotation.z);

        for (point, _) in &mut self.points {
            *point = rotation_matrix * *point;
        }
    }
}
