pub use wasm_bindgen::{prelude::*, JsCast};
pub use web_sys::{self, WebGl2RenderingContext};
pub use gloo_events::EventListener;
pub use gloo_render::{AnimationFrame, request_animation_frame};
pub use nalgebra::{
    SVector, Vector3, VectorSliceMut3,
    Point3, Point2, Matrix4, ComplexField,
};
pub use trackball::Orbit;
pub use rand::{Rng, SeedableRng};
pub use rand::distributions::{Distribution, Uniform};
pub use rand::rngs::SmallRng;
pub use std::{mem, slice, ops::AddAssign};

pub type Gl = WebGl2RenderingContext;
pub type SVectorSliceMut3 = VectorSliceMut3<'static, f32>;
pub const VEC3_SZ: usize = 3*4;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! include_shader {
    ($path:literal) => {
        include_str!(concat!("./shaders/", $path))
    };
}
