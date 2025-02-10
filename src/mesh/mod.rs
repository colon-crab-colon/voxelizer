use bvh::{bounding_hierarchy::BoundingHierarchy, bvh::Bvh, ray::Ray};
use image::{ImageReader, RgbaImage};
use indicatif::ProgressBar;
use nalgebra::{OPoint, SVector, Vector2, Vector3};
use texture::Texture;
use triangle::Triangle;

use crate::bbox::Bbox;

pub mod texture;
pub mod triangle;

pub struct Mesh {
    triangles: Vec<Triangle>,
    bvh: Bvh<f32, 3>,
    bbox: Bbox,
    texture: Option<Texture>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vector3<f32>>,
        indices: Vec<usize>,
        coords: Option<Vec<Vector2<f32>>>,
        texture: Option<Texture>,
    ) -> Self {
        let mut triangles = Vec::with_capacity(indices.len() / 3);

        for indices in indices.chunks(3) {
            let coords = match coords {
                Some(ref coords) => [coords[indices[0]], coords[indices[1]], coords[indices[2]]],
                None => [Vector2::new(0f32, 0f32); 3],
            };

            triangles.push(Triangle::new(
                vertices[indices[0]],
                vertices[indices[1]],
                vertices[indices[2]],
                coords[0],
                coords[1],
                coords[2],
            ));
        }

        let mut mesh = Self {
            bvh: Bvh::build_par(&mut triangles),
            triangles,
            texture,
            bbox: Bbox::new(Vector3::zeros(), Vector3::zeros()),
        };

        mesh.bbox = Bbox::from_mesh(&mesh);

        mesh
    }

    pub fn rotate(&mut self, rotation: Vector3<f32>) {
        let rotation_matrix =
            nalgebra::Rotation3::from_euler_angles(rotation.x, rotation.y, rotation.z);

        for triangle in &mut self.triangles {
            triangle.position_a = rotation_matrix * triangle.position_a;
            triangle.position_b = rotation_matrix * triangle.position_b;
            triangle.position_c = rotation_matrix * triangle.position_c;
        }

        self.bvh = Bvh::build(&mut self.triangles);

        self.bbox = Bbox::from_mesh(self);
    }

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn voxelize_shell(
        &self,
        resolution: f32,
        bar: &ProgressBar,
    ) -> Vec<(Vector3<i32>, [u8; 4])> {
        let min = Vector3::new(
            (self.bbox.min.x / resolution) as i32 - 1,
            (self.bbox.min.y / resolution) as i32 - 1,
            (self.bbox.min.z / resolution) as i32 - 1,
        );
        let max = Vector3::new(
            (self.bbox.max.x / resolution).ceil() as i32 + 1,
            (self.bbox.max.y / resolution).ceil() as i32 + 1,
            (self.bbox.max.z / resolution).ceil() as i32 + 1,
        );

        let mut voxels = Vec::new();

        let texture = match self.texture.clone() {
            Some(texture) => match texture {
                Texture::Raw(raw) => Some(image::load_from_memory(&raw).unwrap().to_rgba8()),
                Texture::Path(p) => Some(
                    ImageReader::open(p)
                        .unwrap()
                        .with_guessed_format()
                        .unwrap()
                        .decode()
                        .unwrap()
                        .to_rgba8(),
                ),
            },
            None => None,
        };

        let n = (max.y - min.y) + (max.x - min.x) + (max.x - min.x);

        bar.set_length(n as u64);
        bar.set_position(0);

        // Voxelizes along the x axis
        for y in min.y..max.y {
            for z in min.z..max.z {
                let origin = Vector3::new(
                    min.x as f32 * resolution,
                    y as f32 * resolution,
                    z as f32 * resolution,
                );

                let ray = Ray::new(OPoint::from(origin), *SVector::x_axis());

                for triangle in self.bvh.traverse(&ray, &self.triangles) {
                    if let Some(intersection) = triangle.intersects(&ray) {
                        let color = match &texture {
                            Some(texture) => {
                                let coords = triangle.intersection_to_uv(&intersection);

                                sample_texture(texture, coords.x, coords.y)
                            }
                            None => [255u8; 4],
                        };

                        let point = origin + Vector3::x_axis().scale(intersection.distance);
                        voxels.push((voxel(&point, resolution), color));
                    }
                }
            }

            bar.inc(1);
        }

        // Voxelizes along the y axis
        for x in min.x..max.x {
            for z in min.z..max.z {
                let origin = Vector3::new(
                    x as f32 * resolution,
                    min.y as f32 * resolution,
                    z as f32 * resolution,
                );

                let ray = Ray::new(OPoint::from(origin), *SVector::y_axis());

                for triangle in self.bvh.traverse(&ray, &self.triangles) {
                    if let Some(intersection) = triangle.intersects(&ray) {
                        let color = match &texture {
                            Some(texture) => {
                                let coords = triangle.intersection_to_uv(&intersection);

                                sample_texture(texture, coords.x, coords.y)
                            }
                            None => [255u8; 4],
                        };

                        let point = origin + Vector3::y_axis().scale(intersection.distance);
                        voxels.push((voxel(&point, resolution), color));
                    }
                }
            }

            bar.inc(1);
        }

        // Voxelizes along the z axis
        for x in min.x..max.x {
            for y in min.y..max.y {
                let origin = Vector3::new(
                    x as f32 * resolution,
                    y as f32 * resolution,
                    min.z as f32 * resolution,
                );

                let ray = Ray::new(OPoint::from(origin), *SVector::z_axis());

                for triangle in self.bvh.traverse(&ray, &self.triangles) {
                    if let Some(intersection) = triangle.intersects(&ray) {
                        let color = match &texture {
                            Some(texture) => {
                                let coords = triangle.intersection_to_uv(&intersection);

                                sample_texture(texture, coords.x, coords.y)
                            }
                            None => [255u8; 4],
                        };

                        let point = origin + Vector3::z_axis().scale(intersection.distance);
                        voxels.push((voxel(&point, resolution), color));
                    }
                }
            }

            bar.inc(1);
        }

        voxels
    }
}

pub fn sample_texture(image: &RgbaImage, u: f32, v: f32) -> [u8; 4] {
    image
        .get_pixel(
            ((image.width() as f32 * u) as u32).min(image.width() - 1),
            ((image.height() as f32 * v) as u32).min(image.height() - 1),
        )
        .0
}

fn voxel(pos: &Vector3<f32>, resolution: f32) -> Vector3<i32> {
    Vector3::new(
        (pos.x / resolution).round() as i32,
        (pos.y / resolution).round() as i32,
        (pos.z / resolution).round() as i32,
    )
}
