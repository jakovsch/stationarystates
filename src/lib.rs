#[macro_use]
mod prelude; use prelude::*;
mod wavefunc; use wavefunc::Psi;
mod icosphere; use icosphere::IcoSphere;
mod deferred; use deferred::RenderPass;

const num_inst: usize = 100000;
static mut STATE: Option<RenderState> = None;

struct RenderState {
    _frame: AnimationFrame,
    _listeners: Vec<EventListener>,
    geometry_pass: GeometryPass,
    ssao_pass: SSAOPass,
    blend_pass: BlendPass,
    context: Gl,
    canvas: web_sys::HtmlCanvasElement,
    proj: Matrix4::<f32>,
    view: Matrix4::<f32>,
    orbit: Orbit::<f32>,
    orbiting: bool,
    time: f64,
}

struct GeometryPass {
    rp: RenderPass,
    n_vert: i32,
    n_inst: i32,
    t_color: web_sys::WebGlTexture,
    t_gdata: web_sys::WebGlTexture,
}
struct SSAOPass {
    rp: RenderPass,
    t_occlusion: web_sys::WebGlTexture,
}
struct BlendPass {
    rp: RenderPass,
}

impl GeometryPass {
    pub fn new(
        context: Gl,
        geometry: &[f32],
        normals: &[f32],
        instances: &[f32],
        n_vert: i32,
        n_inst: i32,
    ) -> Self {
        let rp = RenderPass::new(
            context,
            include_shader!("vert-g.glsl"),
            include_shader!("frag-g.glsl"),
            &["u_proj", "u_view", "u_scale", "u_lightdir"],
            &["i_pos", "a_pos", "a_normal"],
            &["o_color", "o_gdata"],
        );
        rp.attrib_buffer(
            "a_pos", geometry,
            3, 0, 0, false, 0,
        );
        rp.attrib_buffer(
            "a_normal", normals,
            3, 0, 0, true, 0,
        );
        rp.attrib_buffer(
            "i_pos", instances,
            3, 0, 0, false, 1,
        );
        rp.fb_renderbuffer(
            0,
            Gl::DEPTH_COMPONENT16,
            Gl::DEPTH_ATTACHMENT,
        );
        let t_color = rp.fb_texture(
            Gl::TEXTURE0,
            Gl::RGBA8,
            Gl::COLOR_ATTACHMENT0,
        );
        let t_gdata = rp.fb_texture(
            Gl::TEXTURE1,
            Gl::RGBA16F,
            Gl::COLOR_ATTACHMENT1,
        );
        rp.set_draw_buffers(
            &[Gl::COLOR_ATTACHMENT0, Gl::COLOR_ATTACHMENT1],
        );

        Self {rp, n_vert, n_inst, t_color, t_gdata}
    }

    pub fn render(
        &self,
        scale: f32,
        lightdir: &Vector3<f32>,
        proj: &Matrix4<f32>,
        view: &Matrix4<f32>,
    ) {
        let context = &self.rp.context;
        let rp = &self.rp;

        rp.active();
        rp.uniform_float("u_scale", scale);
        rp.uniform_vec3("u_lightdir", lightdir);
        rp.uniform_mat4("u_proj", proj);
        rp.uniform_mat4("u_view", view);

        context.enable(Gl::DEPTH_TEST);
        context.clear_bufferfv_with_f32_array(
            Gl::COLOR, 0, &[1.0, 1.0, 1.0, 1.0],
        );
        context.clear_bufferfv_with_f32_array(
            Gl::COLOR, 1, &[0.0, 0.0, 0.0, 1.0],
        );
        context.clear_bufferfi(
            Gl::DEPTH_STENCIL, 0, 1.0, 0,
        );
        context.draw_arrays_instanced(
            Gl::TRIANGLES, 0,
            self.n_vert, self.n_inst,
        );
    }
}

impl SSAOPass {
    pub fn new(
        context: Gl,
    ) -> Self {
        let rp = RenderPass::new(
            context,
            include_shader!("vert-quad.glsl"),
            include_shader!("frag-ssao.glsl"),
            &["s_gdata", "u_width", "u_height"],
            &["a_pos"],
            &["o_occlusion"],
        );
        rp.attrib_buffer(
            "a_pos",
            &[
                -1.0, 1.0, -1.0, -1.0,
                1.0, -1.0, -1.0, 1.0,
                1.0, -1.0, 1.0, 1.0,
            ],
            2, 0, 0, false, 0,
        );
        let t_occlusion = rp.fb_texture(
            Gl::TEXTURE3,
            Gl::R16F,
            Gl::COLOR_ATTACHMENT0,
        );

        Self {rp, t_occlusion}
    }

