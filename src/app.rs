
use crate::parsing::internals::GatherSeen;
//use crate::types::demo::{DemoData, TickData};
use crate::parsing as par;
use crate::datatransmit as dt;

use std::path::PathBuf;
use std::io;
use itertools::enumerate;
use itertools::Itertools;
use tf_demo_parser::MessageType;

use log::{info, error};

use crate::types::demo::DemoData;

fn parse_singlethread(fnames: Vec<PathBuf>) -> io::Result<Vec<(PathBuf, DemoData)>> {
    info!("Single threaded parse!");
    let mut parse_results = Vec::new();
    for fname in fnames {
        let mut max_ticks = 0;
        let mut last_perc_printed = 0;
        par::parse_demo(fname.clone(), |prog_rep| {
            match prog_rep {
                par::ParseProgressReport::Info(max_tick) => {
                    info!("({:#?}) Max Ticks: {}", fname.file_name().unwrap(), max_tick);
                    max_ticks = max_tick;
                },
                par::ParseProgressReport::Error(err) => {
                    error!("({:#?}) Error: {:?}", fname.file_name().unwrap(), err);
                },
                par::ParseProgressReport::Working(tick) => {
                    let perc = (tick * 100) / max_ticks;
                    if perc > last_perc_printed && perc % 5 == 0 {
                        last_perc_printed = perc;
                        info!("({:#?}) {}%", fname.file_name().unwrap(), perc);
                    }
                },
                par::ParseProgressReport::Done(data, _) => {
                    parse_results.push((fname.clone(), data));
                },
                _ => {}
            }

            Ok(())
        });
    }

    Ok(parse_results)
}

pub fn do_parses(fnames: Vec<PathBuf>, multithread: bool) -> io::Result<Vec<(PathBuf, DemoData)>> {
    if !multithread {
        return parse_singlethread(fnames);
    }

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
                        info!("({:#?}) Max Ticks: {}", fname.file_name().unwrap(), max_tick);
                        *max_ticks = max_tick;
                    },
                    par::ParseProgressReport::Error(err) => {
                        error!("({:#?}) Error: {:?}", fname.file_name().unwrap(), err);
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
                info!("({:#?}) {}%", fname.file_name().unwrap(), perc_done);
                last_report[i] = *perc_done;
            }
        }
    }

    Ok(parse_results)
}

use io::Write;
use io::Read;

