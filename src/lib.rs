// lib.rs
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlElement};
use js_sys::Math;
use wasm_bindgen::JsCast;
use std::cell::RefCell;

// Constants
const TIME_LIMIT: usize = 120; // 120 seconds time limit

// Asset paths
const IMAGE_PATH: &str = "assets/images/";
const SOUND_PATH: &str = "assets/sounds/"; // For sound files

// Game state
struct GameState {
    cards: Vec<usize>,
    flipped_cards: Vec<usize>,
    matched_pairs: Vec<usize>,
    moves: usize,
    timer: usize,
    score: usize,
    game_started: bool,
    game_over: bool,
    timer_interval_id: Option<i32>,
    is_checking: bool,
}

// Sound settings
struct SoundSettings {
    enabled: bool,
}

// Global RefCell to store the game state
thread_local! {
    static GAME_STATE: RefCell<GameState> = RefCell::new(GameState {
        cards: Vec::new(),
        flipped_cards: Vec::new(),
        matched_pairs: Vec::new(),
        moves: 0,
        timer: 0,
        score: 0,
        game_started: false,
        game_over: false,
        timer_interval_id: None,
        is_checking: false,
    });
    
    // Thread-local variable for sound settings
    static SOUND_SETTINGS: RefCell<SoundSettings> = RefCell::new(SoundSettings {
        enabled: true,
    });
}

// Sound playing function
#[wasm_bindgen]
pub fn play_sound(sound_name: &str) {
    // Don't play if sound is disabled
    let enabled = SOUND_SETTINGS.with(|settings| {
        settings.borrow().enabled
    });
    
    if !enabled {
        return;
    }
    
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // Create audio element
    if let Ok(audio_element) = document.create_element("audio") {
        audio_element.set_attribute("src", &format!("{}{}", SOUND_PATH, sound_name)).ok();
        audio_element.set_attribute("autoplay", "true").ok();
        
        // Clone the audio element for the closure
        let audio_element_clone = audio_element.clone();
        
        // Remove from DOM when playback is complete
        let cleanup_closure = Closure::wrap(Box::new(move || {
            if let Some(parent) = audio_element_clone.parent_node() {
                parent.remove_child(&audio_element_clone).ok();
            }
        }) as Box<dyn FnMut()>);
        
        audio_element
            .dyn_ref::<web_sys::HtmlAudioElement>()
            .expect("Not an HtmlAudioElement")
            .set_onended(Some(cleanup_closure.as_ref().unchecked_ref()));
        
        cleanup_closure.forget();
        
        // Add audio element to the body
        document.body().unwrap().append_child(&audio_element).ok();
    }
}

// Toggle sound settings
#[wasm_bindgen]
pub fn toggle_sound() -> bool {
    SOUND_SETTINGS.with(|settings| {
        let mut sound_settings = settings.borrow_mut();
        sound_settings.enabled = !sound_settings.enabled;
        sound_settings.enabled
    })
}

#[wasm_bindgen]
pub fn initialize_game() -> Result<(), JsValue> {
    // Get DOM elements
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // Create the game board
    create_game_board(&document)?;
    
    // Create cards but make them unclickable before the game starts
    prepare_cards(&document);
    
    // Notify about asset folders
    web_sys::console::log_1(&"Asset folders need to be created:".into());
    web_sys::console::log_1(&format!("- {} (For card images)", IMAGE_PATH).into());
    web_sys::console::log_1(&format!("- {} (For sound effects)", SOUND_PATH).into());
    
    Ok(())
}

fn prepare_cards(document: &Document) {
    // Create 8 pairs of cards (0-7, each one twice)
    let mut cards: Vec<usize> = (0..8).flat_map(|i| vec![i, i]).collect();
    
    // Shuffle the cards
    for i in (1..cards.len()).rev() {
        let j = (Math::random() * (i as f64 + 1.0)) as usize;
        cards.swap(i, j);
    }
    
    // Update game state
    GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        game_state.cards = cards;
        game_state.game_started = false; // Game hasn't started yet
        game_state.game_over = false;
    });
    
    // Create cards visually
    render_game_board(document);
}

