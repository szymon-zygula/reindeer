macro_rules! impl_dot_product_next {
    ($self:ident, $rhs:ident, $coord:ident) => {
        $self.$coord * $rhs.$coord
    };

    ($self:ident, $rhs:ident, $coord:ident, $($tail:ident),+) => {
        $self.$coord * $rhs.$coord + impl_dot_product_next!($self, $rhs, $($tail),+)
    }
}

macro_rules! declare_vector {
    ($type:ident, $($coord:ident),+) => {
        #[derive(Clone, Copy)]
        pub struct $type {
            $(pub $coord: f32),+
        }

        impl $type {
            pub const ZERO: Self = Self { $($coord: 0.0),+ };

            pub fn len(&self) -> f32 {
                (0.0 $(+ self.$coord * self.$coord)+).sqrt()
            }

            pub fn normalized(&self) -> $type {
                let len = self.len();
                $type {
                    $($coord: self.$coord / len),+
                }
            }
        }

        impl std::ops::Mul<f32> for $type {
            type Output = $type;

            fn mul(self, rhs: f32) -> Self::Output {
                $type {
                    $($coord: self.$coord * rhs),+
                }
            }
        }

        impl std::ops::Mul<$type> for f32 {
            type Output = $type;

            fn mul(self, rhs: $type) -> Self::Output {
                $type {
                    $($coord: rhs.$coord * self),+
                }
            }
        }

        impl std::ops::Mul<$type> for $type {
            type Output = f32;

            fn mul(self, rhs: $type) -> Self::Output {
                impl_dot_product_next!(self, rhs, $($coord),+)
            }
        }

        impl std::ops::Add<$type> for $type {
            type Output = $type;

            fn add(self, rhs: $type) -> Self::Output {
                $type {
                    $($coord: self.$coord + rhs.$coord),+
                }
            }
        }

        impl std::ops::Sub<$type> for $type {
            type Output = $type;

            fn sub(self, rhs: $type) -> Self::Output {
                $type {
                    $($coord: self.$coord - rhs.$coord),+
                }
            }
        }
    }
}

declare_vector!(Vec2, x, y);
declare_vector!(Vec3, x, y, z);

impl Vec3 {
    pub fn homo_point(&self) -> Vec4 {
        Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: 1.0
        }
    }

    pub fn homo_vector(&self) -> Vec4 {
        Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: 0.0
        }
    }
}

pub fn cross(v: &Vec3, u: &Vec3) -> Vec3 {
    Vec3 {
        x: v.y * u.z - v.z * u.y,
        y: v.z * u.x - v.x * u.z,
        z: v.x * u.y - v.y * u.x
    }
}


declare_vector!(Vec4, x, y, z, w);

impl Vec4 {
    pub fn point_proj(&self) -> Vec3 {
        Vec3 {
            x: self.x / self.w,
            y: self.y / self.w,
            z: self.z / self.w
        }
    }

    pub fn vector_proj(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z
        }
    }
}

