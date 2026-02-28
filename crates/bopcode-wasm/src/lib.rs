pub mod engine;
pub mod host;
pub mod levels;
pub mod models;
pub mod pre_parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_simulation(puzzle_id: &str, code: &str) -> JsValue {
    let puzzle = match levels::get_puzzle(puzzle_id) {
        Some(p) => p,
        None => {
            let error_result = models::SimulationResult {
                actions: vec![],
                final_state: models::BotState::new(
                    models::Position::new(0, 0),
                    models::Direction::Right,
                ),
                final_grid: models::Grid::new(1, 1),
                puzzle_completed: false,
                stars_met: vec![],
                error: Some(models::SimulationError {
                    line: None,
                    column: None,
                    message: format!("Puzzle '{}' not found", puzzle_id),
                    friendly_hint: None,
                }),
            };
            return serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL);
        }
    };
    let result = engine::run_simulation(code, &puzzle);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn run_simulation_with_config(config: JsValue, code: &str) -> JsValue {
    let puzzle: models::PuzzleConfig = match serde_wasm_bindgen::from_value(config) {
        Ok(p) => p,
        Err(e) => {
            let error_result = models::SimulationResult {
                actions: vec![],
                final_state: models::BotState::new(
                    models::Position::new(0, 0),
                    models::Direction::Right,
                ),
                final_grid: models::Grid::new(1, 1),
                puzzle_completed: false,
                stars_met: vec![],
                error: Some(models::SimulationError {
                    line: None,
                    column: None,
                    message: format!("Invalid puzzle config: {}", e),
                    friendly_hint: None,
                }),
            };
            return serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL);
        }
    };
    let result = engine::run_simulation(code, &puzzle);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn get_worlds() -> JsValue {
    let worlds = levels::get_worlds();
    serde_wasm_bindgen::to_value(&worlds).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn get_world_levels(world_id: &str) -> JsValue {
    let levels = levels::get_world_levels(world_id);
    serde_wasm_bindgen::to_value(&levels).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn get_level(level_id: &str) -> JsValue {
    match levels::get_puzzle(level_id) {
        Some(puzzle) => serde_wasm_bindgen::to_value(&puzzle).unwrap_or(JsValue::NULL),
        None => JsValue::NULL,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::run_simulation as run_sim;
    use crate::models::*;

    // ─── Test helpers ──────────────────────────────────────────────

    fn test_grid() -> Grid {
        Grid::new(8, 8)
    }

    fn test_bot() -> BotState {
        BotState::new(Position::new(1, 1), Direction::Right)
    }

    fn run(code: &str) -> SimulationResult {
        let puzzle = PuzzleConfig {
            puzzle_id: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            grid: test_grid(),
            bot_start: test_bot(),
            completion: PuzzleObjective::ReachPosition { x: -1, y: -1 },
            star_objectives: vec![],
            starter_code: String::new(),
            hint: None,
            tutorial: None,
        };
        run_sim(code, &puzzle)
    }

    fn run_output(code: &str) -> Vec<String> {
        let result = run(code);
        result
            .actions
            .iter()
            .filter_map(|a| match a {
                GameAction::Say { message } => Some(message.clone()),
                _ => None,
            })
            .collect()
    }

    fn run_ok(code: &str) -> SimulationResult {
        let result = run(code);
        assert!(
            result.error.is_none(),
            "Expected no error, got: {:?}",
            result.error
        );
        result
    }

    fn run_err(code: &str) -> SimulationError {
        let result = run(code);
        result.error.expect("Expected an error but got none")
    }

    // ─── Language tests (via game engine pipeline) ─────────────────

    #[test]
    fn test_arithmetic() {
        assert_eq!(run_output("say(1 + 2)"), vec!["3"]);
        assert_eq!(run_output("say(10 - 3)"), vec!["7"]);
        assert_eq!(run_output("say(4 * 5)"), vec!["20"]);
        assert_eq!(run_output("say(10 / 3)"), vec!["3.3333333333333335"]);
        assert_eq!(run_output("say(10 % 3)"), vec!["1"]);
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(
            run_output(r#"say("hello" + " " + "world")"#),
            vec!["hello world"]
        );
    }

    #[test]
    fn test_variables() {
        assert_eq!(run_output("let x = 42\nsay(x)"), vec!["42"]);
    }

    #[test]
    fn test_if_else() {
        assert_eq!(
            run_output("if true { say(1) } else { say(2) }"),
            vec!["1"]
        );
        assert_eq!(
            run_output("if false { say(1) } else { say(2) }"),
            vec!["2"]
        );
    }

    #[test]
    fn test_while_loop() {
        assert_eq!(
            run_output("let i = 0\nwhile i < 3 { say(i)\ni += 1 }"),
            vec!["0", "1", "2"]
        );
    }

    #[test]
    fn test_for_in() {
        assert_eq!(
            run_output("for x in [10, 20, 30] { say(x) }"),
            vec!["10", "20", "30"]
        );
    }

    #[test]
    fn test_functions() {
        assert_eq!(
            run_output("fn add(a, b) { return a + b }\nsay(add(3, 4))"),
            vec!["7"]
        );
    }

    #[test]
    fn test_print_is_say() {
        assert_eq!(run_output("print(42)"), vec!["42"]);
        assert_eq!(run_output("print(1, 2, 3)"), vec!["1 2 3"]);
    }

    // ─── Game command tests ───────────────────────────────────────

    #[test]
    fn test_move() {
        let result = run_ok(r#"move("right")"#);
        assert_eq!(result.final_state.position, Position::new(2, 1));
    }

    #[test]
    fn test_move_and_turn() {
        let result = run_ok("move(\"right\")\nturn(\"left\")");
        assert_eq!(result.final_state.position, Position::new(2, 1));
        assert_eq!(result.final_state.direction, Direction::Up);
    }

    #[test]
    fn test_move_into_wall() {
        let err = run_err("move(\"left\")\nmove(\"left\")");
        assert!(err.message.contains("blocked"));
    }

    #[test]
    fn test_grab_gem() {
        let mut grid = test_grid();
        grid.tiles[1][2].item = Some(TileItem::Gem);
        let puzzle = PuzzleConfig {
            puzzle_id: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            grid,
            bot_start: test_bot(),
            completion: PuzzleObjective::CollectAllGems,
            star_objectives: vec![],
            starter_code: String::new(),
            hint: None,
            tutorial: None,
        };
        let result = run_sim("move(\"right\")\ngrab()", &puzzle);
        assert!(result.error.is_none());
        assert_eq!(result.final_state.gems, 1);
        assert!(result.puzzle_completed);
    }

    #[test]
    fn test_wall_ahead() {
        assert_eq!(run_output("say(wall_ahead())"), vec!["false"]);
    }

    #[test]
    fn test_look() {
        assert_eq!(run_output(r#"say(look("right"))"#), vec!["floor"]);
    }

    #[test]
    fn test_position_and_facing() {
        assert_eq!(run_output("say(position())"), vec!["[1, 1]"]);
        assert_eq!(run_output("say(facing())"), vec!["right"]);
    }

    // ─── Error handling tests ─────────────────────────────────────

    #[test]
    fn test_parse_error() {
        let err = run_err("let = 5");
        assert!(err.line.is_some());
    }

    #[test]
    fn test_undefined_variable() {
        let err = run_err("say(x)");
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn test_function_not_found_shows_hint() {
        let err = run_err("foo()");
        assert!(err.message.contains("not found"));
        assert!(err.friendly_hint.is_some());
        let hint = err.friendly_hint.unwrap();
        assert!(hint.contains("move") || hint.contains("game"));
    }

    // ─── Pre-parser tests ─────────────────────────────────────────

    #[test]
    fn test_reserved_word_precheck() {
        let err = run_err("let for = 5");
        assert!(err.message.contains("reserved"));
    }

    #[test]
    fn test_typo_precheck() {
        let err = run_err(r#"moev("right")"#);
        assert!(err.message.contains("move") || err.message.contains("moev"));
    }

    // ─── Safety tests ─────────────────────────────────────────────

    #[test]
    fn test_infinite_loop_stops() {
        let err = run_err("while true { let x = 1 }");
        assert!(err.message.contains("too many steps"));
    }

    #[test]
    fn test_deep_recursion_stops() {
        let err = run_err("fn f() { f() }\nf()");
        assert!(err.message.contains("recursion") || err.message.contains("nested"));
    }

    // ─── Level loading tests ──────────────────────────────────────

    #[test]
    fn test_get_worlds() {
        let worlds = levels::get_worlds();
        assert!(worlds.len() >= 3);
    }

    #[test]
    fn test_get_puzzle_exists() {
        assert!(levels::get_puzzle("puzzle-1").is_some());
        assert!(levels::get_puzzle("puzzle-14").is_some());
        assert!(levels::get_puzzle("bop-beg-01").is_some());
        assert!(levels::get_puzzle("bop-int-01").is_some());
    }

    #[test]
    fn test_get_puzzle_not_found() {
        assert!(levels::get_puzzle("nonexistent").is_none());
    }

    #[test]
    fn test_get_world_levels() {
        let levels = levels::get_world_levels("first-steps");
        assert_eq!(levels.len(), 14);
    }

    #[test]
    fn test_all_puzzles_load() {
        let all = levels::get_all_puzzles();
        assert!(all.len() >= 42);
    }

    #[test]
    fn test_puzzle_1_solvable() {
        let puzzle = levels::get_puzzle("puzzle-1").unwrap();
        let code = r#"repeat 7 { move("right") }"#;
        let result = run_sim(code, &puzzle);
        assert!(result.error.is_none(), "Error: {:?}", result.error);
        assert!(result.puzzle_completed);
    }

    #[test]
    fn test_puzzle_2_solvable() {
        let puzzle = levels::get_puzzle("puzzle-2").unwrap();
        let code = r#"repeat 4 { move("right") }
grab()
repeat 3 { move("right") }"#;
        let result = run_sim(code, &puzzle);
        assert!(result.error.is_none(), "Error: {:?}", result.error);
        assert!(result.puzzle_completed);
    }
}
