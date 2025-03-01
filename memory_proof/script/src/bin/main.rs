use std::path::PathBuf;
use sp1_sdk::{ProverClient, SP1Stdin, utils};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Input file path
    #[clap(long)]
    input: Option<String>,
    
    /// Output file path
    #[clap(long)]
    output: Option<String>,
    
    /// Execute only (no proving)
    #[clap(long)]
    execute: bool,
    
    /// Generate proof
    #[clap(long)]
    prove: bool,
}

fn main() {
    // Setup logging
    utils::setup_logger();
    
    // Parse arguments
    let args = Args::parse();
    
    // Get ELF path
    let elf = sp1_sdk::include_elf!("memory_proof-program");
    
    // Read input data
    let mut stdin = SP1Stdin::new();
    
    if let Some(input_path) = args.input {
        let input_data = std::fs::read_to_string(input_path)
            .expect("Failed to read input file");
        
        let json_data: serde_json::Value = serde_json::from_str(&input_data)
            .expect("Failed to parse JSON input");
        
        // Write JSON data to stdin
        stdin.write_json(&json_data);
    }
    
    // Create ProverClient
    let client = ProverClient::from_env();
    
    if args.execute {
        // Execute only
        let (pub_values, _) = client.execute(elf, &stdin).run().unwrap();
        println!("Execution successful");
        
        // Save output if path provided
        if let Some(output_path) = args.output {
            std::fs::write(output_path, serde_json::to_string_pretty(&pub_values).unwrap())
                .expect("Failed to write output file");
        }
    } else if args.prove {
        // Generate proof
        let (pk, vk) = client.setup(elf);
        let proof = client.prove(&pk, &stdin).run().unwrap();
        
        println!("Proof generation successful");
        
        // Verify proof
        client.verify(&proof, &vk).expect("Verification failed");
        println!("Proof verified successfully");
        
        // Save output if path provided
        if let Some(output_path) = args.output {
            proof.save(&output_path).expect("Failed to save proof");
            println!("Proof saved to {}", output_path);
        }
    }
}