#[macro_use]
mod prelude; use prelude::*;
mod wavefunc; use wavefunc::Psi;
mod icosphere; use icosphere::IcoSphere;

const num_inst: usize = 100000;
static mut STATE: Option<RenderState> = None;

struct RenderState {
    _frame: AnimationFrame,
    _e_mouseup: EventListener,
    _e_mousedown: EventListener,
    _e_mousemove: EventListener,
    time: f64,
    n_vert: i32,
    context: Gl,
    canvas: web_sys::HtmlCanvasElement,
    u_proj: web_sys::WebGlUniformLocation,
    u_view: web_sys::WebGlUniformLocation,
    proj: Matrix4::<f32>,
    view: Matrix4::<f32>,
    orbit: Orbit::<f32>,
    orbiting: bool,
    vao: web_sys::WebGlVertexArrayObject,
    mat_buf_dst: web_sys::WebGlBuffer,
    mat_buf_src: &'static[f32],
    matrices: [SMatrixSliceMut4; num_inst],
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let mem = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()?;
    mem.grow(128);

    let mut attrs = web_sys::WebGlContextAttributes::new();
    attrs.depth(true);
    attrs.alpha(false);
    attrs.stencil(false);
    attrs.antialias(true);
    attrs.power_preference(
        web_sys::WebGlPowerPreference::HighPerformance);

