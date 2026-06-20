use clap::{Parser, ValueEnum};


/// toy clap parser to explore te clap crate and experimemt
#[derive(Parser)]
struct Cli {
    /// name input from user
    #[arg(short, long)]
    model: Model, 
    /// count input from user
    #[arg(short, long, default_value_t=1)]
    count: u8, 
    #[arg(short, long, default_value_t=0.5)]
    growthrate: f64
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Model {
    Timer,
    Sizer,
    Adder
}



fn main() {
    let cli = Cli::parse();
    println!("model: {:#?}", cli.model);
    println!("count: {}", cli.count);
    println!("growthrate: {}", cli.growthrate);
}