fn start_game() {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // Start the game
    GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        
        // Reset the game
        game_state.flipped_cards.clear();
        game_state.matched_pairs.clear();
        game_state.moves = 0;
        game_state.timer = 0;
        game_state.score = 0;
        game_state.is_checking = false;
        game_state.game_over = false;
        
        // Start a new game
        game_state.game_started = true;
    });
    
    // Start the timer
    setup_timer();
    
    // Update the UI
    update_game_stats(&document);
    
    // Disable the start button
    if let Some(start_button) = document.get_element_by_id("start-game") {
        start_button.set_attribute("disabled", "true").ok();
        start_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #95a5a6; color: white; border: none; border-radius: 5px; cursor: not-allowed;").ok();
    }
    
    // Disable the prove button
    if let Some(prove_button) = document.get_element_by_id("prove-game") {
        prove_button.set_attribute("disabled", "true").ok();
        prove_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #3498db; color: white; border: none; border-radius: 5px; cursor: not-allowed; opacity: 0.6;").ok();
    }
    
    // Update card visuals (based on game started status)
    update_card_visuals(&document);
    
    // Show notification
    window.alert_with_message(&format!("Game started! Try to match all cards within {} seconds. Good luck!", TIME_LIMIT)).ok();
}

fn setup_timer() {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        
        // If a timer is already running, clear it
        if let Some(interval_id) = game_state.timer_interval_id {
            window.clear_interval_with_handle(interval_id);
            game_state.timer_interval_id = None;
        }
        
        // Start a new timer
        let timer_callback = Closure::wrap(Box::new(move || {
            let window = web_sys::window().expect("No global window");
            let document = window.document().expect("No global document");
            
            let (game_active, time_up) = GAME_STATE.with(|state| {
                let mut game_state = state.borrow_mut();
                if game_state.game_started && !game_state.game_over {
                    game_state.timer += 1;
                    
                    // Check if time is up
                    if game_state.timer >= TIME_LIMIT {
                        game_state.game_over = true;
                        return (false, true);
                    }
                    
                    return (true, false);
                }
                (false, false)
            });
            
            if game_active {
                // Update the timer
                if let Some(timer_element) = document.get_element_by_id("timer") {
                    let (timer, time_remaining) = GAME_STATE.with(|state| {
                        let game_state = state.borrow();
                        (game_state.timer, TIME_LIMIT - game_state.timer)
                    });
                    
                    // Change color based on remaining time
                    let timer_color = if time_remaining <= 10 {
                        "color: #e74c3c;" // Red (almost out of time)
                    } else if time_remaining <= 30 {
                        "color: #f39c12;" // Orange (warning)
                    } else {
                        "color: white;" // Normal
                    };
                    
                    timer_element.set_attribute("style", &format!("font-size: 24px; {}", timer_color)).ok();
                    timer_element.set_text_content(Some(&format!("Time: {} sec (Remaining: {})", timer, time_remaining)));
                }
            }
            
            // If time is up, end the game
            if time_up {
                end_game(false); // Lost due to time running out
            }
            
        }) as Box<dyn FnMut()>);
        
        let interval_id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                timer_callback.as_ref().unchecked_ref(),
                1000,
            )
            .expect("Could not create interval");
        
        game_state.timer_interval_id = Some(interval_id);
        timer_callback.forget();
    });
}

