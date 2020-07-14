use cgmath::{InnerSpace, Matrix4, Transform, Vector3, Vector4};

pub const EPSYLON: f32 = 0.000001;
pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub struct Plane {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub left: Vector3<f32>,
    pub down: Vector3<f32>,
    #[cfg(feature = "debug_ray")]
    pub name: &'static str,
}

pub const XY_PLANE: Plane = Plane {
    point: Vector3 {x: 0.5, y: 0.5, z: 0.0},
    normal: Vector3 { x: 0.0, y: 0.0, z: 1.0 },
    left: Vector3 { x: 1.0, y: 0.0, z: 0.0 },
    down: Vector3 { x: 0.0, y: -1.0, z: 0.0 },
    #[cfg(feature = "debug_ray")]
    name: "XY",
};

pub const YZ_PLANE: Plane = Plane {
    point: Vector3 {x: 0.0, y: 0.5, z: 0.5},
    normal: Vector3 { x: 1.0, y: 0.0, z: 0.0 },
    left: Vector3 { x: 0.0, y: 0.0, z: -1.0 },
    down: Vector3 { x: 0.0, y: -1.0, z: 0.0 },
    #[cfg(feature = "debug_ray")]
    name: "YZ",
};

pub const XZ_PLANE: Plane = Plane {
    point: Vector3 {x: 0.5, y: 0.0, z: 0.5},
    normal: Vector3 { x: 0.0, y: 1.0, z: 0.0 },
    left: Vector3 { x: 0.0, y: 0.0, z: -1.0 },
    down: Vector3 { x: 1.0, y: 0.0, z: 0.0 },
    #[cfg(feature = "debug_ray")]
    name: "XZ",
};

pub struct Ray {
    pub point: Vector3<f32>,
    pub vector: Vector3<f32>,
}

impl Ray {
    pub fn new(
        point: Vector3<f32>,
        vector: Vector3<f32>,
    ) -> Self {
        Ray {
            point,
            vector,
        }
    }

    pub fn plane_intersection(
        &self,
        plane: &Plane,
    ) -> Option<Vector3<f32>> {
        let dist_square = self.vector.dot(plane.normal);
        if dist_square.abs() > EPSYLON {
            let diff = self.point - plane.point;
            let dist_square2 = diff.dot(plane.normal) / dist_square;
            if dist_square2 >= EPSYLON {
                return Some(self.point - self.vector * dist_square2)
            }
        }
        None
    }
}

pub fn unproject(
    winx: f32,
    winy: f32,
    winz: f32,
    model_view: Matrix4<f32>,
    projection: Matrix4<f32>,
    window_size: winit::dpi::PhysicalSize<u32>,
) -> Vector3<f32> {
    let matrix = (projection * model_view).inverse_transform().unwrap();

    let in_vec = Vector4::new(
        (winx / window_size.width as f32) * 2.0 - 1.0,
        ((window_size.height as f32 - winy) / window_size.height as f32) * 2.0 - 1.0,
        2.0 * winz - 1.0,
        1.0,
    );

    let mut out = matrix * in_vec;
    out.w = 1.0 / out.w;

    Vector3::new(out.x * out.w, out.y * out.w, out.z * out.w)
}

#[derive(Debug, Copy, Clone)]
pub struct Cuboid {
    pub corner: Vector3<f32>,
    pub extent: Vector3<f32>,
    pub color: [f32; 4],
}

impl Cuboid {
    pub fn corner_points(&self) -> [Vector3<f32>; 8] {
        [
            self.corner,
            self.corner + Vector3::new(self.extent.x, 0.0, 0.0),
            self.corner + Vector3::new(self.extent.x, self.extent.y, 0.0),
            self.corner + Vector3::new(0.0, self.extent.y, 0.0),

            self.corner + Vector3::new(0.0, 0.0, self.extent.z),
            self.corner + Vector3::new(self.extent.x, 0.0, self.extent.z),
            self.corner + Vector3::new(self.extent.x, self.extent.y, self.extent.z),
            self.corner + Vector3::new(0.0, self.extent.y, self.extent.z),
        ]
    }

    fn manhattan_distance(
        start: &Vector3<f32>,
        end: &Vector3<f32>,
    ) -> f32 {
        (start.x - end.x).abs() + (start.y - end.y).abs() + (start.z - end.z).abs()
    }

    fn outermost_points(
        &self,
        other: &Self,
    ) -> (Vector3<f32>, Vector3<f32>) {
        let mut distance = 0.0;
        let mut outermost_points = (Vector3::unit_x(), Vector3::unit_x());
        for x in self.corner_points().iter() {
            for y in other.corner_points().iter() {
                let manhattan_dist = Self::manhattan_distance(x, y);
                if manhattan_dist > distance {
                    outermost_points = (*x, *y);
                    distance = manhattan_dist;
                }
            }
        }
        outermost_points
    }

    fn from_corner_points(
        point1: Vector3<f32>,
        point2: Vector3<f32>,
        color: [f32; 4],
    ) -> Self {
        Cuboid {
            corner: point1,
            extent: point2 - point1,
            color,
        }
    }

    pub fn containing_cube(
        &self,
        other: &Self,
    ) -> Self {
        let (point1, point2) = self.outermost_points(other);
        Self::from_corner_points(point1, point2, self.color)
    }

    pub fn new(
        corner: Vector3<f32>,
        extent: Vector3<f32>,
        color: [f32; 4],
    ) -> Self {
        Cuboid {
            corner,
            extent,
            color,
        }
    }

    pub fn rearrange(&mut self) {
        let corner_points = self.corner_points();
        let mut closest_to_origo = corner_points[0];
        let mut farthest_from_origo = corner_points[0];
        for point in corner_points[1..].iter() {
            if point.x <= closest_to_origo.x &&
                point.y <= closest_to_origo.y &&
                point.z <= closest_to_origo.z {
                    closest_to_origo = *point;
                }

            if point.x >= farthest_from_origo.x &&
                point.y >= farthest_from_origo.y &&
                point.z >= farthest_from_origo.z {
                    farthest_from_origo = *point;
                }
        }

        if self.corner == closest_to_origo {
            return;
        }
        *self = Self::from_corner_points(closest_to_origo, farthest_from_origo, self.color);
    }
}