    pub fn render(
        &self,
        width: i32,
        height: i32,
        t_gdata: &web_sys::WebGlTexture,
    ) {
        let context = &self.rp.context;
        let rp = &self.rp;

        rp.active();
        rp.uniform_texture("s_gdata", t_gdata, Gl::TEXTURE1);
        rp.uniform_float("u_width", width as f32);
        rp.uniform_float("u_height", height as f32);

        context.disable(Gl::DEPTH_TEST);
        context.draw_arrays(Gl::TRIANGLES, 0, 6);
    }
}

impl BlendPass {
    pub fn new(
        context: Gl,
    ) -> Self {
        let rp = RenderPass::new(
            context,
            include_shader!("vert-quad.glsl"),
            include_shader!("frag-blend.glsl"),
            &["s_color", "s_occlusion"],
            &["a_pos"],
            &["o_color"],
        );
        rp.attrib_buffer(
            "a_pos",
            &[
                -1.0, 1.0, -1.0, -1.0,
                1.0, -1.0, -1.0, 1.0,
                1.0, -1.0, 1.0, 1.0,
            ],
            2, 0, 0, false, 0,
        );
        rp.fb_renderbuffer(
            4,
            Gl::RGBA8,
            Gl::COLOR_ATTACHMENT0,
        );

        Self {rp}
    }

    pub fn render(
        &self,
        width: i32,
        height: i32,
        t_color: &web_sys::WebGlTexture,
        t_occlusion: &web_sys::WebGlTexture,
    ) {
        let context = &self.rp.context;
        let rp = &self.rp;

        rp.active();
        rp.uniform_texture("s_color", t_color, Gl::TEXTURE0);
        rp.uniform_texture("s_occlusion", t_occlusion, Gl::TEXTURE3);

        context.disable(Gl::DEPTH_TEST);
        context.draw_arrays(Gl::TRIANGLES, 0, 6);
        //context.bind_framebuffer(Gl::READ_FRAMEBUFFER, Some(&s.fbo));
        context.bind_framebuffer(Gl::DRAW_FRAMEBUFFER, None);
        context.blit_framebuffer(
            0, 0, width, height,
            0, 0, width, height,
            Gl::COLOR_BUFFER_BIT,
            Gl::NEAREST,
        );
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(debug_assertions)]
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let mem = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .unwrap_throw();
    mem.grow(128);

    let mut attrs = web_sys::WebGlContextAttributes::new();
    attrs.alpha(true);
    attrs.depth(false);
    attrs.stencil(false);
    attrs.antialias(false);
    attrs.power_preference(
        web_sys::WebGlPowerPreference::HighPerformance);

    let document = web_sys::window()
        .unwrap_throw()
        .document()
        .unwrap_throw();
    let canvas = document
        .get_element_by_id("gl_canvas")
        .expect_throw("err: canvas not found")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap_throw();
    let context = canvas
        .get_context_with_context_options("webgl2", attrs.as_ref())
        .expect_throw("err: WebGL2 context creation failed")
        .unwrap_throw()
        .dyn_into::<Gl>()
        .unwrap_throw();

    context.get_extension("EXT_color_buffer_float")
        .expect_throw("err: float framebuffers not supported")
        .unwrap_throw();
    context.disable(Gl::DITHER);
    context.disable(Gl::BLEND);
    context.disable(Gl::SCISSOR_TEST);
    context.disable(Gl::STENCIL_TEST);
    context.enable(Gl::DEPTH_TEST);
    context.enable(Gl::CULL_FACE);

    let camera = Point3::<f32>::new(30.0, 0.0, 0.0);
    let target = Point3::<f32>::new(0.0, 0.0, 0.0);
    let aspect = canvas.client_width() as f32 / canvas.client_height() as f32;
    let orbit = Orbit::<f32>::default();
    let proj = Matrix4::<f32>::new_perspective(aspect, 0.78, 1.0, 50.0);
    let view = Matrix4::<f32>::look_at_rh(&camera, &target, &Vector3::y());
    let particle = IcoSphere::new(0);
    let normals = particle.normals();
    let (n_vert, geometry) = particle.buffer();

    let mut ins_buf = Vec::<f32>::with_capacity(num_inst*VEC3_SZ);
    let mut instances = Vec::<SVectorSliceMut3>::with_capacity(num_inst);
    unsafe {
        ins_buf.set_len(ins_buf.capacity());
        instances.set_len(instances.capacity());
        ins_buf.fill(0.0);
        for i in 0..num_inst {
            let off = i*VEC3_SZ;
            let ptr = ins_buf.as_mut_ptr();
            let slice = slice::from_raw_parts_mut(ptr.add(off), VEC3_SZ);
            instances[i] = SVectorSliceMut3::from_slice(slice);
        }
    }