fn create_game_board(document: &Document) -> Result<(), JsValue> {
    // Create game area container
    let container = document.create_element("div")?;
    container.set_id("game-container");
    container.set_attribute("style", &format!("width: 1024px; height: 768px; background-image: url('{}background.gif'); background-size: cover; position: relative; margin: 0 auto;", IMAGE_PATH))?;
    
    // Top area - Score and timer
    let header = document.create_element("div")?;
    header.set_id("game-header");
    header.set_attribute("style", "height: 150px; display: flex; justify-content: space-between; align-items: center; padding: 0 50px;")?;
    
    let timer = document.create_element("div")?;
    timer.set_id("timer");
    timer.set_attribute("style", "font-size: 24px; color: white;")?;
    timer.set_text_content(Some(&format!("Time: 0 sec (Remaining: {})", TIME_LIMIT)));
    
    let moves = document.create_element("div")?;
    moves.set_id("moves");
    moves.set_attribute("style", "font-size: 24px; color: white;")?;
    moves.set_text_content(Some("Moves: 0"));
    
    let score = document.create_element("div")?;
    score.set_id("score");
    score.set_attribute("style", "font-size: 24px; color: white;")?;
    score.set_text_content(Some("Score: 0"));
    
    header.append_child(&timer)?;
    header.append_child(&moves)?;
    header.append_child(&score)?;
    
    // Middle area - Game cards
    let board = document.create_element("div")?;
    board.set_id("game-board");
    board.set_attribute("style", "height: 480px; display: flex; flex-wrap: wrap; justify-content: center; align-items: center; gap: 20px; padding: 20px;")?;
    
    // Bottom area - Control buttons
    let footer = document.create_element("div")?;
    footer.set_id("game-footer");
    footer.set_attribute("style", "height: 138px; display: flex; justify-content: center; align-items: center; gap: 30px;")?;
    
    let start_button = document.create_element("button")?;
    start_button.set_id("start-game");
    start_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #2ecc71; color: white; border: none; border-radius: 5px; cursor: pointer;")?;
    start_button.set_text_content(Some("Start Game"));
    
    // Add click event to start button
    let start_closure = Closure::wrap(Box::new(move || {
        start_game();
        play_sound("game-start.mp3");
    }) as Box<dyn FnMut()>);
    
    start_button
        .dyn_ref::<HtmlElement>()
        .expect("Not an HtmlElement")
        .set_onclick(Some(start_closure.as_ref().unchecked_ref()));
    
    start_closure.forget();
    
    let reset_button = document.create_element("button")?;
    reset_button.set_id("reset-game");
    reset_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #e74c3c; color: white; border: none; border-radius: 5px; cursor: pointer;")?;
    reset_button.set_text_content(Some("Reset Game"));
    
    // Add click event to reset button
    let reset_closure = Closure::wrap(Box::new(move || {
        reset_game();
        play_sound("button-click.mp3");
    }) as Box<dyn FnMut()>);
    
    reset_button
        .dyn_ref::<HtmlElement>()
        .expect("Not an HtmlElement")
        .set_onclick(Some(reset_closure.as_ref().unchecked_ref()));
    
    reset_closure.forget();
    
    // Prove button
    let prove_button = document.create_element("button")?;
    prove_button.set_id("prove-game");
    prove_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #3498db; color: white; border: none; border-radius: 5px; cursor: not-allowed; opacity: 0.6;")?;
    prove_button.set_text_content(Some("Prove (SP1)"));
    prove_button.set_attribute("disabled", "true")?;
    
    // Add click event to prove button
    let prove_closure = Closure::wrap(Box::new(move || {
        start_sp1_proof();
        play_sound("button-click.mp3");
    }) as Box<dyn FnMut()>);
    
    prove_button
        .dyn_ref::<HtmlElement>()
        .expect("Not an HtmlElement")
        .set_onclick(Some(prove_closure.as_ref().unchecked_ref()));
    
    prove_closure.forget();
    
    footer.append_child(&start_button)?;
    footer.append_child(&reset_button)?;
    footer.append_child(&prove_button)?;
    
    // Add all areas to the main container
    container.append_child(&header)?;
    container.append_child(&board)?;
    container.append_child(&footer)?;
    
    // Add container to the body
    document.body().unwrap().append_child(&container)?;
    
    Ok(())
}

// Reset the game
fn reset_game() {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // Stop the timer
    GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        if let Some(interval_id) = game_state.timer_interval_id {
            window.clear_interval_with_handle(interval_id);
            game_state.timer_interval_id = None;
        }
        game_state.game_started = false;
        game_state.game_over = false;
    });
    
    // Enable the start button
    if let Some(start_button) = document.get_element_by_id("start-game") {
        start_button.remove_attribute("disabled").ok();
        start_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #2ecc71; color: white; border: none; border-radius: 5px; cursor: pointer;").ok();
    }
    
    // Reset the timer color
    if let Some(timer_element) = document.get_element_by_id("timer") {
        timer_element.set_attribute("style", "font-size: 24px; color: white;").ok();
    }
    
    // Disable the prove button
    if let Some(prove_button) = document.get_element_by_id("prove-game") {
        prove_button.set_attribute("disabled", "true").ok();
        prove_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #3498db; color: white; border: none; border-radius: 5px; cursor: not-allowed; opacity: 0.6;").ok();
    }
    
    // Re-prepare the cards
    prepare_cards(&document);
    
    // Reset statistics
    GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        game_state.moves = 0;
        game_state.timer = 0;
        game_state.score = 0;
        game_state.flipped_cards.clear();
        game_state.matched_pairs.clear();
        game_state.is_checking = false;
    });
    
    update_game_stats(&document);
}

