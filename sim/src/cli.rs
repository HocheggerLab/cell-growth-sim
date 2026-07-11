use clap::{Parser, ValueEnum};
use crate::config::Config;

/// Cell growth & division simulator — runs a size-control model and streams
/// one JSON division-event record per division to stdout.
#[derive(Parser)]
#[command(allow_negative_numbers = true)]
pub struct Cli {
    /// growth model for the sim: sizer, timer or adder
    #[arg(long)]
    pub model: Model, 
    /// basic growth rate, default is 0.5
    #[arg(long, default_value_t=Config::default().growth_rate)]
    pub growthrate: f64,
    /// starting volume, 1
    #[arg(long, default_value_t=Config::default().initial_volume)]
    pub initial_volume: f64,
    /// time increment, 01
    #[arg(long, default_value_t=Config::default().dt)]
    pub dt: f64,
    /// stochastic variation of split volume, 0.05
    #[arg(long, default_value_t=Config::default().split_noise)]
    pub split_noise: f64,
    /// stochastic variation of threshols 0.1
    #[arg(long, default_value_t=Config::default().threshold_noise_cv)]
    pub threshold_noise_cv: f64,
    /// random seed to get reproducible results
    #[arg(long, default_value_t=Config::default().seed)]
    pub seed: u64,
    /// maxiumum number of division cycles
    #[arg(long, default_value_t=10000)]
    pub n_max: usize,
    /// alpha value for alpha model
    #[arg(long, default_value_t=0.0)]
    pub alpha: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Model {
    Timer,
    Sizer,
    Adder,
    AdderAlpha,
}

impl From<&Cli> for Config {
    fn from(cli: &Cli) -> Self { Config {
        growth_rate: cli.growthrate,
        initial_volume: cli.initial_volume,
        dt: cli.dt,
        split_noise: cli.split_noise,
        threshold_noise_cv: cli. threshold_noise_cv,
        seed: cli.seed,
        alpha: cli.alpha
        }
    }
}