use adder_model::simulation::{run, events_to_json};
use adder_model::cell::SizeControlModel;
use adder_model::config::Config;
use adder_model::cli::{Cli, Model};


use rand::rngs::StdRng;
use rand::SeedableRng;
use clap::Parser;


fn main() {
    let cli = Cli::parse();
    let cfg = Config::from(&cli);
    let model = match cli.model {
      Model::Timer => SizeControlModel::Timer { period: cfg.timer_period() },
      Model::Sizer => SizeControlModel::Sizer { target_size: cfg.sizer_target() },
      Model::Adder => SizeControlModel::Adder { increment: cfg.adder_increment() },
      Model::AdderAlpha => SizeControlModel::AdderAlpha { alpha: cfg.alpha, v_c: cfg.v_c() },
  };
    eprintln!("{:?}", cfg);
    eprintln!("timer period (doubling time) = {:.4}", cfg.timer_period());
    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let all_events = run(model, &cfg, &mut rng, cli.n_max);

    eprintln!("division events:  {}", all_events.len());
    let json = events_to_json(&all_events).expect("serialization failed");
    println!("{json}");  // stdout for the FastAPI pipe
    
}
