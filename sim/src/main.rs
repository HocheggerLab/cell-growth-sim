mod cell;
mod config;
mod simulation; 

use crate::simulation::{run, events_to_json};
use crate::cell::SizeControlModel;
use crate::config::Config;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::write;

fn main() {
    let cfg = Config::default();
    let model = SizeControlModel::Timer { period: cfg.timer_period()};
    eprintln!("{:?}", cfg);
    eprintln!("timer period (doubling time) = {:.4}", cfg.timer_period());
    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let all_events = run(model, &cfg, &mut rng, 20);

    eprintln!("division events:  {}", all_events.len());
    let json = events_to_json(&all_events).expect("serialization failed");
    // DEV:   write to a file for marimo
    // LATER: println!("{json}");  // stdout for the FastAPI pipe
    write("../data/events.json", json).expect("writing the json file failed")
}
