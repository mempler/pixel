use std::ffi::{CStr, CString};

use crate::gl;

pub struct Shader {
    program: u32,
}

const SHADER_ERR_SRC_FRAG: &str = "
#version 110

varying vec2 TexPos;

// Yoinked from: https://github.com/mattdesl/glsl-checker

float checker(vec2 uv, float repeats)
{
  float cx = floor(repeats * uv.x);
  float cy = floor(repeats * uv.y);
  float result = mod(cx + cy, 2.0);

  return sign(result);
}

void main()
{
    gl_FragColor = mix(vec4(1.0, 0.0, 1.0, 1.0), vec4(0.0, 0.0, 0.0, 1.0), checker(TexPos.xy, 15.0));
}
";

const SHADER_ERR_SRC_VERT: &str = "
#version 110
attribute vec3 iPos;
attribute vec2 iTexPos;

varying vec2 TexPos;

uniform mat4 iMVP;

void main()
{
    gl_Position = iMVP * vec4( iPos.xyz, 1.0 );
    TexPos = iTexPos;
}
";

impl Shader {
    pub fn new<S: AsRef<str>>(frag: S, vert: S) -> Option<Shader> {
        let program;
        unsafe {
            program = gl::CreateProgram();
            let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);

            let vert_len = vert.as_ref().len() as i32;
            let frag_len = frag.as_ref().len() as i32;

            let vert_ptr = vert.as_ref().as_ptr() as *const i8;
            let frag_ptr = frag.as_ref().as_ptr() as *const i8;

            // Set our shader source
            gl::ShaderSource(vert_shader, 1, &vert_ptr, &vert_len);
            gl::ShaderSource(frag_shader, 1, &frag_ptr, &frag_len);

            // Compile our shader
            gl::CompileShader(vert_shader);
            if let Some(val) = Shader::check_compilation("vert", vert_shader) {
                gl::DeleteProgram(program);

                return Some(val);
            }

            gl::CompileShader(frag_shader);
            if let Some(val) = Shader::check_compilation("frag", frag_shader) {
                gl::DeleteProgram(program);

                return Some(val);
            }

            // Attach our shader to our Program
            gl::AttachShader(program, vert_shader);
            gl::AttachShader(program, frag_shader);

            // Link the shaders together into one program
            gl::LinkProgram(program);

            // Delete the source objects
            gl::DeleteShader(vert_shader);
            gl::DeleteShader(frag_shader);
        }

        // Boom, we got a working shader program for our GPU.
        Some(Shader {
            program
        })
    }

    // If error occurs, it'll get printed out and a purple error shader will be returned
    // TODO: actually return the error shader
    unsafe fn check_compilation<S: AsRef<str>>(name: S, shader: u32) -> Option<Shader> {
        let mut is_compiled = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut is_compiled);
        let is_compiled = is_compiled != 0;

        if !is_compiled {
            let mut max_len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut max_len);

            // Create a CStr
            let mut error_log = vec![0i8; max_len as usize];
            gl::GetShaderInfoLog(shader, max_len, &mut max_len,
                                 error_log.as_mut_ptr());

            let c_str = CStr::from_ptr(error_log.as_ptr());

            log::error!("Failed to compile {} shader {}", name.as_ref(),
                CString::from(c_str).to_str().unwrap());

            gl::DeleteShader(shader); // Don't leak the shader.

            return Shader::new(SHADER_ERR_SRC_FRAG, SHADER_ERR_SRC_VERT)
        }

        None
    }

    pub fn uniform_mat4f<S: AsRef<str>>(&self, name: S, val: &glm::Mat4) {
        unsafe {
            let c_str = CString::new(name.as_ref()).unwrap();
            let uni_loc = gl::GetUniformLocation(self.program, c_str.as_ptr());

            if uni_loc < 0 {
                log::warn!("uniform_mat4f {} was not found", name.as_ref());
            }

            gl::UniformMatrix4fv(uni_loc, 1, gl::FALSE, val.as_ptr());
        }
    }

    pub fn id(&self) -> u32 {
        self.program
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}