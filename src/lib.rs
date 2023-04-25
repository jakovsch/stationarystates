#[macro_use]
mod prelude; use prelude::*;
mod wavefunc; use wavefunc::Psi;
mod icosphere; use icosphere::IcoSphere;
mod deferred; use deferred::RenderPass;

static mut STATE: Option<RenderState> = None;

struct RenderState {
    _frame: AnimationFrame,
    _listeners: Vec<EventListener>,
    xfb_pass: XFBPass,
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

struct XFBPass {
    rp: RenderPass,
    n_inst: usize,
    read_idx: usize,
    write_idx: usize,
    buffers: Vec<web_sys::WebGlBuffer>,
}
struct GeometryPass {
    rp: RenderPass,
    n_inst: usize,
    particle: IcoSphere,
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

impl XFBPass {
    pub fn new(
        context: Gl,
        instances: &[f32],
        n_inst: usize,
    ) -> Self {
        let rp = RenderPass::new(
            context, 0, 2,
            include_shader!("vert-xfb.glsl"),
            include_shader!("no-op.glsl"),
            Some(&["u_dt"]), Some(&["i_pos"]),
            None, Some(&["v_pos"]),
        );
        let buf1 = rp.buffer_data(instances, Gl::STREAM_DRAW);
        let buf2 = rp.buffer_alloc((n_inst*VEC3_SZ) as i32, Gl::STREAM_DRAW);
        rp.vao_buffer(0, &buf1, "i_pos", 3, 0, 0, false, 0);
        rp.vao_buffer(1, &buf2, "i_pos", 3, 0, 0, false, 0);
        let buffers = vec![buf1, buf2];

        Self {rp, n_inst, read_idx: 0, write_idx: 1, buffers}
    }

    pub fn render(
        &mut self,
        dt: f32,
    ) {
        let context = &self.rp.context;
        let rp = &self.rp;

        rp.active(0, self.read_idx);
        rp.uniform_float("u_dt", dt);

        context.bind_buffer_base(
            Gl::TRANSFORM_FEEDBACK_BUFFER,
            0, self.buffers.get(self.write_idx),
        );
        context.enable(Gl::RASTERIZER_DISCARD);
        context.begin_transform_feedback(Gl::POINTS);
        context.draw_arrays(Gl::POINTS, 0, self.n_inst as i32);
        context.end_transform_feedback();
        context.disable(Gl::RASTERIZER_DISCARD);
        context.bind_buffer_base(
            Gl::TRANSFORM_FEEDBACK_BUFFER,
            0, None,
        );

        mem::swap(&mut self.read_idx, &mut self.write_idx);
    }
}

impl GeometryPass {
    pub fn new(
        context: Gl,
        buf_i: &Vec<web_sys::WebGlBuffer>,
        n_inst: usize,
        particle_lod: usize,
    ) -> Self {
        let rp = RenderPass::new(
            context, 1, 2,
            include_shader!("vert-g.glsl"),
            include_shader!("frag-g.glsl"),
            Some(&["u_proj", "u_view", "u_scale", "u_lightdir"]),
            Some(&["i_pos", "a_pos", "a_normal"]),
            Some(&["o_color", "o_gdata"]),
            None,
        );
        let particle = IcoSphere::new(particle_lod);
        let buf_g = rp.buffer_data(particle.vertex_buf().as_slice(), Gl::STATIC_DRAW);
        let buf_n = rp.buffer_data(particle.normal_buf().as_slice(), Gl::STATIC_DRAW);
        for i in 0..2 {
            rp.vao_buffer(i, &buf_i[i], "i_pos", 3, 0, 0, false, 1);
            rp.vao_buffer(i, &buf_g, "a_pos", 3, 0, 0, false, 0);
            rp.vao_buffer(i, &buf_n, "a_normal", 3, 0, 0, true, 0);
        }
        rp.fb_renderbuffer(
            0, 0,
            Gl::DEPTH_COMPONENT16,
            Gl::DEPTH_ATTACHMENT,
        );
        let t_color = rp.fb_texture(
            0,
            Gl::TEXTURE0,
            Gl::RGBA8,
            Gl::COLOR_ATTACHMENT0,
        );
        let t_gdata = rp.fb_texture(
            0,
            Gl::TEXTURE1,
            Gl::RGBA16F,
            Gl::COLOR_ATTACHMENT1,
        );
        rp.set_draw_buffers(
            0, &[Gl::COLOR_ATTACHMENT0, Gl::COLOR_ATTACHMENT1],
        );

        Self {rp, n_inst, particle, t_color, t_gdata}
    }

