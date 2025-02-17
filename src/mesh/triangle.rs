use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
    ray::{Intersection, Ray},
};
use nalgebra::{OPoint, SimdValue, Vector2, Vector3};

pub struct Triangle {
    pub position_a: Vector3<f32>,
    pub position_b: Vector3<f32>,
    pub position_c: Vector3<f32>,
    pub texture_a: Vector2<f32>,
    pub texture_b: Vector2<f32>,
    pub texture_c: Vector2<f32>,
    index: usize,
}

impl Triangle {
    pub fn new(
        position_a: Vector3<f32>,
        position_b: Vector3<f32>,
        position_c: Vector3<f32>,
        texture_a: Vector2<f32>,
        texture_b: Vector2<f32>,
        texture_c: Vector2<f32>,
    ) -> Self {
        Self {
            position_a,
            position_b,
            position_c,
            texture_a,
            texture_b,
            texture_c,
            index: 0,
        }
    }

    pub fn intersection_to_uv(&self, intersection: &Intersection<f32>) -> Vector2<f32> {
        let alpha = 1.0 - intersection.u - intersection.v;
        let beta = intersection.u;
        let gamma = intersection.v;

        Vector2::new(
            alpha * self.texture_a.x + beta * self.texture_b.x + gamma * self.texture_c.x,
            alpha * self.texture_a.y + beta * self.texture_b.y + gamma * self.texture_c.y,
        )
    }

    pub fn intersects(&self, ray: &Ray<f32, 3>) -> Option<Intersection<f32>> {
        const EPSILON: f32 = 1e-5;

        let mut intersection = ray.intersects_triangle(
            &OPoint::from(self.position_a),
            &OPoint::from(self.position_b),
            &OPoint::from(self.position_c),
        );

        if intersection.distance != f32::INFINITY
            && intersection.distance >= -EPSILON
            && intersection.u >= -EPSILON
            && intersection.u <= 1.0 + EPSILON
            && intersection.v >= -EPSILON
            && intersection.v <= 1.0 + EPSILON
            && (intersection.u + intersection.v) <= 1.0 + EPSILON
        {
            if intersection.distance < 0.0 {
                intersection.distance = 0.0;
            }
            intersection.u = intersection.u.clamp(0.0, 1.0);
            intersection.v = intersection.v.clamp(0.0, 1.0);
            return Some(intersection);
        }

        intersection = ray.intersects_triangle(
            &OPoint::from(self.position_a),
            &OPoint::from(self.position_c),
            &OPoint::from(self.position_b),
        );

        if intersection.distance != f32::INFINITY
            && intersection.distance >= -EPSILON
            && intersection.u >= -EPSILON
            && intersection.u <= 1.0 + EPSILON
            && intersection.v >= -EPSILON
            && intersection.v <= 1.0 + EPSILON
            && (intersection.u + intersection.v) <= 1.0 + EPSILON
        {
            if intersection.distance < 0.0 {
                intersection.distance = 0.0;
            }
            intersection.u = intersection.u.clamp(0.0, 1.0);
            intersection.v = intersection.v.clamp(0.0, 1.0);
            std::mem::swap(&mut intersection.u, &mut intersection.v);
            return Some(intersection);
        }

        None
    }
}

const BBOX_MARGIN: f32 = 0.001;

impl Bounded<f32, 3> for Triangle {
    fn aabb(&self) -> Aabb<f32, 3> {
        let mut min = self.position_a.inf(&self.position_b).inf(&self.position_c);
        let mut max = self.position_a.sup(&self.position_b).sup(&self.position_c);

        min -= Vector3::new(BBOX_MARGIN, BBOX_MARGIN, BBOX_MARGIN);
        max += Vector3::new(BBOX_MARGIN, BBOX_MARGIN, BBOX_MARGIN);

        Aabb {
            min: OPoint::from(min),
            max: OPoint::from(max),
        }
    }
}

impl BHShape<f32, 3> for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.index = index;
    }
    fn bh_node_index(&self) -> usize {
        self.index
    }
}
