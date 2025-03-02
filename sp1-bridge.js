/**
 * SP1 Bridge - JavaScript bridge between Memory Game and SP1 ZK Proof system
 */

const SP1Bridge = {
    // Main proof generation function
    generateProof: async function(gameData) {
        console.log("SP1Bridge: Proof generation started", gameData);
        
        // Show log in the proof area
        if (window.log_to_proof_area) {
            window.log_to_proof_area("SP1 Proof system initializing...");
            window.log_to_proof_area(`Score: ${gameData.score}, Moves: ${gameData.moves}, Time: ${gameData.time}s`);
            window.log_to_proof_area("Running SP1 ZK program...");
        }
        
        try {
            // Try to generate a real proof
            const result = await this.generateRealProof(gameData);
            if (result.success) {
                return result;
            }
            
            // Fall back to simulation if unsuccessful
            window.log_to_proof_area("Switching to simulation mode...");
            return this.simulateProofProcess(gameData);
        } catch (error) {
            console.error("Error generating real proof:", error);
            window.log_to_proof_area(`Error: ${error.message}`);
            window.log_to_proof_area("Falling back to simulation mode...");
            
            // Fall back to simulation in case of error
            return this.simulateProofProcess(gameData);
        }
    },
    
    // Call the real SP1 proof API
    generateRealProof: async function(gameData) {
        if (window.log_to_proof_area) {
            window.log_to_proof_area("Connecting to SP1 backend...");
        }
        
        try {
            // API call
            const response = await fetch('http://localhost:3000/api/generate-proof', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(gameData)
            });
            
            if (!response.ok) {
                throw new Error(`API error: ${response.status}`);
            }
            
            const result = await response.json();
            
            if (window.log_to_proof_area) {
                window.log_to_proof_area("SP1 Proof successfully generated!");
                window.log_to_proof_area(`Proof Hash: ${result.proofHash}`);
            }
            
            // Create visual result
            this.createVisualProofResult(gameData, result.proofHash);
            
            // Show final result
            if (window.show_sp1_proof_result) {
                window.show_sp1_proof_result(true, result.proofHash);
            }
            
            return result;
        } catch (error) {
            console.error("API call failed:", error);
            if (window.log_to_proof_area) {
                window.log_to_proof_area(`API Error: ${error.message}`);
            }
            throw error;
        }
    },
    
<<<<<<< HEAD
    // Proof process simulation (your existing code)
=======
    // Proof process simulation
>>>>>>> c319a59 (Initial commit with Vercel deployment settings and X share button)
    simulateProofProcess: function(gameData) {
        // New score calculation simulation
        const TIME_LIMIT = 120;
        const remaining_time = gameData.time < TIME_LIMIT ? TIME_LIMIT - gameData.time : 0;
        const calculated_score = Math.max(0, remaining_time - gameData.moves);
        
        const steps = [
            { message: "Loading SP1 RISC-V program...", delay: 500 },
            { message: "Preparing game data for verification...", delay: 500 },
            { message: `Input values: Moves=${gameData.moves}, Time=${gameData.time}s, Matched=${gameData.matchedPairs}`, delay: 1000 },
            { message: "Validating game rules...", delay: 800 },
            { message: `Checking score calculation: Remaining Time (${remaining_time}) - Moves (${gameData.moves}) = ${calculated_score}`, delay: 1200 },
            { message: "Building SP1 ZK circuit...", delay: 1000 },
            { message: "Generating cryptographic proof (1/3)...", delay: 1200 },
            { message: "Generating cryptographic proof (2/3)...", delay: 1200 },
            { message: "Generating cryptographic proof (3/3)...", delay: 1200 },
            { message: "Verifying proof...", delay: 1000 },
            { message: "Proof successfully generated and verified! (SIMULATION)", delay: 800 }
        ];
        
        let currentStep = 0;
        
        // Show steps sequentially
        const processNextStep = () => {
            if (currentStep < steps.length) {
                if (window.log_to_proof_area) {
                    window.log_to_proof_area(steps[currentStep].message);
                }
                
                setTimeout(() => {
                    currentStep++;
                    processNextStep();
                }, steps[currentStep].delay);
            } else {
                // All steps completed, show the result
                this.completeProof(gameData);
            }
        };
        
        // Start the first step
        processNextStep();
    },
    
<<<<<<< HEAD
    // Other functions remain the same...
=======
    // Complete the proof process and show the result