    pub fn render(
        &self,
        read_idx: usize,
        scale: f32,
        lightdir: &Vector3<f32>,
        proj: &Matrix4<f32>,
        view: &Matrix4<f32>,
    ) {
        let context = &self.rp.context;
        let rp = &self.rp;

        rp.active(0, read_idx);
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
            self.particle.n_vert as i32,
            self.n_inst as i32,
        );
    }
}

impl SSAOPass {
    pub fn new(
        context: Gl,
    ) -> Self {
        let rp = RenderPass::new(
            context, 1, 1,
            include_shader!("vert-quad.glsl"),
            include_shader!("frag-ssao.glsl"),
            Some(&["s_gdata", "u_width", "u_height"]),
            Some(&["a_pos"]),
            Some(&["o_occlusion"]),
            None,
        );
        let buf_g = rp.buffer_data(
            &[
                -1.0, 1.0, -1.0, -1.0,
                1.0, -1.0, -1.0, 1.0,
                1.0, -1.0, 1.0, 1.0,
            ],
            Gl::STATIC_DRAW,
        );
        rp.vao_buffer(0, &buf_g, "a_pos", 2, 0, 0, false, 0);
        let t_occlusion = rp.fb_texture(
            0,
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

        rp.active(0, 0);
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
            context, 1, 1,
            include_shader!("vert-quad.glsl"),
            include_shader!("frag-blend.glsl"),
            Some(&["s_color", "s_occlusion"]),
            Some(&["a_pos"]),
            Some(&["o_color"]),
            None,
        );
        let buf_g = rp.buffer_data(
            &[
                -1.0, 1.0, -1.0, -1.0,
                1.0, -1.0, -1.0, 1.0,
                1.0, -1.0, 1.0, 1.0,
            ],
            Gl::STATIC_DRAW,
        );
        rp.vao_buffer(0, &buf_g, "a_pos", 2, 0, 0, false, 0);
        rp.fb_renderbuffer(
            0, 4,
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

        rp.active(0, 0);
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
    attrs.power_preference(web_sys::WebGlPowerPreference::HighPerformance);
    attrs.alpha(true);
    attrs.depth(false);
    attrs.stencil(false);
    attrs.antialias(false);

    let window = web_sys::window()
        .unwrap_throw();
    let document = window
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
    let proj = Matrix4::<f32>::new_perspective(aspect, 0.78, 1.0, 50.0);
    let view = Matrix4::<f32>::look_at_rh(&camera, &target, &Vector3::y());
    let orbit = Orbit::<f32>::default();
    let particle_lod = 0;
    let num_inst = 100000;

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

    let mut rng1 = SmallRng::seed_from_u64(123456789);
    //let mut rng2 = SmallRng::seed_from_u64(123456789);
    let dist1 = Uniform::<f32>::from(-10.0..10.0).map(
        |x| (x*0.32).sinh()
    );
    //let dist2 = Uniform::<f32>::from(0.0..1.0);
    let mut r_iter = dist1.sample_iter(&mut rng1);
    //let mut s_iter = dist2.sample_iter(&mut rng2);

    let wavefunc = Psi::new(4, 1, 0);
    let mut samples = 0;
    const batch_size: usize = 1000;
    while samples < num_inst {
        let x = SVector::<f32, batch_size>::from_iterator(r_iter.by_ref());
        let y = SVector::<f32, batch_size>::from_iterator(r_iter.by_ref());
        let z = SVector::<f32, batch_size>::from_iterator(r_iter.by_ref());
        //let r = SVector::<f32, batch_size>::from_iterator(s_iter.by_ref());
        let psi = wavefunc.eval(&x, &y, &z).map(|x| x.norm_sqr());
        //let max = psi.camax().powi(2);
        let max = psi.max();
        for j in 0..batch_size { unsafe {
            if samples == num_inst { break }
            //let val = (*psi.vget_unchecked(j)).re.powi(2);
            let val = *psi.vget_unchecked(j);
            //let rnd = *r.vget_unchecked(j);
            if val/max >= 0.02 {
                let inst = instances.get_unchecked_mut(samples);
                inst.x += *x.vget_unchecked(j);
                inst.y += *y.vget_unchecked(j);
                inst.z += *z.vget_unchecked(j);
                samples += 1;
            }
        }}
    }

    let xfb_pass = XFBPass::new(
        context.clone(),
        ins_buf.as_slice(),
        num_inst,
    );
    let geometry_pass = GeometryPass::new(
        context.clone(),
        &xfb_pass.buffers,
        num_inst,
        particle_lod,
    );
    let ssao_pass = SSAOPass::new(
        context.clone(),
    );
    let blend_pass = BlendPass::new(
        context.clone(),
    );

    unsafe {
        STATE = Some(RenderState {
            _frame: request_animation_frame(render),
            _listeners: setup_event_handlers(&window, &document),
            xfb_pass,
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

    time *= 0.001;
    let dt = time-s.time;
    s.time = time;

    let width = s.context.drawing_buffer_width();
    let height = s.context.drawing_buffer_height();
    let scale = 2.0;
    let lightdir = Vector3::<f32>::new(0.0, 1.0, 1.0);

    s.xfb_pass.render(
        dt as f32,
    );
    s.geometry_pass.render(
        s.xfb_pass.read_idx,
        scale, &lightdir, &s.proj, &s.view,
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

fn setup_event_handlers(
    window: &web_sys::Window,
    document: &web_sys::Document,
) -> Vec<EventListener> {
    let resize = EventListener::new(window, "resize",
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

    let mouseup = EventListener::new(document, "mouseup",
        |e: &web_sys::Event| {
            if e.dyn_ref::<web_sys::MouseEvent>().is_some() {
                let s = unsafe { STATE.as_mut().unwrap_throw() };
                s.orbiting = false;
            }
        },
    );

    let mousedown = EventListener::new(document, "mousedown",
        |e: &web_sys::Event| {
            if e.dyn_ref::<web_sys::MouseEvent>().is_some() {
                let s = unsafe { STATE.as_mut().unwrap_throw() };
                s.orbit.discard();
                s.orbiting = true;
            }
        },
    );

    let mousemove = EventListener::new(document, "mousemove",
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

    return vec![resize, mouseup, mousedown, mousemove];
}
