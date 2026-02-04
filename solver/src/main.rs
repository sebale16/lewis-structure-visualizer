mod solve;

use std::env;
use crate::solve::parse_input;
use crate::solve::build_model;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let input_molecule = parse_input(&args);

    use std::time::Instant;
    let now = Instant::now();
    let model_molecule = build_model(&input_molecule);
    let elapsed = now.elapsed();
    model_molecule.print_model();
    let _ = model_molecule.write_to_json(format!("out/{}_{}.json", args[1], args[2]));
    println!("Elapsed: {:.2?}", elapsed);
}
