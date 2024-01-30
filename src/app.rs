
//use crate::types::demo::{DemoData, TickData};
use crate::parsing as par;
use crate::datatransmit as dt;

use std::path::PathBuf;
use std::io;
use itertools::enumerate;

pub fn run(fnames: Vec<PathBuf>, do_analysis: bool) -> io::Result<()> {
    println!("###############################");
    println!("# Beginning Parse: {:?}", fnames);
    if do_analysis {println!("# (with analysis!)");}
    
    let mut workers = Vec::new();
    for f in fnames {
        workers.push((f.clone(), par::ParseWorker::new(f)?, u32::MAX, 0));
    }

    let mut parse_results = Vec::new();
    let mut done_count = 0;
    let mut last_report = vec![0; workers.len()];
    while done_count < workers.len() {
        for (fname, worker, max_ticks, perc_done) in &mut workers {
            if let Some(prog_rep) = worker.get_most_recent() {
                match prog_rep {
                    par::ParseProgressReport::Info(max_tick) => {
                        println!("({:#?}) Max Ticks: {}", fname.file_name().unwrap(), max_tick);
                        *max_ticks = max_tick;
                    },
                    par::ParseProgressReport::Error(err) => {
                        println!("({:#?}) Error: {:?}", fname.file_name().unwrap(), err);
                        *perc_done = 100;
                        done_count += 1;
                    },
                    par::ParseProgressReport::Working(tick) => {
                        *perc_done = (tick * 100) / *max_ticks;
                    },
                    par::ParseProgressReport::Done(data, _) => {
                        parse_results.push((fname.clone(), data));
                        done_count += 1;
                    },
                    _ => {}
                }
            }
        }
    
        for (i, (fname, _, _, perc_done)) in enumerate(&workers) {
            if perc_done % 10 == 0 && perc_done > last_report.get(i).unwrap(){
                println!("({:#?}) {}%", fname.file_name().unwrap(), perc_done);
                last_report[i] = *perc_done;
            }
        }
    }

    if do_analysis {
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        println!("| Starting analysis with pyo3...");

        for (fname, demodata) in &parse_results {
            println!("| Analyzing: {:#?}", fname.file_name().unwrap().to_str());
            dt::launch_demo_analysis(demodata);
        }
    }

    Ok(())
}