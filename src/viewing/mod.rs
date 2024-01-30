
use std::path::PathBuf;
use std::sync::Arc;

use eframe::{egui, egui_glow};
use egui_glow::glow;

use egui::mutex::Mutex;

use tf_demo_parser::demo::data::DemoTick;
use tf_demo_parser::demo::header::Header;

use crate::parsing::{ParseProgressReport, ParseWorker, ParseWorkerError, ParseDrawInfo};
use crate::parsing::internals::{InternalParse, InternalParseInstruction, InternalParseResult};
use crate::types::demo::DemoData;
use self::drawing::Drawing;

pub mod renderable;
pub mod shader;
pub mod drawing;

struct InternalParseUI {
    parse: InternalParse,
    header: Option<Header>
}

impl InternalParseUI {
    pub fn new(fpath: PathBuf) -> Self {
        InternalParseUI {
            parse: InternalParse::new(fpath),
            header: None
        }
    }

    pub fn draw_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> bool {
        let mut close_window = false;
        egui::Window::new("Internal Parse")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                use InternalParseInstruction::*;
                use InternalParseResult::*;
                if let Some(header) = &self.header {
                    let mut done_check = || {    
                        match self.parse.try_recv() {
                            Some(resp) => match resp {
                                Header(_) | Tick(_) => {},
                                Error | Done => {close_window = true;}
                            },
                            None => {}
                        }
                    };

                    ui.label(format!("HEADER: DemoType: {}, Version: {}, Protocol: {}",
                        header.demo_type,
                        header.version,
                        header.protocol
                    ));

                    ui.label(format!("Duration: {}, Ticks: {}, Frames: {}",
                        header.duration,
                        header.ticks,
                        header.frames
                    ));

                    ui.horizontal(|ui| {
                        if ui.button("Next Tick").clicked() {
                            self.parse.send(ParseNext);
                            done_check();
                        }
                        if ui.button("Next Data Table").clicked() {
                            self.parse.send(NextWithDataTable);
                            done_check();
                        }
                        if ui.button("Next String Entry").clicked() {
                            self.parse.send(NextWithStringEntry);
                            done_check();
                        }
                        if ui.button("Next Packet").clicked() {
                            self.parse.send(NextWithPacketMeta);
                            done_check();
                        }
                    });
                    ui.horizontal(|ui| {
                        let mut parse_count_input: u32 = 10;
                        ui.add(egui::DragValue::new(&mut parse_count_input).clamp_range(0..=header.ticks));
                        if ui.button("Next X Ticks").clicked() {
                            self.parse.send(ParseNextX(parse_count_input));
                            done_check();
                        }
                        if ui.button("Go To Tick").clicked() {
                            self.parse.send(ParseUntil(DemoTick::from(parse_count_input)));
                            done_check();
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("To End").clicked() {
                            self.parse.send(ParseUntilEnd);
                            done_check();
                        }
                        if ui.button("Stop").clicked() {
                            self.parse.send(StopParse);
                            done_check();
                        }
                    });
                } else {
                    self.header = match self.parse.recv() { 
                        Header(h) => Some(h),
                        _ => {panic!("did not get header");}
                    };
                }
                
                
            });
        !close_window
    }
}


#[derive(Default)]
struct OpenDemoToParseUI {
    selected_file: PathBuf,
    analyze: bool,
    internals: bool,
}

#[derive(Default)]
enum OpenDemoResult {
    #[default]
    Continue,
    Cancelled,
    Internals(InternalParseUI),
    FullParse(DemoViewUI),
}

impl OpenDemoToParseUI {
    pub fn draw_ui(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) -> (
        OpenDemoResult, // Result
        bool            // Load with analysis
    ){
        let mut ret = (OpenDemoResult::Continue, self.analyze);
        egui::Window::new("Load Demo File")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            //.min_size([300.0, 200.0])
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    let text = egui::RichText::new(self.selected_file.to_str().unwrap_or("None"))
                        .monospace()
                        .background_color(ui.style().visuals.faint_bg_color);

                    //ui.add_sized([700.0, 20.0],egui::Label::new(text));
                    ui.label(text);
                });

                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.analyze, "Analyze");
                        ui.checkbox(&mut self.internals, "Internals");

                        ui.horizontal(|ui| {
                            if ui.button("Browse").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.selected_file = path;
                                }
                            }

                            if ui.button("Load").clicked() {
                                if self.internals {
                                    ret.0 = OpenDemoResult::Internals(InternalParseUI::new(self.selected_file.clone()));
                                }
                                else {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                                        format!("TF2 Demo Info: {:?}", self.selected_file.file_name())
                                    ));
                                    
                                    ret.0 = OpenDemoResult::FullParse(DemoViewUI::new(self.selected_file.clone()));
                                }
                            }
                        
                            if ui.button("Cancel").clicked() {
                                ret.0 = OpenDemoResult::Cancelled;
                            }
                        });
                    });
                });
        
        ret
    }
}