    let document = web_sys::window()
        .unwrap()
        .document()
        .unwrap();
    let canvas = document
        .get_element_by_id("gl_canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let context = canvas
        .get_context_with_context_options("webgl2", attrs.as_ref())?
        .unwrap()
        .dyn_into::<Gl>()?;

    let vert_shader = gl_compile_shader(
        &context,
        Gl::VERTEX_SHADER,
        include_shader!("vert.glsl"))?;
    let frag_shader = gl_compile_shader(
        &context,
        Gl::FRAGMENT_SHADER,
        include_shader!("frag.glsl"))?;

    let program = gl_link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let ident = Matrix4::<f32>::identity();
    let camera = Point3::<f32>::new(30.0, 0.0, 0.0);
    let target = Point3::<f32>::new(0.0, 0.0, 0.0);
    let aspect = canvas.client_width() as f32 / canvas.client_height() as f32;
    let orbit = Orbit::<f32>::default();
    let proj = Matrix4::<f32>::new_perspective(aspect, 0.78, 1.0, 50.0);
    let view = Matrix4::<f32>::look_at_rh(&camera, &target, &Vector3::y());
    let particle = IcoSphere::new(0);
    let normals = particle.normals();
    let (n_vert, geometry) = particle.buffer();

    let u_proj = context.get_uniform_location(&program, "u_proj")
        .ok_or("err: uniform_location")?;
    let u_view = context.get_uniform_location(&program, "u_view")
        .ok_or("err: uniform_location")?;

    let i_model = context.get_attrib_location(&program, "i_model");
    let a_pos = context.get_attrib_location(&program, "a_pos");
    let a_normal = context.get_attrib_location(&program, "a_normal");

    let vao = context.create_vertex_array()
        .ok_or("err: create_vertex_array")?;
    context.bind_vertex_array(Some(&vao));

    let vert_buf_dst = context.create_buffer()
        .ok_or("err: create_buffer")?;
    context.bind_buffer(Gl::ARRAY_BUFFER, Some(&vert_buf_dst));
    unsafe {
        let vert_buf_src = js_sys::Float32Array::view(geometry.as_slice());
        context.buffer_data_with_array_buffer_view(
            Gl::ARRAY_BUFFER, &vert_buf_src, Gl::STATIC_DRAW);
    }

    context.enable_vertex_attrib_array(a_pos as u32);
    context.vertex_attrib_pointer_with_i32(
        a_pos as u32, 3, Gl::FLOAT, false, 0, 0);

    let norm_buf_dst = context.create_buffer()
        .ok_or("err: create_buffer")?;
    context.bind_buffer(Gl::ARRAY_BUFFER, Some(&norm_buf_dst));
    unsafe {
        let norm_buf_src = js_sys::Float32Array::view(normals.as_slice());
        context.buffer_data_with_array_buffer_view(
            Gl::ARRAY_BUFFER, &norm_buf_src, Gl::STATIC_DRAW);
    }

    context.enable_vertex_attrib_array(a_normal as u32);
    context.vertex_attrib_pointer_with_i32(
        a_normal as u32, 3, Gl::FLOAT, false, 0, 0);

    static mut mat_buf_src: [f32; num_inst*16*4] = [0.0; num_inst*16*4];
    let mut matrices = {
        let mut arr: [
            mem::MaybeUninit<SMatrixSliceMut4>; num_inst
        ] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        for i in 0..num_inst {
            let off = i*16*4;
            unsafe {
                let ptr = mat_buf_src.as_mut_ptr();
                let slice = slice::from_raw_parts_mut(ptr.add(off), 16*4);
                arr[i].write(SMatrixSliceMut4::from_slice(slice));
            }
        }
        unsafe { mem::transmute::<_, [SMatrixSliceMut4; num_inst]>(arr) }
    };

    let mat_buf_dst = context.create_buffer()
        .ok_or("err: create_buffer")?;
    context.bind_buffer(Gl::ARRAY_BUFFER, Some(&mat_buf_dst));
    context.buffer_data_with_i32(
        Gl::ARRAY_BUFFER,
        unsafe { mat_buf_src.len() as i32*4 },
        Gl::DYNAMIC_DRAW
    );

    for i in 0..4 {
        let off = i*16;
        let loc = (i+i_model) as u32;
        context.enable_vertex_attrib_array(loc);
        context.vertex_attrib_pointer_with_i32(
            loc, 4, Gl::FLOAT, false, 16*4, off);
        context.vertex_attrib_divisor(loc, 1);
    }

    let mouseup = EventListener::new(&document, "mouseup",
        |e: &web_sys::Event| {
            if e.dyn_ref::<web_sys::MouseEvent>().is_some() {
                let s = unsafe { STATE.as_mut().unwrap() };
                s.orbiting = false;
            }
        },
    );
    let mousedown = EventListener::new(&document, "mousedown",
        |e: &web_sys::Event| {
            if e.dyn_ref::<web_sys::MouseEvent>().is_some() {
                let s = unsafe { STATE.as_mut().unwrap() };
                s.orbit.discard();
                s.orbiting = true;
            }
        },
    );
    let mousemove = EventListener::new(&document, "mousemove",
        |e: &web_sys::Event| {
            if let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() {
                let s = unsafe { STATE.as_mut().unwrap() };
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

    let mut rng = SmallRng::seed_from_u64(123);
    let dist = Uniform::<f64>::from(-10.0..10.0);
    let mut r_iter = dist.sample_iter(&mut rng);

    let wavefunc = Psi::new(4, 1, 0);
    const l: usize = if num_inst >= 1000 { 1000 } else { num_inst };
    const d: usize = num_inst/l;
    const m: usize = num_inst%l;
    for i in 0..d {
        let x = SVector::<f64, l>::from_iterator(r_iter.by_ref());
        let y = SVector::<f64, l>::from_iterator(r_iter.by_ref());
        let z = SVector::<f64, l>::from_iterator(r_iter.by_ref());
        let psi = wavefunc.eval(&x, &y, &z);
        let max_l1 = psi.camax().powi(2);
        for j in 0..l {
            let pos = Vector3::<f32>::new(
                *x.index(j) as f32, *y.index(j) as f32, *z.index(j) as f32
            );
            let val = (*psi.index(j)).re.powi(2);
            if val/max_l1 >= 0.02 {
                matrices[j+i*l].add_assign(&ident);
                matrices[j+i*l].append_scaling_mut(0.15);
                matrices[j+i*l].append_translation_mut(&pos);
            }
        }
    }
    context.bind_buffer(Gl::ARRAY_BUFFER, Some(&mat_buf_dst));
    unsafe {
        let _mat_buf_src = js_sys::Float32Array::view(&mat_buf_src);
        context.buffer_sub_data_with_i32_and_array_buffer_view(
            Gl::ARRAY_BUFFER, 0, &_mat_buf_src);
    }
    unsafe {
        STATE = Some(RenderState {
            _frame: request_animation_frame(render),
            _e_mouseup: mouseup,
            _e_mousedown: mousedown,
            _e_mousemove: mousemove,
            time: 0.0, n_vert,
            context, canvas,
            u_proj, u_view, proj, view,
            orbit, orbiting: false,
            vao, mat_buf_dst,
            mat_buf_src: &mat_buf_src,
            matrices,
        });
    }

    Ok(())
}

fn render(mut time: f64) {
    let s = unsafe { STATE.as_mut().unwrap() };
    let context = &s.context;

    time *= 0.001;
    let dt = time-s.time;
    s.time = time;

    //let rot = 0.5*dt as f32;
    //s.model.append_rotation_mut(
    //    &UnitQuaternion::<f32>::from_axis_angle(&Vector3::y_axis(), rot));

    context.uniform_matrix4fv_with_f32_array(
        Some(&s.u_proj), false, s.proj.as_slice());
    context.uniform_matrix4fv_with_f32_array(
        Some(&s.u_view), false, s.view.as_slice());

    context.viewport(
        0, 0,
        context.drawing_buffer_width(),
        context.drawing_buffer_height(),
    );
    context.enable(Gl::DEPTH_TEST);
    context.enable(Gl::CULL_FACE);
    context.clear_color(1.0, 1.0, 1.0, 1.0);
    context.clear_depth(1.0);
    context.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);
    context.bind_vertex_array(Some(&s.vao));
    // context.bind_buffer(Gl::ARRAY_BUFFER, Some(&s.mat_buf_dst));
    // unsafe {
    //     let mat_buf_src = js_sys::Float32Array::view(s.mat_buf_src);
    //     context.buffer_sub_data_with_i32_and_array_buffer_view(
    //         Gl::ARRAY_BUFFER, 0, &mat_buf_src);
    // }
    context.draw_arrays_instanced(Gl::TRIANGLES, 0, s.n_vert, num_inst as i32);
    //context.draw_arrays(Gl::TRIANGLES, 0, (vertices.len()/3) as i32);

    s._frame = request_animation_frame(render);
}

fn gl_compile_shader(
    context: &Gl,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("err: gl_compile: failed"))?;

    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("err: gl_compile: unknown error")))
    }
}

fn gl_link_program(
    context: &Gl,
    vert_shader: &web_sys::WebGlShader,
    frag_shader: &web_sys::WebGlShader,
) -> Result<web_sys::WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("err: gl_link: failed"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("err: gl_link: unknown error")))
    }
}