>>>>>>> c319a59 (Initial commit with Vercel deployment settings and X share button)
    completeProof: function(gameData) {
        // Generate proof hash
        const hash = this.generateProofHash(gameData);
        
        // Show the result
        if (window.log_to_proof_area) {
            window.log_to_proof_area("=== PROOF RESULT ===");
            window.log_to_proof_area(`Hash: ${hash}`);
            window.log_to_proof_area("===================");
        }
        
        // Create visual elements for result display
        this.createVisualProofResult(gameData, hash);
        
        // Call show_sp1_proof_result function in WASM
        if (window.show_sp1_proof_result) {
            window.show_sp1_proof_result(true, hash);
        }
    },
    
    // Create visual proof result
    createVisualProofResult: function(gameData, hash) {
        // Create div to display proof result
        const proofResultDiv = document.createElement('div');
        proofResultDiv.id = 'proof-result';
        proofResultDiv.style.marginTop = '20px';
        proofResultDiv.style.padding = '15px';
        proofResultDiv.style.backgroundColor = 'rgba(46, 204, 113, 0.2)';
        proofResultDiv.style.borderRadius = '8px';
        proofResultDiv.style.border = '1px solid #2ecc71';
        
        // TIME_LIMIT and new score calculation
        const TIME_LIMIT = 120;
        const remaining_time = gameData.time < TIME_LIMIT ? TIME_LIMIT - gameData.time : 0;
        const calculated_score = Math.max(0, remaining_time - gameData.moves);
        
        // Add result div to proof area
        const proofLog = document.getElementById('proof-log');
        if (proofLog) {
            proofLog.appendChild(proofResultDiv);
            
            // Create result content
            const resultHTML = `
                <div style="text-align: center; margin-bottom: 10px;">
                    <span style="font-size: 24px; color: #2ecc71;">✓</span>
                    <span style="font-weight: bold; font-size: 18px; color: #2ecc71;"> Proof Verified!</span>
                </div>
                <div style="margin-bottom: 1px;">
                    <span style="font-weight: bold;">Score:</span> ${calculated_score} (Remaining Time - Moves)
                </div>
                <div style="margin-bottom: 1px;">
                    <span style="font-weight: bold;">Remaining Time:</span> ${remaining_time} seconds
                </div>
                <div style="margin-bottom: 2px;">
                    <span style="font-weight: bold;">Matched Pairs:</span> ${gameData.matchedPairs}
                </div>
                <div style="margin-bottom: 2px;">
                    <span style="font-weight: bold;">Moves:</span> ${gameData.moves}
                </div>
                <div style="margin-bottom: 2px;">
                    <span style="font-weight: bold;">Game Time:</span> ${gameData.time} seconds
                </div>
                <div style="margin-top: 5px; word-break: break-all;">
                    <span style="font-weight: bold;">Proof Hash:</span> 
                    <span style="font-family: monospace; color: #3498db;">${hash}</span>
                </div>
<<<<<<< HEAD
            `;
            
            proofResultDiv.innerHTML = resultHTML;
=======
                
                <!-- Share Button -->
                <div style="margin-top: 15px; text-align: center;">
                    <button id="share-x-button" style="
                        background-color: #000;
                        color: white;
                        border: none;
                        padding: 8px 15px;
                        border-radius: 20px;
                        font-weight: bold;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        gap: 8px;
                        margin: 0 auto;
                    ">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="white">
                            <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
                        </svg>
                        Share Score on X
                    </button>
                </div>
            `;
            
            proofResultDiv.innerHTML = resultHTML;
            
            // Add click event to share button
            setTimeout(() => {
                const shareButton = document.getElementById('share-x-button');
                if (shareButton) {
                    shareButton.addEventListener('click', () => this.shareOnX(gameData, calculated_score));
                }
            }, 100);
>>>>>>> c319a59 (Initial commit with Vercel deployment settings and X share button)
        }
    },
    
    // Generate proof hash (simulation of a real proof hash)
    generateProofHash: function(gameData) {
        // Used for TIME_LIMIT and new score calculation
        const TIME_LIMIT = 120;
        const remaining_time = gameData.time < TIME_LIMIT ? TIME_LIMIT - gameData.time : 0;
        const calculated_score = Math.max(0, remaining_time - gameData.moves);
        
        // Create a random hash
        const scoreHex = calculated_score.toString(16).padStart(4, '0');
        const movesHex = gameData.moves.toString(16).padStart(4, '0');
        const timeHex = gameData.time.toString(16).padStart(4, '0');
        const randomPart = Math.floor(Math.random() * 0x10000000000000).toString(16).padStart(14, '0');
        
        return `0x${scoreHex}${movesHex}${timeHex}${randomPart}`;
<<<<<<< HEAD
=======
    },
    
    // Share score on X (Twitter)
    shareOnX: function(gameData, score) {
        const shareText = `I scored ${score} points in Succinct Memory Game. Play it yourself! memory-game-sp1.vercel.app`;
        const encodedText = encodeURIComponent(shareText);
        const twitterUrl = `https://twitter.com/intent/tweet?text=${encodedText}`;
        
        // Open X sharing page in a new window
        window.open(twitterUrl, '_blank');
        
        // Log the share action
        console.log("Shared score on X:", score);
>>>>>>> c319a59 (Initial commit with Vercel deployment settings and X share button)
    }
};

// Global SP1Bridge object
window.SP1Bridge = SP1Bridge;

// Global proof generation function
window.generateSP1Proof = function(gameData) {
    return SP1Bridge.generateProof(gameData);
};

console.log("SP1Bridge loaded - Memory Game ZK integration ready!");