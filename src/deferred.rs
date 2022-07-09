use rustc_hash::FxHashMap;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{
    WebGl2RenderingContext,
    WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
    WebGlUniformLocation,
    WebGlFramebuffer,
    WebGlTexture,
};

type Gl = WebGl2RenderingContext;

pub struct RenderPass {
    pub context: Gl,
    pub shader: WebGlProgram,
    pub fbo: WebGlFramebuffer,
    pub vao: WebGlVertexArrayObject,
    pub uniforms: FxHashMap<&'static str, WebGlUniformLocation>,
    pub attributes: FxHashMap<&'static str, u32>,
    pub draw_buffers: FxHashMap<&'static str, u32>,
}

impl RenderPass {
    pub fn new(
        context: Gl,
        vert_src: &str,
        frag_src: &str,
        uniform_vars: &[&'static str],
        attribute_vars: &[&'static str],
        draw_buf_vars: &[&'static str],
    ) -> Self {
        let vert = gl_compile_shader(&context, Gl::VERTEX_SHADER, vert_src);
        let frag = gl_compile_shader(&context, Gl::FRAGMENT_SHADER, frag_src);
        let shader = gl_link_program(&context, &vert, &frag);
        let fbo = context
            .create_framebuffer()
            .expect_throw("err: create_framebuffer");
        let vao = context
            .create_vertex_array()
            .expect_throw("err: create_vertex_array");
        let uniforms = FxHashMap::from_iter(
            uniform_vars.into_iter().map(
                |i| (*i, context.get_uniform_location(&shader, *i)
                                .expect_throw("err: uniform_location"))
            )
        );
        let attributes = FxHashMap::from_iter(
            attribute_vars.into_iter().map(
                |i| (*i, context.get_attrib_location(&shader, *i) as u32)
            )
        );
        let draw_buffers = FxHashMap::from_iter(
            draw_buf_vars.into_iter().map(
                |i| (*i, context.get_frag_data_location(&shader, *i) as u32)
            )
        );

        Self {context, shader, fbo, vao, uniforms, attributes, draw_buffers}
    }

    pub fn attrib_buffer(
        &self,
        attribute_var: &str,
        buf_src: &[f32],
        size: i32,
        stride: i32,
        offset: i32,
        normalize: bool,
        divisor: u32,
    ) {
        let context = &self.context;

        context.bind_vertex_array(Some(&self.vao));
        let buf_dst = context
            .create_buffer()
            .expect_throw("err: create_buffer");
        context.bind_buffer(Gl::ARRAY_BUFFER, Some(&buf_dst));

        unsafe {
            let buf_view = js_sys::Float32Array::view(buf_src);
            context.buffer_data_with_array_buffer_view(
                Gl::ARRAY_BUFFER, &buf_view, Gl::STATIC_DRAW);
        }

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
        samples: i32,
        format: u32,
        attachment: u32,
    ) {
        let context = &self.context;

        context.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));
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
        unit: u32,
        format: u32,
        attachment: u32,
    ) -> WebGlTexture {
        let context = &self.context;

        context.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));
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
        vars: &[u32],
    ) {
        self.context.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));

        let len = self.draw_buffers.values().len();
        let args = js_sys::Array::new_with_length(len as u32);
        for i in 0..len {
            args.set(i as u32, vars[i].into());
        }
        self.context.draw_buffers(&args.into());

        self.context.bind_framebuffer(Gl::FRAMEBUFFER, None);
    }

    pub fn active(&self) {
        self.context.use_program(Some(&self.shader));
        self.context.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));
        self.context.bind_vertex_array(Some(&self.vao));
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
) -> WebGlProgram {
    let program = context
        .create_program()
        .expect_throw("err: gl_link: failed");

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    context
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_throw()
        .then(|| ())
        .expect_throw(&context.get_program_info_log(&program).unwrap_throw());

    program
}
