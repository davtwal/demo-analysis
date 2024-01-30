use std::mem::size_of;
use glow::HasContext;
//use tf_demo_parser::demo::parser::gamestateanalyser::{Player, Team};
use eframe::egui_glow::glow;
//use crate::analysis::Grouping;

use crate::types::game::entities::Player;
use crate::types::game::Team;

pub trait Renderable {
    // Creates any necessary gl constructs (buffer, etc.) for rendering.
    fn create(gl: &glow::Context) -> Self;

    // Populates constructed buffers.
    fn load(&mut self, gl: &glow::Context, data: Loadable<'_>);

    // Draws
    fn paint(&self, gl: &glow::Context, shader: Option<glow::Program>);

    // Deletes all previously constructed buffers.
    fn destroy(&self, gl: &glow::Context);
}

pub enum Loadable<'a>  {
    PlayerLoad(&'a Vec<Player>),
    //GroupingLoad(&'a Vec<Grouping>),
}

pub struct PlayerRendering {
    // Contains data for rendering the data of a player.
    // Players are typically rendered as points.
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    point_count: u16,
}

impl PlayerRendering {
    const ARRAY_BUFF_SIZE: usize = 7 * size_of::<f32>();

    pub fn new(gl: &glow::Context) -> Self {
        PlayerRendering::create(gl)
    }
}

// Players are all rendered 
impl Renderable for PlayerRendering {
    fn create(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        unsafe {
            let vao = gl.create_vertex_array().expect("could not create vao");

            let vbo = gl.create_buffer().expect("could not create vbo");
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            // 0 = vec4 color;
            // 1 = vec3 point;

            gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, 7*size_of::<f32>() as i32, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 7*size_of::<f32>() as i32, 4*size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(1);
            
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            PlayerRendering {
                vao,
                vbo,
                point_count: 0,
            }
        }
        
    }

    // Fills the VBOs and such with the required data to draw a player.
    fn load(&mut self, gl: &glow::Context, data: Loadable<'_>) {
        match data {
            Loadable::PlayerLoad(playerlist) => {
                use glow::HasContext as _;
                unsafe {
                    gl.bind_vertex_array(Some(self.vao));
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

                    let mut raw: Vec<u8> 
                        = Vec::with_capacity(playerlist.len()*PlayerRendering::ARRAY_BUFF_SIZE);

                    self.point_count = playerlist.len() as u16;

                    for player in playerlist {
                        let mut color: Vec<f32> = match player.team {
                            Team::Red => vec![1.0, 0.2, 0.2, 1.0],
                            Team::Blue => vec![0.2, 0.2, 1.0, 1.0],
                            _ => vec![0.0; 4]
                        };
    
                        let mut position: Vec<f32> = 
                            vec![player.position.x, player.position.y, player.position.z];

                        color.append(&mut position);
                        assert_eq!(color.len() * size_of::<f32>(), PlayerRendering::ARRAY_BUFF_SIZE);

                        raw.extend_from_slice(std::slice::from_raw_parts(
                            color.as_ptr() as *const u8,
                            color.len() * size_of::<f32>()
                        ));
                    }

                    assert_eq!(raw.len(), 7*size_of::<f32>()*playerlist.len(), "raw byte/player len equality");
                    //print!("{:?}", raw);
                    //gl.buffer_data_size(glow::ARRAY_BUFFER, (raw.len()) as i32, glow::DYNAMIC_DRAW);
                    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw.as_slice(), glow::DYNAMIC_DRAW);
                    //gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, raw.as_slice());
        
                }
            }
            //_ => { panic!("wrong loadable type given to player load"); }
        }
        
    }

    fn paint(&self, gl: &glow::Context, _shader: Option<glow::Program>) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.draw_arrays(glow::POINTS, 0, self.point_count as i32);
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }
    }
}