#[allow(dead_code)]
fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[allow(dead_code)]
pub fn run(fnames: Vec<PathBuf>, do_analysis: bool) -> io::Result<()> {
    info!("###############################");
    info!("# Beginning Parse: {:?}", fnames);

    if do_analysis {
        info!("# (with analysis!)");

        let parse_results = do_parses(fnames, true)?;

        info!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        info!("| Starting analysis with pyo3...");

        for (fname, demodata) in &parse_results {
            info!("| Analyzing: {:#?}", fname.file_name().unwrap().to_str());
            dt::launch_demo_analysis(demodata);
        }
    }

    else {
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        println!("| No analysis, so showing internals...");

        use tf_demo_parser::{Demo, DemoParser};
        use crate::parsing::internals::Gatherer;
        use std::collections::HashMap;

        for fname in fnames {
            let file = std::fs::read(fname)?;
            let demo = Demo::new(&file);

            let parser = DemoParser::new_all_with_analyser(
                demo.get_stream(), 
                Gatherer::default()
            );

            let (_, output) = parser.parse().unwrap();

            println!("Outer map: {:#?}", output.outer_map);

            println!("Seen is of length {:?}, with {:?} containing messages", output.seen.len(), output.seen_message_types.len());
            println!("Continuing will go through the [seen] vector, without printing packet meta. String entries will be collapsed.");
            //pause();

            fn match_remainder(seenitem: &GatherSeen, cur_print: &mut u32, cur_tick: &mut u32) {
                match seenitem {
                    GatherSeen::Header(_) => {println!("Header."); *cur_print += 1;},
                    GatherSeen::_DataTables(dt) => {
                        println!("% Data Tables @ {} ({}): {:?}", cur_tick, dt.len(), 
                            dt.iter().map(|name| name.to_string()).sorted().collect_vec());
                        *cur_print += 1;
                    },
                    GatherSeen::PacketMeta(tick) => {
                        //println!("# Packet Meta for tick {}", tick);
                        *cur_tick = *tick;
                    },
                    _ => {}
                }
            }

            println!("seen ent handles: ");
            for handle in output.seen_ent_handles {
                println!("      - {:08} ({:08X})", handle, handle);
            }
            println!("seen player entids: ");
            for pent in output.seen_player_entids {
                println!("      - {:08} ({:08X})", pent, pent);
            }

            let mut cur_iter = output.seen.iter();
            let mut cur_print: u32 = 0;
            let mut cur_tick: u32 = 0;
            while let Some(seenitem) = cur_iter.next() {
                match seenitem {
                    GatherSeen::StringEntry(tablename, _index, _entry) => {
                        //println!("$ String Entry {} @ {}: {:?}", tablename, index, entry);
                        //cur_print += 1;

                         // string table name, number of changes / entries
                         let mut cur_collected_strings = HashMap::<&String, u32>::new();
                         cur_collected_strings.insert(tablename, 1);
                         while let Some(nextitem) = cur_iter.next() {
                             match nextitem {
                                 GatherSeen::StringEntry(newtablename, _, _entry) => {
                                     *cur_collected_strings.entry(newtablename).or_insert(0) += 1;
                                 },
                                 other => {match_remainder(other, &mut cur_print, &mut cur_tick); break}
                             }
                         }

                        println!("$ String Entries @ {}: {:?}", cur_tick, cur_collected_strings);
                    }
                    _ => match_remainder(seenitem, &mut cur_print, &mut cur_tick)
                }

                if cur_print >= 10 {
                    //pause();
                    cur_print = 0;
                }
            }

            println!("|| Finished printing [seen]. Continuing will count the number of times each message type was sent.");
            //pause();

            let mut typecollect = HashMap::<u8, u32>::new();
            for (_, typelist) in output.seen_message_types {
                for mtype in typelist {
                    use MessageType::*;
                    let u8type = match mtype {
                        Empty => 0,
                        File => 2,
                        NetTick => 3,               // 98574; holy shet
                        StringCmd => 4,
                        SetConVar => 5,             // 1 hmm
                        SignOnState => 6,           // 3 ?? i expected 1
                        Print => 7,
                        ServerInfo => 8,            // 1
                        ClassInfo => 10,            // 1
                        SetPause => 11,
                        CreateStringTable => 12,
                        UpdateStringTable => 13,
                        VoiceInit => 14,            // 1; no one talked in game
                        VoiceData => 15,
                        ParseSounds => 17,          // 19022; wow
                        SetView => 18,              // 1; the server's view never changes
                        FixAngle => 19,
                        BspDecal => 21,
                        UserMessage => 23,          // 127
                        EntityMessage => 24,        // 407; entity message?
                        GameEvent => 25,            // 13275; oh yeah baby
                        PacketEntities => 26,       // 98573; the MEAT
                        TempEntities => 27,         // 21387
                        PreFetch => 28,             // 5168; i have no idea what this is
                        Menu => 29,
                        GameEventList => 30,        // 1; maybe this is the types that could possibly happen?
                        GetCvarValue => 31,
                        CmdKeyValues => 32,
                    };

                    *typecollect.entry(u8type).or_insert(0) += 1;
                }
            }

            for (mtype, count) in typecollect {
                println!("# {mtype}: {count}");
            }

            //pause();

            println!("\n{:?}", output.seen_packet_entities_types);

            //pause();

            println!("\n{:?}", output.seen_game_event_types);

            println!("{} interesting entities", output.interesting_entities.len());
            //pause();

            for (entname, props) in output.interesting_entities {
                println!("- {}: {} total props", entname, props.len());

                let mut cur_print = 0;
                for prop in props.iter().sorted() {
                    println!("% {:} = {:}", prop.0, prop.1);
                    cur_print += 1;

                    if cur_print > 100 {
                        pause();
                        cur_print = 0;
                    }
                }
            }

            //pause(); println!("\n");

            for (tick, eventlist) in output.interesting_events.iter().sorted_by_key(|x| x.0) {
                println!("- {}, {:?}", tick, eventlist);
            }

            //pause(); println!("\n");

            for table in output.interesting_datatable_entries {
                println!("- {:}", table.name.as_str());
                for prop in table.props {
                    println!(":: {:?}", prop);
                }
            }
        }
    }

    Ok(())
}