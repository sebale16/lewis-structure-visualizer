mod solve;
// mod format;

use std::env;
use crate::solve::parse_input;
use crate::solve::build_model;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let input_compound = parse_input(&args);

    use std::time::Instant;
    let now = Instant::now();
    let model_compound = build_model(&input_compound);
    let elapsed = now.elapsed();
    model_compound.print_model();
    println!("Elapsed: {:.2?}", elapsed);
}