#[derive(Default)]
struct DemoViewUI {
    // parsing in progress
    parse_worker: Option<ParseWorker>,
    parse_max_tick: u32,
    parse_draw_info: Option<ParseDrawInfo>,
    parse_data: Option<DemoData>,
    parse_file: PathBuf,

    draw_mutex: Option<Arc<Mutex<Drawing>>>,
    //data_transmitter: Option<DataTransmitter>,
    current_tick_view: u32,

    encountered_error: Option<ParseWorkerError>,
    closed_err: bool,
}

use crate::datatransmit::{launch_demo_analysis, launch_tick_analysis};

impl DemoViewUI {
    pub fn new(fpath: PathBuf) -> Self {
        match ParseWorker::new(fpath) {
            Ok(pw) => DemoViewUI {
                parse_worker: Some(pw),
                //data_transmitter: Some(DataTransmitter::new().unwrap()),
                ..Default::default()
            },
            Err(err) => DemoViewUI {
                encountered_error: Some(ParseWorkerError::IoError(err)),
                //data_transmitter: Some(DataTransmitter::new().unwrap()),
                ..Default::default()
            }
        }
    }

    pub fn set_draw_mutex(&mut self, draw_mutex: Option<Arc<Mutex<Drawing>>>) {
        self.draw_mutex = draw_mutex;
    }

    pub fn draw_ui(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) {
        // Check if it's done parsing
        if let Some(inprog) = &mut self.parse_worker {
            if self.parse_max_tick == 0 {
                match inprog.get_next() {
                    Some(report) => match report {
                        ParseProgressReport::Info(tickcount) => {
                            self.parse_max_tick = tickcount;
                        },
                        _ => {}
                    },
                    _ => {}
                }
            } else {
                match inprog.get_most_recent() {
                    Some(report) => {
                        match report {
                            ParseProgressReport::Info(tickcount) => {
                                self.parse_max_tick = tickcount;
                            },
                            ParseProgressReport::Working(tick) => {
                                egui::Window::new(format!("Parsing {:?}", self.parse_file.file_name().unwrap_or_default()))
                                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                                    .collapsible(false)
                                    .resizable(false)
                                    .show(ctx, |ui| {
                                        let max_tick = self.parse_max_tick;
                                        let cur_tick = tick;
                                    
                                        ui.vertical_centered(|ui| {
                                            if max_tick > 0 {
                                                ui.add(egui::ProgressBar::new(cur_tick as f32 / max_tick as f32));
                                            }
                                            ui.label(format!("{} / {}", cur_tick, max_tick));
                                        });
                                    
                                        // request repaint to repaint the progress bar
                                        ctx.request_repaint();
                                    });
                            },
                            ParseProgressReport::Error(err) => {
                                self.encountered_error = Some(err);
                                self.parse_worker.take();
                            },
                            ParseProgressReport::Done(data, drawdata) => {
                                self.parse_data = Some(data);
                                self.parse_draw_info = Some(drawdata);
                                self.parse_worker.take();
                            },
                            _ => {}
                        }
                    },
                    None => {}
                }
            }
        }
    
        else {
            // Display error
            if let Some(err) = self.encountered_error.as_ref() {
                egui::Window::new("Error")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!("{:?}", err));
                        if ui.button("OK").clicked() {
                            self.closed_err = true;
                        }
                    });
                
                if self.closed_err {
                    self.closed_err = false;
                    self.encountered_error.take();
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(result) = &self.parse_data {
                    ui.heading(format!("Viewing Demo: {:?}", result.demo_filename.file_name()));

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.custom_painting(ui);
                    });
    
                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = 300.0;
    
                        let min_tick = 0;
                        let max_tick = self.parse_max_tick;
    
                        let slider = egui::Slider::new(&mut self.current_tick_view, min_tick..=max_tick)
                            .step_by(100.0)
                            .drag_value_speed(50.0);
    