fn render_game_board(document: &Document) {
    if let Some(board) = document.get_element_by_id("game-board") {
        // First, clear existing cards
        while let Some(child) = board.first_child() {
            board.remove_child(&child).expect("Failed to remove child element");
        }
        
        // Create cards - defining variables outside GAME_STATE
        let cards: Vec<usize> = GAME_STATE.with(|state| {
            let game_state = state.borrow();
            game_state.cards.clone()
        });
        
        for (index, &card_value) in cards.iter().enumerate() {
            let card_element = document.create_element("div").expect("Failed to create element");
            card_element.set_attribute("data-index", &index.to_string()).expect("Failed to set attribute");
            card_element.set_attribute("data-value", &card_value.to_string()).expect("Failed to set attribute");
            
            // Set card style (back face)
            card_element.set_attribute(
                "style", 
                &format!("width: 120px; height: 120px; background-image: url('{}card-back.png'); background-size: cover; cursor: pointer; transform-style: preserve-3d; transition: transform 0.5s; border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);", IMAGE_PATH)
            ).expect("Failed to set style");
            
            // Add click event to the card
            let click_index = index.clone();
            let closure = Closure::wrap(Box::new(move || {
                card_click(click_index);
            }) as Box<dyn FnMut()>);
            
            card_element
                .dyn_ref::<HtmlElement>()
                .expect("Not an HtmlElement")
                .set_onclick(Some(closure.as_ref().unchecked_ref()));
            
            closure.forget();
            
            board.append_child(&card_element).expect("Failed to add card");
        }
    }
}

fn card_click(index: usize) {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // If the game hasn't started, has ended, or another operation is in progress, don't process the click
    let (game_started, game_over, is_checking, card_already_handled) = GAME_STATE.with(|state| {
        let game_state = state.borrow();
        
        // Card is already flipped or matched
        let card_value = game_state.cards[index];
        let already_handled = game_state.flipped_cards.contains(&index) || 
                              game_state.matched_pairs.contains(&card_value);
                              
        (game_state.game_started, game_state.game_over, game_state.is_checking, already_handled)
    });
    
    if !game_started || game_over || is_checking || card_already_handled {
        return;
    }
    
    // Flip the card and handle game logic
    let should_check = GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        
        // Flip the card
        game_state.flipped_cards.push(index);
        
        // If two cards are flipped, return true for checking
        game_state.flipped_cards.len() == 2
    });
    
    // Card flip sound
    play_sound("card-flip.mp3");
    
    // Update card visuals
    update_card_visuals(&document);
    
    // If two cards are flipped, check for a match
    if should_check {
        // Update game state
        GAME_STATE.with(|state| {
            let mut game_state = state.borrow_mut();
            game_state.moves += 1;
            game_state.is_checking = true;
        });
        
        // Update statistics
        update_game_stats(&document);
        
        // Check for a match (after 1000ms delay)
        let check_closure = Closure::wrap(Box::new(move || {
            check_match();
        }) as Box<dyn FnMut()>);
        
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            check_closure.as_ref().unchecked_ref(),
            1000 // 1000ms delay (increased for better visibility)
        ).expect("setTimeout failed");
        
        check_closure.forget();
    }
}

