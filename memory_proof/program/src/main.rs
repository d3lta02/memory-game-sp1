#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

// Game data structure
#[derive(Serialize, Deserialize)]
struct GameData {
    moves: u32,
    time: u32,
    matched_pairs: u32,
}

pub fn main() {
    // Read input data
    let moves = sp1_zkvm::io::read::<u32>();
    let time = sp1_zkvm::io::read::<u32>();
    let matched_pairs = sp1_zkvm::io::read::<u32>();
    
    // Constants
    const TIME_LIMIT: u32 = 120;
    
    // Game validity check
    let is_complete = matched_pairs == 8;
    
    // Calculate score: Remaining Time - Moves
    let remaining_time = if time < TIME_LIMIT { TIME_LIMIT - time } else { 0 };
    let score = if is_complete { 
        remaining_time as i32 - moves as i32 
    } else { 
        0 
    };
    
    // Set negative scores to zero
    let final_score = if score < 0 { 0 } else { score as u32 };
    
    // Commit calculated values (verifiable outputs of the proof)
    sp1_zkvm::io::commit(&moves);
    sp1_zkvm::io::commit(&time);
    sp1_zkvm::io::commit(&matched_pairs);
    sp1_zkvm::io::commit(&final_score);
    sp1_zkvm::io::commit(&is_complete);
}