                        if ui.add(slider).changed() {
                            if let Some(dmutex) = &self.draw_mutex {
                                dmutex.lock().set_draw_info(self.parse_draw_info.as_ref().unwrap().clone());
                            }
                        }
                    });

                    if ui.button("Recompile Shaders").clicked() {
                        if let Some(dmutex) = &self.draw_mutex {
                            if dmutex.lock().attempt_recompile(_frame.gl().unwrap()) {
                                log::info!("Successfully recompiled shaders");
                            }
                        }
                    }
        
                    ui.horizontal(|ui| {
                        if ui.button("Analyze Demo").clicked() {
                            launch_demo_analysis(result);
                        }
                        if ui.button("Analyze Tick").clicked() {
                            if let Some(tickdata) = result.tick_states.get(&self.current_tick_view) {
                                launch_tick_analysis(tickdata);
                            }
                            else {
                                self.encountered_error = Some(ParseWorkerError::IoError(
                                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Tick has no tick state")
                                ));
                            }
                        }
                    });
                    
                }
                else {
                    ui.heading("Click File -> Open to parse a demo.");
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    powered_by_egui_and_eframe(ui);
                    egui::warn_if_debug_build(ui);
                });
            });
        }
    }

    fn custom_painting(&self, ui: &mut egui::Ui) {
        if let Some(dmutex) = &self.draw_mutex {
            if let Some(result) = &self.parse_data {
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(400.0), egui::Sense::drag());
                let drawing = dmutex.clone();
                let tick_data = result.tick_states.get(&self.current_tick_view).cloned();
        
                let cb = egui_glow::CallbackFn::new(move |_info, painter| {
                    drawing.lock().buffer_data(painter.gl(), &tick_data);
                    drawing.lock().paint(painter.gl());
                });
        
                let callback = egui::PaintCallback {
                    rect,
                    callback: Arc::new(cb),
                };
        
                ui.painter().add(callback);
            }
        }        
    }
}

//use crate::datatransmit::{DataTransmitter, INVALID_INPUT, BIT_ERROR_RET};

#[derive(Default)]
pub struct TemplateApp {
    /////////////////
    /// Internal parse data structures
    demo_open_window: Option<OpenDemoToParseUI>, 
    demo_view_ui: Option<DemoViewUI>,
    internal_parse: Option<InternalParseUI>,

    draw_mutex: Option<Arc<Mutex<Drawing>>>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let gl = cc.gl.as_ref().unwrap();
        TemplateApp {
            draw_mutex: Some(Arc::new(Mutex::new(Drawing::new(gl).unwrap()))),
            ..Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if let Some(intparse) = &mut self.internal_parse {
            if !intparse.draw_ui(ctx, frame) {
                self.internal_parse.take();
            }
        }

        if let Some(opendemo) = &mut self.demo_open_window {
            let (res, _with_analysis) = opendemo.draw_ui(ctx, frame);
            match res {
                OpenDemoResult::Cancelled => {
                    self.demo_open_window = None;
                },
                OpenDemoResult::Continue => {},
                OpenDemoResult::FullParse(p) => {
                    self.demo_view_ui = Some(p);
                    self.demo_view_ui.as_mut().unwrap().set_draw_mutex(self.draw_mutex.clone());
                    self.demo_open_window = None;
                },
                OpenDemoResult::Internals(p) => {
                    self.internal_parse = Some(p);
                    self.demo_open_window = None;
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        /*if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.current_tick_view = 0;
                            self.parse_progress = Some(parse_demo(path.clone()));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                                format!("TF2 Demo Info: {:?}", path.file_name())
                            ));
                        }*/
                        self.demo_open_window = Some(OpenDemoToParseUI::default());
                        ui.close_menu()
                    }
                    
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.add_space(16.0);
                
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        // TODO: grab analysis to view in viewing mode
        // egui::SidePanel::right("right_panel").show(ctx, |ui| {
        //     /* This panel is used to display analysis data. */
        //     egui::TopBottomPanel::top("right_inner_top_panel").show_inside(ui, |ui| {
        //         // Top panel gets tabs
        //         // What would I like to see:
        //         // - Groupings per team
        //         // - Player information
        //         // - Graphs of damage?
        //         // if let Some(parse) = &self.parse_result {

        //         // }
        //     });
        // });

        if let Some(demoview) = &mut self.demo_view_ui {
            demoview.draw_ui(ctx, frame);
        }
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        self.demo_view_ui.take();
        if let Some(gl) = gl {
            if let Some(dmutex) = &self.draw_mutex {
                dmutex.lock().destroy(gl);
            }
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}