fn check_match() {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    let (is_match, game_completed) = GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        
        if game_state.flipped_cards.len() != 2 {
            game_state.is_checking = false;
            return (false, false);
        }
        
        let first_card_index = game_state.flipped_cards[0];
        let second_card_index = game_state.flipped_cards[1];
        
        let first_card_value = game_state.cards[first_card_index];
        let second_card_value = game_state.cards[second_card_index];
        
        // Do the cards match?
        if first_card_value == second_card_value {
            // Save the matched pair
            game_state.matched_pairs.push(first_card_value);
            
            
            // Check if all cards are matched
            let all_matched = game_state.matched_pairs.len() == 8; // 8 pairs of cards
            
            // Clear flipped cards
            game_state.flipped_cards.clear();
            game_state.is_checking = false;
            
            (true, all_matched)
        } else {
            // No match, flip cards back
            game_state.flipped_cards.clear();
            game_state.is_checking = false;
            
            (false, false)
        }
    });
    
    // Update card visuals
    update_card_visuals(&document);
    
    // Update statistics (score may have changed)
    update_game_stats(&document);
    
    // Play match sound
    if is_match {
        play_sound("match.mp3");
    } else {
        play_sound("no-match.mp3");
    }
    
    // If game completed
    if game_completed {
        end_game(true); // Won
    }
}

fn update_card_visuals(document: &Document) {
    // Get required data from GAME_STATE
    let (cards, flipped_cards, matched_pairs, game_started, game_over) = GAME_STATE.with(|state| {
        let game_state = state.borrow();
        (
            game_state.cards.clone(),
            game_state.flipped_cards.clone(),
            game_state.matched_pairs.clone(),
            game_state.game_started,
            game_state.game_over
        )
    });
    
    // Loop through all cards
    for index in 0..cards.len() {
        if let Some(card_element) = document.query_selector(&format!("[data-index=\"{}\"]", index)).expect("Query failed") {
            let card_value = cards[index];
            
            // Check if card is flipped or matched
            if matched_pairs.contains(&card_value) {
                // Matched card - faded look
                card_element.set_attribute(
                    "style", 
                    &format!("width: 120px; height: 120px; background-image: url('{}card-{}.png'); background-size: cover; cursor: default; opacity: 0.7; transform: rotateY(0deg); border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);", IMAGE_PATH, card_value)
                ).expect("Failed to set style");
            } else if flipped_cards.contains(&index) {
                // Flipped but not yet matched card
                card_element.set_attribute(
                    "style", 
                    &format!("width: 120px; height: 120px; background-image: url('{}card-{}.png'); background-size: cover; cursor: pointer; transform: rotateY(0deg); border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,0.2);", IMAGE_PATH, card_value)
                ).expect("Failed to set style");
            } else {
                // Face down card
                let cursor_style = if game_started && !game_over { "cursor: pointer;" } else { "cursor: not-allowed;" };
                card_element.set_attribute(
                    "style", 
                    &format!("width: 120px; height: 120px; background-image: url('{}card-back.png'); background-size: cover; {} transform: rotateY(0deg); border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);", IMAGE_PATH, cursor_style)
                ).expect("Failed to set style");
            }
        }
    }
}

fn update_game_stats(document: &Document) {
    let (moves, timer, score, time_remaining) = GAME_STATE.with(|state| {
        let game_state = state.borrow();
        (
            game_state.moves, 
            game_state.timer, 
            game_state.score,
            if game_state.timer < TIME_LIMIT { TIME_LIMIT - game_state.timer } else { 0 }
        )
    });
    
    // Update move count
    if let Some(moves_element) = document.get_element_by_id("moves") {
        moves_element.set_text_content(Some(&format!("Moves: {}", moves)));
    }
    
    // Update timer
    if let Some(timer_element) = document.get_element_by_id("timer") {
        timer_element.set_text_content(Some(&format!("Time: {} sec (Remaining: {})", timer, time_remaining)));
    }
    
    // Update score
    if let Some(score_element) = document.get_element_by_id("score") {
        score_element.set_text_content(Some(&format!("Score: {}", score)));
    }
}

