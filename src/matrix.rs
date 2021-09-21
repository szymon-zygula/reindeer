use crate::vector::{
    Vec2,
    Vec3,
    Vec4
};

macro_rules! declare_square_matrix {
    ($name:ident, $n:expr) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            buf: [f32; Self::N * Self::N]
        }

        impl $name {
            pub const N: usize = $n;

            pub fn new(rcs: [[f32; Self::N]; Self::N]) -> Self {
                let mut buf: [f32; Self::N * Self::N] = unsafe {
                    std::mem::MaybeUninit::uninit().assume_init()
                };
                for i in 0..Self::N {
                    for j in 0..Self::N {
                        buf[i * Self::N + j] = rcs[i][j];
                    }
                }

                $name { buf }
            }
        }

        impl std::ops::Index<(usize, usize)> for $name {
            type Output = f32;

            fn index(&self, rc: (usize, usize)) -> &Self::Output {
                &self.buf[rc.0 * Self::N + rc.1]
            }
        }

        impl std::ops::IndexMut<(usize, usize)> for $name {
            fn index_mut(&mut self, rc: (usize, usize)) -> &mut Self::Output {
                &mut self.buf[rc.0 * Self::N + rc.1]
            }
        }

        impl std::ops::Mul<$name> for f32 {
            type Output = $name;

            fn mul(self, rhs: $name) -> Self::Output {
                let mut m: $name = rhs;
                for el in m.buf.iter_mut() {
                    *el *= self;
                }

                m
            }
        }

        impl std::ops::Mul<f32> for $name {
            type Output = $name;

            fn mul(self, rhs: f32) -> Self::Output {
                let mut m: $name = self;
                for el in m.buf.iter_mut() {
                    *el *= rhs;
                }

                m
            }
        }

        impl std::ops::Mul<$name> for $name {
            type Output = $name;

            fn mul(self, rhs: $name) -> Self::Output {
                let mut m = $name::new([[0.0; Self::N]; Self::N]);

                for i in 0..Self::N {
                    for j in 0..Self::N {
                        for k in 0..Self::N {
                            m[(i, j)] += self[(i, k)] * rhs[(k, j)];
                        }
                    }
                }

                m
            }
        }
    }
}

declare_square_matrix!(Matrix2, 2);

impl Matrix2 {
    pub const IDENTITY: Self = Self {
        buf: [
            1.0, 0.0,
            0.0, 1.0
        ]
    };

    pub fn determinant(&self) -> f32 {
        self[(0, 0)] * self[(1, 1)] - self[(0, 1)] * self[(1, 0)]
    }

    pub fn inverse(&self) -> Option<Self> {
        let determinant = self.determinant();

        if determinant == 0.0 {
            return None;
        }

        Some((1.0 / determinant) * Self::new([
            [ self[(1, 1)], -self[(0, 1)]],
            [-self[(1, 0)],  self[(0, 0)]]
        ]))
    }

    pub fn trans(&self) -> Self {
        Self::new([
            [self[(0, 0)], self[(1, 0)]],
            [self[(0, 1)], self[(1, 1)]]
        ])
    }
}

impl std::ops::Mul<Vec2> for Matrix2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        let mut v: [f32; 2] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        for i in 0..2 {
            v[i] = rhs.x * self[(i, 0)] + rhs.y * self[(i, 1)];
        }

        Vec2 {
            x: v[0],
            y: v[1]
        }
    }
}

declare_square_matrix!(Matrix3, 3);

impl Matrix3 {
    pub const IDENTITY: Self = Self {
        buf: [
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
        ]
    };

    // Rule of Sarrus
    pub fn determinant(&self) -> f32 {
        self[(0, 0)] * (self[(1, 1)] * self[(2, 2)] - self[(1, 2)] * self[(2, 1)]) +
        self[(0, 1)] * (self[(1, 2)] * self[(2, 0)] - self[(1, 0)] * self[(2, 2)]) +
        self[(0, 2)] * (self[(1, 0)] * self[(2, 1)] - self[(1, 1)] * self[(2, 0)])
    }

    pub fn inverse(&self) -> Option<Self> {
        let determinant = self.determinant();

        if determinant == 0.0 {
            return None;
        }

        Some((1.0 / determinant) * Self::new([
            [
                self[(1, 1)] * self[(2, 2)] - self[(1, 2)] * self[(2, 1)],
                self[(0, 2)] * self[(2, 1)] - self[(0, 1)] * self[(2, 2)],
                self[(0, 1)] * self[(1, 2)] - self[(0, 2)] * self[(1, 1)]
            ],
            [
                self[(1, 2)] * self[(2, 0)] - self[(1, 0)] * self[(2, 2)],
                self[(0, 0)] * self[(2, 2)] - self[(0, 2)] * self[(2, 0)],
                self[(0, 2)] * self[(1, 0)] - self[(0, 0)] * self[(1, 2)]
            ],
            [
                self[(1, 0)] * self[(2, 1)] - self[(1, 1)] * self[(2, 0)],
                self[(0, 1)] * self[(2, 0)] - self[(0, 0)] * self[(2, 1)],
                self[(0, 0)] * self[(1, 1)] - self[(0, 1)] * self[(1, 0)]
            ]
        ]))
    }
}

impl std::ops::Mul<Vec3> for Matrix3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let mut v: [f32; 3] = unsafe {
            std::mem::MaybeUninit::uninit().assume_init()
        };

        for i in 0..3 {
            v[i] =
                rhs.x * self[(i, 0)] +
                rhs.y * self[(i, 1)] +
                rhs.z * self[(i, 2)];
        }

        Vec3 {
            x: v[0],
            y: v[1],
            z: v[2]
        }
    }
}

declare_square_matrix!(Matrix4, 4);

impl Matrix4 {
    pub const IDENTITY: Self = Self {
        buf: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ]
    };
}

impl std::ops::Mul<Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        let mut v: [f32; 4] = unsafe {
            std::mem::MaybeUninit::uninit().assume_init()
        };

        for i in 0..4 {
            v[i] =
                rhs.x * self[(i, 0)] +
                rhs.y * self[(i, 1)] +
                rhs.z * self[(i, 2)] +
                rhs.w * self[(i, 3)];
        }

        Vec4 {
            x: v[0],
            y: v[1],
            z: v[2],
            w: v[3]
        }
    }
}
