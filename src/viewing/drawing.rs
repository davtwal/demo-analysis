use std::env;

use eframe::egui_glow;
use egui_glow::glow;

use super::renderable::{Renderable, PlayerRendering, Loadable};
use super::shader::Shader;

use crate::types::demo::TickData;
use crate::parsing::ParseDrawInfo;

pub struct Drawing {
    dot_program: Shader,
    player_rendering: PlayerRendering,

    draw_info: ParseDrawInfo
}

impl Drawing {
    pub fn set_draw_info(&mut self, draw_info: ParseDrawInfo) {
        self.draw_info = draw_info;
    }

    pub fn new(gl: &glow::Context) -> Option<Self> {
        let shader_version = egui_glow::ShaderVersion::get(gl);

        println!("current dir: {}", env::current_dir().ok()?.display());

        let dot_program = Shader::new(gl, shader_version,
            "point.vert", "point.frag", None);

        if let None = dot_program {
            log::error!("Could not compile point fragment shader upon loading");
            return None;
        }

        println!("retry");
        Some(Self {
            dot_program: dot_program.unwrap(),
            player_rendering: PlayerRendering::new(gl),
            draw_info: ParseDrawInfo::default(),
        })
    }

    pub fn attempt_recompile(&mut self, gl: &glow::Context) -> bool {
        let shader_version = egui_glow::ShaderVersion::get(gl);
        let new_dot_prog = Shader::new(gl, shader_version,
            "point.vert", "point.frag", None);

        match new_dot_prog {
            Some(new_dot) => {
                self.dot_program.destroy(gl);
                self.dot_program = new_dot;
                true
            },
            None => false
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.dot_program.destroy(gl);
        self.player_rendering.destroy(gl);
    }

    pub fn buffer_data(&mut self, gl: &glow::Context, data: &Option<TickData>) {
        if let Some(data) = data {
            self.player_rendering.load(gl, Loadable::PlayerLoad(&data.players));
        }
    }

    pub fn paint(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            self.dot_program.bind(gl);
            gl.uniform_4_f32(
                gl.get_uniform_location(self.dot_program.program, "world_bounds").as_ref(),
                self.draw_info.player_at_max.bound_min.x - 50.0,
                self.draw_info.player_at_max.bound_min.y - 50.0,
                self.draw_info.player_at_max.bound_max.x + 50.0,
                self.draw_info.player_at_max.bound_max.y + 50.0,
            );
            self.player_rendering.paint(gl, None);

            //gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}