// End the game (won or time's up)
fn end_game(is_winner: bool) {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // Stop the timer
    GAME_STATE.with(|state| {
        let mut game_state = state.borrow_mut();
        if let Some(interval_id) = game_state.timer_interval_id {
            window.clear_interval_with_handle(interval_id);
            game_state.timer_interval_id = None;
        }
        game_state.game_started = false;
        game_state.game_over = true;
    });
    
// Get statistics
let (moves, timer, matched_pairs) = GAME_STATE.with(|state| {
    let game_state = state.borrow();
    (game_state.moves, game_state.timer, game_state.matched_pairs.len())
});

// Yeni puanlama sistemi: Kalan Zaman - Hamle Sayısı
let remaining_time = if timer < TIME_LIMIT { TIME_LIMIT - timer } else { 0 };
let score = if is_winner { 
    // Kazandıysa: Kalan Zaman - Hamle Sayısı (negatif olabilir)
    remaining_time as i32 - moves as i32 
} else { 
    0 // Kaybettiyse 0 puan
};

// Skoru güncelle
GAME_STATE.with(|state| {
    let mut game_state = state.borrow_mut();
    game_state.score = if score < 0 { 0 } else { score as usize }; // Negatif puanları 0 olarak kabul et
});
    
// Create message
let message = if is_winner {
    format!(
        "Congratulations! You won the game!\nMoves: {}\nScore: {} (Remaining Time - Moves)\nTime: {} seconds",
        moves, score, timer
    )
} else {
    format!(
        "Time's up! Game over.\nMoves: {}\nScore: {}\nTime: {} seconds",
        moves, score, timer
    )
};
    
    // Show congratulations or notification message
    window.alert_with_message(&message).expect("Alert could not be shown");
    
    // Enable the start button
    if let Some(start_button) = document.get_element_by_id("start-game") {
        start_button.remove_attribute("disabled").ok();
        start_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #2ecc71; color: white; border: none; border-radius: 5px; cursor: pointer;").ok();
    }
    
    // Enable the prove button
    if let Some(prove_button) = document.get_element_by_id("prove-game") {
        prove_button.remove_attribute("disabled").ok();
        prove_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #3498db; color: white; border: none; border-radius: 5px; cursor: pointer; opacity: 1.0;").ok();
    }
    
    // Game ending sound
    if is_winner {
        play_sound("success.mp3");
    } else {
        play_sound("lose.mp3");
    }
}

