use clap::Parser;

//mod drawing;
mod parsing;
mod types;
mod viewing;
mod analysis;
mod app;
mod datatransmit;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The demo files to parse. If viewing with a window, only the first will be parsed.
    filenames: Option<Vec<std::path::PathBuf>>,

    /// Automatically do analysis or not.
    #[arg(short)]
    analysis: bool,

    /// Stop the window from showing up. Without analysis, does nothing.
    #[arg(short)]
    no_window: bool,
}

fn main() -> eframe::Result<()> {
    let args = Args::parse();

    println!("filenames: {:?} ; analysis: {:?} ; no_window: {:?}",
        args.filenames, args.analysis, args.no_window);

    if !args.no_window {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([300.0, 220.0])
            .with_title("TF2 Data Analyzer"),
            ..Default::default()
        };

        eframe::run_native(
            "tf2 demo info", 
            native_options,
            Box::new(|cc| Box::new(viewing::TemplateApp::new(cc))),
        )?;
    }
    else if let Some(fnames) = args.filenames {
        if fnames.len() > 0 {
            app::run(fnames, args.analysis).unwrap();
        }
    }

    Ok(())
}