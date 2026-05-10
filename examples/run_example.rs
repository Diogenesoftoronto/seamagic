use std::env;
use std::fs;
use base64::Engine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: cargo run --example run_example <mode> <input_image> <script_file> <output_image>");
        eprintln!("  mode: 'legacy' or 'scheme'");
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_path = &args[2];
    let script_path = &args[3];
    let output_path = &args[4];

    let script = fs::read_to_string(script_path).expect("Failed to read script file");

    if mode == "legacy" {
        let img = image::open(input_path).expect("Failed to open input image");
        let result = seamagic::script::run_script(&script, Some(img))
            .expect("Script execution failed");
        result.save(output_path).expect("Failed to save output image");
    } else if mode == "scheme" {
        let img_bytes = fs::read(input_path).expect("Failed to read input image");
        let b64 = base64::engine::general_purpose::STANDARD.encode(&img_bytes);
        let (_, result_b64) = seamagic::steel_engine::run_scheme_script(&script, Some(&b64))
            .expect("Scheme execution failed");
        let result_bytes = base64::engine::general_purpose::STANDARD.decode(&result_b64).expect("Failed to decode result");
        let result = image::load_from_memory(&result_bytes).expect("Failed to load result image");
        result.save(output_path).expect("Failed to save output image");
    } else {
        eprintln!("Unknown mode: {}. Use 'legacy' or 'scheme'", mode);
        std::process::exit(1);
    }

    println!("Saved result to {}", output_path);
}
