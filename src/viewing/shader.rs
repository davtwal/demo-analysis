use eframe::{egui_glow, glow::HasContext};
use egui_glow::glow;
use std::{fs, path::PathBuf};

pub struct Shader {
    pub program: glow::Program
}

impl Shader {
    fn compile_shader(
        gl: &glow::Context,
        _shader_version: egui_glow::ShaderVersion,
        shader_type: u32,
        fname: &str
    ) -> Result<glow::Shader, String> {
        use glow::HasContext as _;

        unsafe {
            let shader = gl
                .create_shader(shader_type)
                .expect("Cannot create shader");

            match fs::read_to_string(PathBuf::from("./src/shaders/").join(fname)) {
                Ok(source) => {
                    gl.shader_source(shader, &format!(
                        //"{}\n{}",
                        "{}", // Shader version is now inside of the shader file
                        //shader_version.version_declaration(),
                        //r#"#version 330 core"#,
                        source.as_str()
                    ));

                    gl.compile_shader(shader);

                    if gl.get_shader_compile_status(shader) == false {
                        return Err(format!("Failed to compile shader: {}",
                            gl.get_shader_info_log(shader)));
                    }
                },
                Err(e) => return Err(e.to_string())
            }

            Ok(shader)
        }
    }

    pub fn new(
        gl: &glow::Context,
        shader_version: egui_glow::ShaderVersion,
        vertex_fname: &str,
        fragment_fname: &str,
        geom_fname: Option<&str>
    ) -> Option<Self> {
        use glow::HasContext as _;

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let shader_sources = [
                (glow::VERTEX_SHADER, Some(vertex_fname)),
                (glow::GEOMETRY_SHADER, geom_fname),
                (glow::FRAGMENT_SHADER, Some(fragment_fname))
            ];

            let shaders: Vec<Option<Result<glow::Shader, String>>> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    match shader_source {
                        Some(source) => {
                            Some(Shader::compile_shader(gl, shader_version, *shader_type, *source))
                        },
                        None => None
                    }
                })
                .collect();

            let mut had_error: bool = false;
            for shader_result in &shaders {
                if let Some(result) = shader_result {
                    match result {
                        Ok(shader) => {
                            gl.attach_shader(program, *shader);
                        },
                        Err(e) => {
                            println!("{}", e);
                            had_error = true;
                        }
                    }
                }
            }

            if !had_error {
                gl.link_program(program);

                if gl.get_program_link_status(program) == false {
                    println!("Could not link program: {}", gl.get_program_info_log(program));
                    had_error = true;
                }
            }

            for shader_result in &shaders {
                if let Some(Ok(shader)) = shader_result {
                    gl.detach_shader(program, *shader);
                    gl.delete_shader(*shader);
                }
            }
            
            if had_error {
                gl.delete_program(program);
                return None;
            }

            Some(Shader {
                program
            })
        }
    }

    pub unsafe fn bind(&self, gl: &glow::Context) {
        gl.use_program(Some(self.program));
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
        }
    }
}