    let mut rng = SmallRng::seed_from_u64(123);
    let dist = Uniform::<f32>::from(-10.0..10.0).map(
        |x| (x*0.32).sinh()
    );
    let mut r_iter = dist.sample_iter(&mut rng);

    let wavefunc = Psi::new(4, 1, 0);
    let mut num_points = 0;
    const LN: usize = 1000;
    while num_points < num_inst {
        let x = SVector::<f32, LN>::from_iterator(r_iter.by_ref());
        let y = SVector::<f32, LN>::from_iterator(r_iter.by_ref());
        let z = SVector::<f32, LN>::from_iterator(r_iter.by_ref());
        let psi = wavefunc.eval(&x, &y, &z);
        let max_l1 = psi.camax().powi(2);
        for j in 0..LN { unsafe {
            if num_points == num_inst { break }
            let val = (*psi.vget_unchecked(j)).re.powi(2);
            if val/max_l1 >= 0.02 {
                let inst = instances
                    .get_unchecked_mut(num_points);
                inst.x += *x.vget_unchecked(j);
                inst.y += *y.vget_unchecked(j);
                inst.z += *z.vget_unchecked(j);
                num_points += 1;
            }
        }}
    }

    let geometry_pass = GeometryPass::new(
        context.clone(),
        geometry.as_slice(),
        normals.as_slice(),
        ins_buf.as_slice(),
        n_vert, num_inst as i32,
    );
    let ssao_pass = SSAOPass::new(
        context.clone(),
    );
    let blend_pass = BlendPass::new(
        context.clone(),
    );

    let resize = EventListener::new(&web_sys::window().unwrap_throw(), "resize",
        |_e: &web_sys::Event| {
            let s = unsafe { STATE.as_mut().unwrap_throw() };
            let window = web_sys::window().unwrap_throw();
            let width = window
                .inner_width().unwrap_throw()
                .as_f64().unwrap_throw();
            let height = window
                .inner_height().unwrap_throw()
                .as_f64().unwrap_throw();
            let aspect = (width/height) as f32;
            s.canvas.set_width(width as u32);
            s.canvas.set_height(height as u32);
            s.context.viewport(0, 0, width as i32, height as i32);
            s.proj[(0, 0)] = s.proj[(1, 1)] / aspect;
        },
    );

    let mouseup = EventListener::new(&document, "mouseup",
        |e: &web_sys::Event| {
            if e.dyn_ref::<web_sys::MouseEvent>().is_some() {
                let s = unsafe { STATE.as_mut().unwrap_throw() };
                s.orbiting = false;
            }
        },
    );

    let mousedown = EventListener::new(&document, "mousedown",
        |e: &web_sys::Event| {
            if e.dyn_ref::<web_sys::MouseEvent>().is_some() {
                let s = unsafe { STATE.as_mut().unwrap_throw() };
                s.orbit.discard();
                s.orbiting = true;
            }
        },
    );

    let mousemove = EventListener::new(&document, "mousemove",
        |e: &web_sys::Event| {
            if let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() {
                let s = unsafe { STATE.as_mut().unwrap_throw() };
                let mx = s.canvas.client_width() as f32;
                let my = s.canvas.client_height() as f32;
                let max = Point2::new(mx, my);
                let pos = Point2::new(
                    mx-e.client_x() as f32,
                    my-e.client_y() as f32,
                );
                if s.orbiting {
                    let q = s.orbit.compute(&pos, &max).unwrap_or_default();
                    s.view *= q.to_homogeneous();
                }
            }
        },
    );

    unsafe {
        STATE = Some(RenderState {
            _frame: request_animation_frame(render),
            _listeners: vec![
                mouseup,
                mousedown,
                mousemove,
                resize,
            ],
            geometry_pass,
            ssao_pass,
            blend_pass,
            context, canvas,
            proj, view, orbit,
            orbiting: false,
            time: 0.0,
        });
    };
}

fn render(mut time: f64) {
    let s = unsafe { STATE.as_mut().unwrap_throw() };
    let context = &s.context;

    time *= 0.001;
    let dt = time-s.time;
    s.time = time;

    let width = context.drawing_buffer_width();
    let height = context.drawing_buffer_height();
    let lightdir = Vector3::<f32>::new(0.0, 1.0, 1.0);

    s.geometry_pass.render(
        0.15, &lightdir, &s.proj, &s.view,
    );
    s.ssao_pass.render(
        width, height,
        &s.geometry_pass.t_gdata,
    );
    s.blend_pass.render(
        width, height,
        &s.geometry_pass.t_color,
        &s.ssao_pass.t_occlusion,
    );

    s._frame = request_animation_frame(render);
}
