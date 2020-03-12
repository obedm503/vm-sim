use std::env;
use std::fs;
use std::io::*;
use std::path;
use std::string::*;

mod sim;
use sim::*;

fn writes_to_memory(trace: String, algorithm: String) -> Vec<(u32, u32)> {
    // start with some guess
    // if a write occurs, abort, and restart with more pages
    let mut entries: Vec<(u32, u32)> = Vec::new();
    let mut n_pages = 0;

    loop {
        n_pages += 50;
        let sim = Sim::new(
            n_pages,
            algorithm.as_str().to_string(),
            trace.as_str().to_string(),
            false,
        );

        println!("  testing with {} pages", n_pages);

        let last = sim.last().unwrap() as SimState;

        entries.push((n_pages, last.write_count));

        if last.write_count == 0 {
            return entries;
        }
    }
}

fn get_data(trace: String, algorithm: &String) -> Result<()> {
    let stem = path::Path::new(trace.as_str())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let out = format!("out/{}-{}.csv", stem, algorithm);
    let path = path::Path::new(&out);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match fs::File::create(&path) {
        Ok(file) => file,
        Err(why) => panic!("couldn't create {}: {}", display, why),
    };

    println!("running {} algorithim for {} trace", algorithm, trace);
    let entries = writes_to_memory(trace.to_string(), algorithm.to_string());

    // header
    writeln!(file, "\"pages\",\"writes\"")?;
    for (pages, writes) in entries {
        writeln!(file, "{},{}", pages, writes)?;
    }

    println!(
        "  stored data for {} algorithm to {}\n\n",
        algorithm, display
    );

    return Ok(());
}

fn find_optimal_memory(trace: String, algorithm: String) -> u32 {
    // start with some guess
    // if a write occurs, abort, and restart with more pages

    let mut current_attempt = 20;
    let mut previous_attempt = current_attempt;
    let mut within_20 = false;

    'outer: loop {
        let sim = Sim::new(
            current_attempt,
            algorithm.as_str().to_string(),
            trace.as_str().to_string(),
            false,
        );

        for state in sim {
            // check write_count after every event
            if state.write_count > 0 {
                // a write has occurred
                if within_20 {
                    return current_attempt - 1;
                }

                // adjust attempted memory in increments of 20
                previous_attempt = current_attempt;
                current_attempt += 20;
                continue 'outer;
            }
        }

        // ran whole sim without any writes

        if within_20 && current_attempt >= previous_attempt {
            return current_attempt;
        }
        if within_20 {
            current_attempt += 1;
            continue;
        }

        current_attempt = previous_attempt + 1;
        within_20 = true;
    }
}

fn get_args() -> Result<(u32, String, bool, String)> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        panic!("Expected arguments to be in format <nframes> <random|lru|fifo> <quiet|debug> <tracefile>");
    }

    let n_frames: u32 = match u32::from_str_radix(args.get(1).unwrap(), 10) {
        Ok(arg) => arg,
        Err(e) => panic!("Expected an eviction algorithm: {}", e),
    };
    let algorithm: String = args.get(2).unwrap().to_owned();
    let debug: bool = match args.get(3).unwrap().as_str() {
        "quiet" => false,
        "debug" => true,
        _ => panic!("Expected trace file path"),
    };
    let trace: String = args.get(4).unwrap().to_owned();

    return Ok((n_frames, algorithm, debug, trace));
}

fn main() -> Result<()> {
    let traces = vec![
        "traces/gcc.trace",
        "traces/sixpack.trace",
        "traces/swim.trace",
    ];
    let algorithms = vec!["lru", "fifo", "random"];

    let args = env::args().collect::<Vec<String>>();
    let mode = args.get(1).unwrap().as_str();

    if mode == "memory" {
        for trace in traces {
            for algorithm in algorithms.to_vec() {
                let optimal_memory_size =
                    find_optimal_memory(trace.to_string(), algorithm.to_string());
                println!(
                    "optimal memory for {} trace with {} algorithm is {} pages",
                    trace, algorithm, optimal_memory_size
                );
            }
        }
        return Ok(());
    }

    if mode == "data" {
        let out_dir = path::Path::new("out");
        if !out_dir.exists() {
            fs::create_dir(out_dir)?;
        }

        // run for specific algorithm
        if let Some(algorithim) = args.get(2) {
            for trace in traces {
                get_data(trace.to_string(), algorithim)?;
            }

            return Ok(());
        }

        for trace in traces {
            for algorithm in algorithms.to_vec() {
                get_data(trace.to_string(), &algorithm.to_string())?;
            }
        }
        return Ok(());
    }

    let (n_frames, algorithm, debug, trace_file) = get_args()?;

    let state = Sim::new(n_frames, algorithm, trace_file, debug)
        .last()
        .unwrap();

    println!(
        "total memory frames: {}\nevents in trace:     {}\ntotal disk reads:    {}\ntotal disk writes:   {}",
        n_frames, state.total_events, state.read_count, state.write_count
    );

    return Ok(());
}