// Show proof in game area
fn show_proof_in_game_area(document: &Document) -> Result<(), JsValue> {
    // Get game board element
    if let Some(board) = document.get_element_by_id("game-board") {
        // First, clear existing content
        while let Some(child) = board.first_child() {
            board.remove_child(&child)?;
        }
        
        // Create proof panel container
        let proof_container = document.create_element("div")?;
        proof_container.set_id("proof-container");
        proof_container.set_attribute("style", "
            width: 100%;
            height: 100%;
            background-color: rgba(0, 0, 0, 0.85);
            color: #2ecc71;
            font-family: monospace;
            border-radius: 8px;
            padding: 20px;
            display: flex;
            flex-direction: column;
        ")?;
        
        // Create proof header
        let proof_header = document.create_element("div")?;
        proof_header.set_id("proof-header");
        proof_header.set_text_content(Some("SP1 Zero Knowledge Proof Process"));
        proof_header.set_attribute("style", "
            font-size: 24px;
            font-weight: bold;
            text-align: center;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 1px solid #2ecc71;
        ")?;
        
        // Create proof log area
        let proof_log = document.create_element("div")?;
        proof_log.set_id("proof-log");
        proof_log.set_attribute("style", "
            flex: 1;
            overflow-y: auto;
            font-size: 14px;
            padding: 10px;
            background-color: rgba(0, 0, 0, 0.5);
            border-radius: 4px;
            white-space: pre-wrap;
            line-height: 1.5;
        ")?;
        
        // Create proof buttons area
        let proof_buttons = document.create_element("div")?;
        proof_buttons.set_id("proof-buttons");
        proof_buttons.set_attribute("style", "
            margin-top: 20px;
            display: flex;
            justify-content: center;
            gap: 20px;
        ")?;
        
        // Back to Game button
        let back_button = document.create_element("button")?;
        back_button.set_id("back-to-game");
        back_button.set_text_content(Some("Back to Game"));
        back_button.set_attribute("style", "
            padding: 10px 20px;
            background-color: #3498db;
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
        ")?;
        
        // Add click event to button
        let back_closure = Closure::wrap(Box::new(move || {
            let window = web_sys::window().expect("No global window");
            let document = window.document().expect("No global document");
            render_game_board(&document);
            play_sound("button-click.mp3");
        }) as Box<dyn FnMut()>);
        
        back_button
            .dyn_ref::<HtmlElement>()
            .expect("Not an HtmlElement")
            .set_onclick(Some(back_closure.as_ref().unchecked_ref()));
        
        back_closure.forget();
        
        // Restart Game button
        let restart_button = document.create_element("button")?;
        restart_button.set_id("restart-game");
        restart_button.set_text_content(Some("Restart Game"));
        restart_button.set_attribute("style", "
            padding: 10px 20px;
            background-color: #e74c3c;
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
        ")?;
        
        // Add click event to button
        let restart_closure = Closure::wrap(Box::new(move || {
            reset_game();
            play_sound("button-click.mp3");
        }) as Box<dyn FnMut()>);
        
        restart_button
            .dyn_ref::<HtmlElement>()
            .expect("Not an HtmlElement")
            .set_onclick(Some(restart_closure.as_ref().unchecked_ref()));
        
        restart_closure.forget();
        
        // Add buttons
        proof_buttons.append_child(&back_button)?;
        proof_buttons.append_child(&restart_button)?;
        
        // Add elements to proof container
        proof_container.append_child(&proof_header)?;
        proof_container.append_child(&proof_log)?;
        proof_container.append_child(&proof_buttons)?;
        
        // Add proof container to game area
        board.append_child(&proof_container)?;
    }
    
    Ok(())
}

// Add log message to proof area
#[wasm_bindgen]
pub fn log_to_proof_area(message: &str) -> Result<(), JsValue> {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    if let Some(proof_log) = document.get_element_by_id("proof-log") {
        // Add timestamp
        let date = js_sys::Date::new_0();
        let timestamp = format!("[{:02}:{:02}:{:02}] ", 
            date.get_hours(), 
            date.get_minutes(), 
            date.get_seconds()
        );
        
        // Create new line
        let line = document.create_element("div")?;
        line.set_text_content(Some(&format!("{}{}", timestamp, message)));
        
        // Set line color
        if message.contains("error") || message.contains("failed") || message.contains("ERROR") {
            line.set_attribute("style", "color: #e74c3c;")?; // Red
        } else if message.contains("success") || message.contains("verified") || message.contains("SUCCESS") {
            line.set_attribute("style", "color: #2ecc71;")?; // Green
        } else if message.contains("generating") || message.contains("wait") {
            line.set_attribute("style", "color: #f39c12;")?; // Orange
        }
        
        // Add line to log area
        proof_log.append_child(&line)?;
        
        // Auto-scroll
        let _ = js_sys::eval(&format!("document.getElementById('proof-log').scrollTop = document.getElementById('proof-log').scrollHeight"));
    }
    
    Ok(())
}

// Start SP1 proof generation
fn start_sp1_proof() {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    play_sound("button-click.mp3");
    
    // Get game state
    let (moves, timer, score, matched_pairs, is_game_over) = GAME_STATE.with(|state| {
        let game_state = state.borrow();
        (
            game_state.moves,
            game_state.timer,
            game_state.score,
            game_state.matched_pairs.len(),
            game_state.game_over
        )
    });
    
    // If game is not over, show error
    if !is_game_over {
        window.alert_with_message("Game is not completed yet! You need to finish the game first.").ok();
        return;
    }
    
    // Disable the prove button
    if let Some(prove_button) = document.get_element_by_id("prove-game") {
        prove_button.set_attribute("disabled", "true").ok();
        prove_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #95a5a6; color: white; border: none; border-radius: 5px; cursor: not-allowed;").ok();
    }
    
    // Clear game area and add proof panel
    show_proof_in_game_area(&document).ok();
    
    // Show proof start message
    log_to_proof_area("Starting SP1 ZK Proof process...").ok();
    log_to_proof_area(&format!("Game Info: Score: {}, Moves: {}, Time: {}s, Matched Pairs: {}", 
        score, moves, timer, matched_pairs)).ok();
    
    // SP1 proof generation (delegated to JavaScript)
    // Communicate with JavaScript (sp1-bridge.js)
    let js_game_data = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&js_game_data, &"score".into(), &(score as u32).into());
    let _ = js_sys::Reflect::set(&js_game_data, &"moves".into(), &(moves as u32).into());
    let _ = js_sys::Reflect::set(&js_game_data, &"time".into(), &(timer as u32).into());
    let _ = js_sys::Reflect::set(&js_game_data, &"matchedPairs".into(), &(matched_pairs as u32).into());
    
    // Call generateProof function in JavaScript
    // Note: You need to include this JSBridge in index.html
    let _ = js_sys::eval(&format!("
        if (window.SP1Bridge && typeof window.SP1Bridge.generateProof === 'function') {{
            window.SP1Bridge.generateProof({});
        }} else {{
            document.getElementById('proof-log').innerHTML += '<div style=\"color: #e74c3c;\">[Error] SP1Bridge not found! Check sp1-bridge.js file.</div>';
        }}
    ", js_sys::JSON::stringify(&js_game_data).unwrap()));
}

// Show SP1 proof result
#[wasm_bindgen]
pub fn show_sp1_proof_result(success: bool, hash: &str) -> Result<(), JsValue> {
    let window = web_sys::window().expect("No global window");
    let document = window.document().expect("No global document");
    
    // Re-enable the prove button
    if let Some(prove_button) = document.get_element_by_id("prove-game") {
        prove_button.remove_attribute("disabled").ok();
        prove_button.set_attribute("style", "padding: 15px 30px; font-size: 20px; background-color: #3498db; color: white; border: none; border-radius: 5px; cursor: pointer; opacity: 1.0;").ok();
    }
    
    if success {
        // Success sound
        play_sound("success.mp3");
        
        log_to_proof_area("Proof verification SUCCESS!").ok();
        log_to_proof_area(&format!("Proof Hash: {}", hash)).ok();
        log_to_proof_area("=====================================").ok();
        log_to_proof_area("This proof verifies the validity of your game score.").ok();
        
        // Add success message to proof panel
        if let Some(proof_header) = document.get_element_by_id("proof-header") {
            proof_header.set_text_content(Some("✅ Proof Successfully Generated"));
            proof_header.set_attribute("style", "
                font-size: 24px;
                font-weight: bold;
                text-align: center;
                margin-bottom: 20px;
                padding-bottom: 10px;
                border-bottom: 1px solid #2ecc71;
                color: #2ecc71;
            ").ok();
        }
        
    } else {
        // Error sound
        play_sound("lose.mp3");
        
        log_to_proof_area("ERROR: Proof verification failed!").ok();
        log_to_proof_area("Please try again or contact administrator.").ok();
        
        // Add error message to proof panel
        if let Some(proof_header) = document.get_element_by_id("proof-header") {
            proof_header.set_text_content(Some("❌ Proof Generation Failed"));
            proof_header.set_attribute("style", "
                font-size: 24px;
                font-weight: bold;
                text-align: center;
                margin-bottom: 20px;
                padding-bottom: 10px;
                border-bottom: 1px solid #e74c3c;
                color: #e74c3c;
            ").ok();
        }
    }
    
    Ok(())
}

// Generate game proof (simulation)
fn generate_game_proof() -> Vec<u8> {
    // Note: This function doesn't create a real SP1 proof, it's just for demonstration
    // In a real application, the SP1 library would be used
    
    // Safely copy game state
    let (moves, timer, score, matched_pairs, is_game_over) = GAME_STATE.with(|state| {
        let game_state = state.borrow();
        (
            game_state.moves,
            game_state.timer,
            game_state.score,
            game_state.matched_pairs.clone(),
            game_state.game_over
        )
    });
    
    // Create sample proof data
    let mut proof = Vec::new();
    
    // Add game info to proof
    proof.extend_from_slice(&moves.to_le_bytes());
    proof.extend_from_slice(&timer.to_le_bytes());
    proof.extend_from_slice(&score.to_le_bytes());
    proof.push(if is_game_over { 1 } else { 0 });
    
    // Add matched cards
    for &pair in &matched_pairs {
        proof.push(pair as u8);
    }
    
    proof
}

// Start the game - this function can be called from HTML
#[wasm_bindgen]
pub fn start_game_from_js() {
    start_game();
}