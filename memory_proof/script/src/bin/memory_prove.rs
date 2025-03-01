use sp1_sdk::{include_elf, utils, ProverClient, SP1Stdin};
use std::env;

fn main() {
    // Setup logging
    utils::setup_logger();
    
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Default values
    let mut moves = 15u32;
    let mut time = 30u32;
    let mut matched_pairs = 8u32;
    
    // Use provided arguments if available
    if args.len() >= 4 {
        moves = args[1].parse().unwrap_or(15);
        time = args[2].parse().unwrap_or(30);
        matched_pairs = args[3].parse().unwrap_or(8);
    }
    
    println!("Memory Game SP1 Proof Generator");
    println!("-------------------------------");
    println!("Game data: Moves={}, Time={}, Matched Pairs={}", moves, time, matched_pairs);
    
    // Load the ELF - memory-proof-program
    let elf = include_elf!("memory-proof-program");
    
    // Prepare SP1 input stream
    let mut stdin = SP1Stdin::new();
    stdin.write(&moves);
    stdin.write(&time);
    stdin.write(&matched_pairs);
    
    // Create ProverClient
    let client = ProverClient::from_env();
    
    // Execute program without proof
    println!("Executing program...");
    let (mut public_values, report) = client.execute(elf, &stdin).run().unwrap();
    println!("Program executed with {} cycles", report.total_instruction_count());
    
    // Read values
    let stored_moves = public_values.read::<u32>();
    let stored_time = public_values.read::<u32>();
    let stored_matched_pairs = public_values.read::<u32>();
    let final_score = public_values.read::<u32>();
    let is_complete = public_values.read::<bool>();
    
    // Print results
    println!("Execution results:");
    println!("- Moves: {}", stored_moves);
    println!("- Time: {}", stored_time);
    println!("- Matched Pairs: {}", stored_matched_pairs);
    println!("- Score (Remaining Time - Moves): {}", final_score);
    println!("- Game Complete: {}", is_complete);
    
    // Add result values to output
    let time_limit = 120u32;
    let remaining_time = if time < time_limit { time_limit - time } else { 0 };
    
    println!("TIME_LIMIT={}, REMAINING_TIME={}", time_limit, remaining_time);
    println!("FINAL_SCORE={}", final_score);
    
    // Generate proof
    println!("\nGenerating proof (this may take a while)...");
    let (pk, vk) = client.setup(elf);
    let proof = client.prove(&pk, &stdin).run().unwrap();
    
    println!("Proof generated successfully!");
    
    // Verify proof
    println!("Verifying proof...");
    client.verify(&proof, &vk).expect("Verification failed");
    
    println!("Proof verified successfully!");
    
    // Save proof
    let proof_path = "memory_game_proof.bin";
    proof.save(proof_path).expect("Failed to save proof");
    println!("Proof saved to: {}", proof_path);
    
    println!("\nSP1 ZK Proof generation complete!");
}