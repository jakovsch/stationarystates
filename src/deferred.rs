use rustc_hash::FxHashMap;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{
    WebGl2RenderingContext,
    WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
    WebGlUniformLocation,
    WebGlFramebuffer,
    WebGlTexture,
    WebGlBuffer,
};

type Gl = WebGl2RenderingContext;

pub struct RenderPass {
    pub context: Gl,
    pub shader: WebGlProgram,
    pub fbos: Vec<WebGlFramebuffer>,
    pub vaos: Vec<WebGlVertexArrayObject>,
    pub uniforms: FxHashMap<&'static str, WebGlUniformLocation>,
    pub attributes: FxHashMap<&'static str, u32>,
    pub draw_buffers: FxHashMap<&'static str, u32>,
}

impl RenderPass {
    pub fn new(
        context: Gl,
        n_fbos: usize,
        n_vaos: usize,
        vert_src: &str,
        frag_src: &str,
        uniform_vars: Option<&[&'static str]>,
        attribute_vars: Option<&[&'static str]>,
        draw_buf_vars: Option<&[&'static str]>,
        xfb_varyings: Option<&[&'static str]>,
    ) -> Self {
        let vert = gl_compile_shader(&context, Gl::VERTEX_SHADER, vert_src);
        let frag = gl_compile_shader(&context, Gl::FRAGMENT_SHADER, frag_src);
        let shader = gl_link_program(&context, &vert, &frag, xfb_varyings);
        let fbos = (0..n_fbos).map(|_|
            context.create_framebuffer()
                .expect_throw("err: create_framebuffer"))
            .collect();
        let vaos = (0..n_vaos).map(|_|
            context.create_vertex_array()
                .expect_throw("err: create_vertex_array"))
            .collect();
        let uniforms = match uniform_vars {
            Some(vars) => FxHashMap::from_iter(vars.into_iter().map(
                    |i| (*i, context.get_uniform_location(&shader, *i)
                                    .expect_throw("err: uniform_location"))
                )),
            None => FxHashMap::default(),
        };
        let attributes = match attribute_vars {
            Some(vars) => FxHashMap::from_iter(vars.into_iter().map(
                    |i| (*i, context.get_attrib_location(&shader, *i) as u32)
                )),
            None => FxHashMap::default(),
        };
        let draw_buffers = match draw_buf_vars {
            Some(vars) => FxHashMap::from_iter(vars.into_iter().map(
                    |i| (*i, context.get_frag_data_location(&shader, *i) as u32)
                )),
            None => FxHashMap::default(),
        };

        Self {context, shader, fbos, vaos, uniforms, attributes, draw_buffers}
    }

    pub fn buffer_alloc(
        &self,
        size: i32,
        hint: u32,
    ) -> WebGlBuffer {
        let context = &self.context;

        let buffer = context
            .create_buffer()
            .expect_throw("err: create_buffer");
        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&buffer));
        context.buffer_data_with_i32(
            Gl::ARRAY_BUFFER, size, hint,
        );
        context.bind_buffer(Gl::ARRAY_BUFFER, None);

        buffer
    }

    pub fn buffer_data(
        &self,
        data: &[f32],
        hint: u32,
    ) -> WebGlBuffer {
        let context = &self.context;

        let buffer = context
            .create_buffer()
            .expect_throw("err: create_buffer");
        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let view = js_sys::Float32Array::view(data);
            context.buffer_data_with_array_buffer_view(
                Gl::ARRAY_BUFFER, &view, hint,
            );
        }

        context.bind_buffer(Gl::ARRAY_BUFFER, None);

        buffer
    }

    pub fn vao_buffer(
        &self,
        vao: usize,
        buffer: &WebGlBuffer,
        attribute_var: &str,
        size: i32,
        stride: i32,
        offset: i32,
        normalize: bool,
        divisor: u32,
    ) {
        let context = &self.context;

        context.bind_vertex_array(self.vaos.get(vao));
        context.bind_buffer(Gl::ARRAY_BUFFER, Some(buffer));

        let attrib = *self.attributes
            .get(attribute_var)
            .expect_throw("err: attribute name");
        context.enable_vertex_attrib_array(attrib);
        context.vertex_attrib_pointer_with_i32(
            attrib, size, Gl::FLOAT, normalize, stride, offset);
        if divisor > 0 {
            context.vertex_attrib_divisor(attrib, divisor);
        }

        context.bind_buffer(Gl::ARRAY_BUFFER, None);
        context.bind_vertex_array(None);
    }

    pub fn fb_renderbuffer(
        &self,
        fbo: usize,
        samples: i32,
        format: u32,
        attachment: u32,
    ) {
        let context = &self.context;

        context.bind_framebuffer(Gl::FRAMEBUFFER, self.fbos.get(fbo));
        let rbo = context
            .create_renderbuffer()
            .expect_throw("err: create_renderbuffer");
        context.bind_renderbuffer(Gl::RENDERBUFFER, Some(&rbo));

        context.renderbuffer_storage_multisample(
            Gl::RENDERBUFFER,
            samples, format,
            context.drawing_buffer_width(),
            context.drawing_buffer_height(),
        );

        context.framebuffer_renderbuffer(
            Gl::FRAMEBUFFER,
            attachment,
            Gl::RENDERBUFFER,
            Some(&rbo),
        );

        context.bind_renderbuffer(Gl::RENDERBUFFER, None);
        context.bind_framebuffer(Gl::FRAMEBUFFER, None);
    }

    pub fn fb_texture(
        &self,
        fbo: usize,
        unit: u32,
        format: u32,
        attachment: u32,
    ) -> WebGlTexture {
        let context = &self.context;

        context.bind_framebuffer(Gl::FRAMEBUFFER, self.fbos.get(fbo));
        let tex = context
            .create_texture()
            .expect_throw("err: create_texture");
        context.active_texture(unit);
        context.bind_texture(Gl::TEXTURE_2D, Some(&tex));

        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
        context.tex_storage_2d(
            Gl::TEXTURE_2D,
            1, format,
            context.drawing_buffer_width(),
            context.drawing_buffer_height(),
        );

        context.framebuffer_texture_2d(
            Gl::FRAMEBUFFER,
            attachment,
            Gl::TEXTURE_2D,
            Some(&tex),
            0,
        );

        context.bind_texture(Gl::TEXTURE_2D, None);
        context.bind_framebuffer(Gl::FRAMEBUFFER, None);

        tex
    }

    pub fn uniform_texture(
        &self,
        var: &str,
        val: &WebGlTexture,
        unit: u32,
    ) {
        let uniform = self.uniforms.get(var);
        self.context.active_texture(unit);
        self.context.bind_texture(Gl::TEXTURE_2D, Some(val));
        self.context.uniform1i(uniform, (unit - Gl::TEXTURE0) as i32);
    }

    pub fn uniform_mat4(
        &self,
        var: &str,
        val: &nalgebra::Matrix4<f32>,
    ) {
        let uniform = self.uniforms.get(var);
        self.context.uniform_matrix4fv_with_f32_array(
            uniform, false, val.as_slice()
        );
    }

    pub fn uniform_vec3(
        &self,
        var: &str,
        val: &nalgebra::Vector3<f32>,
    ) {
        let uniform = self.uniforms.get(var);
        self.context.uniform3fv_with_f32_array(
            uniform, val.as_slice()
        );
    }

    pub fn uniform_float(
        &self,
        var: &str,
        val: f32,
    ) {
        let uniform = self.uniforms.get(var);
        self.context.uniform1f(uniform, val);
    }

    pub fn set_draw_buffers(
        &self,
        fbo: usize,
        vars: &[u32],
    ) {
        self.context.bind_framebuffer(Gl::FRAMEBUFFER, self.fbos.get(fbo));

        let len = self.draw_buffers.values().len();
        let args = js_sys::Array::new_with_length(len as u32);
        for i in 0..len {
            args.set(i as u32, vars[i].into());
        }
        self.context.draw_buffers(&args.into());

        self.context.bind_framebuffer(Gl::FRAMEBUFFER, None);
    }

    pub fn active(
        &self,
        fbo: usize,
        vao: usize,
    ) {
        self.context.use_program(Some(&self.shader));
        self.context.bind_framebuffer(Gl::FRAMEBUFFER, self.fbos.get(fbo));
        self.context.bind_vertex_array(self.vaos.get(vao));
    }
}

fn gl_compile_shader(
    context: &Gl,
    shader_type: u32,
    source: &str,
) -> WebGlShader {
    let shader = context
        .create_shader(shader_type)
        .expect_throw("err: gl_compile: failed");

    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    context
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_throw()
        .then(|| ())
        .expect_throw(&context.get_shader_info_log(&shader).unwrap_throw());

    shader
}

fn gl_link_program(
    context: &Gl,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
    xfb_varyings: Option<&[&'static str]>,
) -> WebGlProgram {
    let program = context
        .create_program()
        .expect_throw("err: gl_link: failed");

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    if let Some(vars) = xfb_varyings {
        let len = vars.len();
        let args = js_sys::Array::new_with_length(len as u32);
        for i in 0..len {
            args.set(i as u32, vars[i].into());
        }
        context.transform_feedback_varyings(
            &program, &args.into(), Gl::SEPARATE_ATTRIBS,
        );
    }
    context.link_program(&program);

    context
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_throw()
        .then(|| ())
        .expect_throw(&context.get_program_info_log(&program).unwrap_throw());

    program
}
