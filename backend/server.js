const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const app = express();
const PORT = 3000;

// Middleware
app.use(cors());
app.use(bodyParser.json());

// SP1 proof generation endpoint
app.post('/api/generate-proof', async (req, res) => {
    try {
        const gameData = req.body;
        console.log('Received game data:', gameData);

        // TIME_LIMIT and score calculation
        const TIME_LIMIT = 120;
        const remainingTime = gameData.time < TIME_LIMIT ? TIME_LIMIT - gameData.time : 0;
        const calculatedScore = Math.max(0, remainingTime - gameData.moves);
        
        console.log(`Calculated score: ${calculatedScore}`);
        
        // Run SP1 script with real game data
        console.log("Running SP1 proof generator with game data...");
        
        const scriptPath = path.join(__dirname, '..', 'memory_proof', 'script');
        const command = `cd "${scriptPath}" && cargo run --bin memory_prove --release -- ${gameData.moves} ${gameData.time} ${gameData.matchedPairs}`;
        
        exec(command, (error, stdout, stderr) => {
            console.log("SP1 proof output:", stdout);
            if (stderr) console.error("SP1 proof errors:", stderr);
            
            // Check if proof was successful
            let isRealProof = false;
            let finalScore = calculatedScore;
            
            if (!error && stdout.includes("Proof verified successfully")) {
                isRealProof = true;
                console.log("Real SP1 proof generated and verified!");
                
                // Extract actual score from SP1 output
                const scoreMatch = stdout.match(/FINAL_SCORE=(\d+)/);
                if (scoreMatch && scoreMatch[1]) {
                    finalScore = parseInt(scoreMatch[1]);
                    console.log(`Using SP1 verified score: ${finalScore}`);
                }
            } else {
                console.log("Using simulation mode, SP1 proof generation failed or incomplete");
            }
            
            // Generate proof hash
            const scoreHex = finalScore.toString(16).padStart(4, '0');
            const movesHex = gameData.moves.toString(16).padStart(4, '0');
            const timeHex = gameData.time.toString(16).padStart(4, '0');
            
            // Proof hash - real or simulated
            let proofHash;
            if (isRealProof) {
                // Special hash format for real proof
                proofHash = `0xSP1_${scoreHex}_${movesHex}_${timeHex}_REAL`;
            } else {
                // Simulated proof
                const randomPart = crypto.randomBytes(4).toString('hex');
                proofHash = `0xSIM_${scoreHex}_${movesHex}_${timeHex}_${randomPart}`;
            }
            
            // Prepare response
            const response = {
                success: true,
                proofHash: proofHash,
                calculatedScore: finalScore,
                isRealProof: isRealProof,
                gameData: {
                    moves: gameData.moves,
                    time: gameData.time,
                    matchedPairs: gameData.matchedPairs
                },
                remainingTime: remainingTime,
                proofDetails: {
                    algorithm: "SP1 ZK-STARK",
                    verificationMethod: isRealProof ? "Real SP1 RISC-V zkVM" : "Simulation",
                    scoreFormula: "Remaining Time - Moves",
                    createdAt: new Date().toISOString()
                }
            };
            
            // Log real proof success
            if (isRealProof) {
                console.log("✅ REAL SP1 ZERO-KNOWLEDGE PROOF GENERATED AND VERIFIED!");
                console.log(`Game score ${finalScore} cryptographically proven with ZK-STARK technology`);
            } else {
                console.log("⚠️ Using simulation mode - Real proof failed");
            }
            
            // Send response
            res.json(response);
        });
    } catch (error) {
        console.error('Error in proof generation process:', error);
        res.status(500).json({
            success: false,
            error: error.message
        });
    }
});

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({
        status: 'ok',
        timestamp: new Date().toISOString()
    });
});

// Start server
app.listen(PORT, () => {
    console.log(`SP1 API Server running on http://localhost:${PORT}`);
    console.log(`Generate real ZK proofs with the "Prove (SP1)" button in the web interface!`);
    console.log(`This server proves game scores using the Remaining Time - Moves formula`);
});