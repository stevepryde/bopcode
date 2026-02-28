use std::{collections::HashMap, sync::OnceLock};

use crate::models::{
    BotState, Direction, Grid, LevelSummary, Position, PuzzleConfig, PuzzleObjective, Tile,
    TileItem, TileType, WorldInfo, WorldTheme, WorldUnlock,
};
use serde::Deserialize;

const BOP_FOUNDATIONS_DATA: &str = include_str!("../data/courses/bop-foundations.json");
const BOP_OPERATIONS_DATA: &str = include_str!("../data/courses/bop-operations.json");
const FIRST_STEPS_DATA: &str = include_str!("../data/courses/first-steps.json");

#[derive(Debug, Clone)]
pub struct HardcodedWorldDefinition {
    pub world_id: String,
    pub title: String,
    pub description: String,
    pub story_intro: String,
    pub theme: WorldTheme,
    pub sort_order: u32,
    pub unlock: WorldUnlock,
}

#[derive(Debug, Deserialize)]
struct CoursePackData {
    world: CourseWorldData,
    levels: Vec<CourseLevelData>,
}

#[derive(Debug, Deserialize)]
struct CourseWorldData {
    world_id: String,
    title: String,
    description: String,
    #[serde(default)]
    story_intro: String,
    theme: WorldTheme,
    sort_order: u32,
}

#[derive(Debug, Deserialize)]
struct CourseLevelData {
    level_id: String,
    title: String,
    description: String,
    map: Vec<String>,
    completion: CourseCompletion,
    #[serde(default)]
    stars: CourseStars,
    #[serde(default)]
    starter_code: String,
    hint: Option<String>,
    tutorial: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CourseCompletion {
    Preset(CourseCompletionPreset),
    Objective(CourseObjectiveSpec),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CourseCompletionPreset {
    ReachGoal,
    CollectAllGems,
    CollectAllGemsAndReachGoal,
    DepositAllGems,
    DepositAllDiamonds,
    DepositAllGemsAndReachGoal,
    DepositAllDiamondsAndReachGoal,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CourseObjectiveSpec {
    ReachGoal,
    ReachPosition {
        x: i32,
        y: i32,
    },
    CollectAllGems,
    DepositAllGems,
    DepositAllDiamonds,
    All {
        conditions: Vec<CourseObjectiveSpec>,
    },
}

#[derive(Debug, Default, Deserialize)]
struct CourseStars {
    #[serde(default)]
    collect_all_gems: bool,
    max_instructions: Option<u32>,
    max_steps: Option<u32>,
}

#[derive(Debug, Clone)]
struct ParsedCoursePack {
    world: HardcodedWorldDefinition,
    puzzles: Vec<PuzzleConfig>,
    puzzle_ids: Vec<String>,
    puzzles_by_id: HashMap<String, PuzzleConfig>,
}

static FIRST_STEPS_PACK: OnceLock<ParsedCoursePack> = OnceLock::new();
static BOP_FOUNDATIONS_PACK: OnceLock<ParsedCoursePack> = OnceLock::new();
static BOP_OPERATIONS_PACK: OnceLock<ParsedCoursePack> = OnceLock::new();

fn first_steps_pack() -> &'static ParsedCoursePack {
    FIRST_STEPS_PACK
        .get_or_init(|| parse_course_pack(FIRST_STEPS_DATA, "first-steps.json"))
}

fn bop_foundations_pack() -> &'static ParsedCoursePack {
    BOP_FOUNDATIONS_PACK
        .get_or_init(|| parse_course_pack(BOP_FOUNDATIONS_DATA, "bop-foundations.json"))
}

fn bop_operations_pack() -> &'static ParsedCoursePack {
    BOP_OPERATIONS_PACK
        .get_or_init(|| parse_course_pack(BOP_OPERATIONS_DATA, "bop-operations.json"))
}

fn parse_course_pack(data: &str, file_name: &str) -> ParsedCoursePack {
    let data: CoursePackData =
        serde_json::from_str(data).unwrap_or_else(|e| panic!("{file_name} must be valid: {e}"));

    let world = HardcodedWorldDefinition {
        world_id: data.world.world_id,
        title: data.world.title,
        description: data.world.description,
        story_intro: data.world.story_intro,
        theme: data.world.theme,
        sort_order: data.world.sort_order,
        unlock: WorldUnlock::Open,
    };

    let puzzles: Vec<PuzzleConfig> = data.levels.into_iter().map(parse_course_level).collect();
    let puzzle_ids: Vec<String> = puzzles.iter().map(|p| p.puzzle_id.clone()).collect();
    let puzzles_by_id: HashMap<String, PuzzleConfig> = puzzles
        .iter()
        .cloned()
        .map(|p| (p.puzzle_id.clone(), p))
        .collect();

    ParsedCoursePack {
        world,
        puzzles,
        puzzle_ids,
        puzzles_by_id,
    }
}

fn parse_course_level(level: CourseLevelData) -> PuzzleConfig {
    let (grid, bot_start, goal_position) = parse_ascii_map(&level.level_id, &level.map);

    let mut completion = parse_course_completion(&level.level_id, level.completion, goal_position);

    // If collect_all_gems is specified, fold it into the completion objective
    if level.stars.collect_all_gems {
        let already_requires_gems = completion_requires_collect_all_gems(&completion);
        if !already_requires_gems {
            completion = PuzzleObjective::All {
                conditions: vec![completion, PuzzleObjective::CollectAllGems],
            };
        }
    }

    let mut star_objectives = Vec::new();
    if let Some(instructions) = level.stars.max_instructions {
        star_objectives.push(PuzzleObjective::MaxInstructions { instructions });
    }
    if let Some(steps) = level.stars.max_steps {
        star_objectives.push(PuzzleObjective::MaxSteps { steps });
    }

    PuzzleConfig {
        puzzle_id: level.level_id,
        title: level.title,
        description: level.description,
        grid,
        bot_start,
        completion,
        star_objectives,
        starter_code: level.starter_code,
        hint: level.hint,
        tutorial: level.tutorial,
    }
}

fn completion_requires_collect_all_gems(objective: &PuzzleObjective) -> bool {
    match objective {
        PuzzleObjective::CollectAllGems => true,
        PuzzleObjective::All { conditions } => {
            conditions.iter().any(completion_requires_collect_all_gems)
        }
        _ => false,
    }
}

fn parse_course_completion(
    level_id: &str,
    completion: CourseCompletion,
    goal_position: Option<Position>,
) -> PuzzleObjective {
    fn reach_goal(level_id: &str, goal_position: Option<Position>) -> PuzzleObjective {
        let goal = goal_position
            .unwrap_or_else(|| panic!("Level '{level_id}' uses reach_goal but has no goal tile"));
        PuzzleObjective::ReachPosition {
            x: goal.x,
            y: goal.y,
        }
    }

    fn parse_spec(
        level_id: &str,
        spec: CourseObjectiveSpec,
        goal_position: Option<Position>,
    ) -> PuzzleObjective {
        match spec {
            CourseObjectiveSpec::ReachGoal => reach_goal(level_id, goal_position),
            CourseObjectiveSpec::ReachPosition { x, y } => PuzzleObjective::ReachPosition { x, y },
            CourseObjectiveSpec::CollectAllGems => PuzzleObjective::CollectAllGems,
            CourseObjectiveSpec::DepositAllGems => PuzzleObjective::DepositAllGems,
            CourseObjectiveSpec::DepositAllDiamonds => PuzzleObjective::DepositAllDiamonds,
            CourseObjectiveSpec::All { conditions } => PuzzleObjective::All {
                conditions: conditions
                    .into_iter()
                    .map(|condition| parse_spec(level_id, condition, goal_position))
                    .collect(),
            },
        }
    }

    match completion {
        CourseCompletion::Preset(preset) => match preset {
            CourseCompletionPreset::ReachGoal => reach_goal(level_id, goal_position),
            CourseCompletionPreset::CollectAllGems => PuzzleObjective::CollectAllGems,
            CourseCompletionPreset::CollectAllGemsAndReachGoal => PuzzleObjective::All {
                conditions: vec![
                    PuzzleObjective::CollectAllGems,
                    reach_goal(level_id, goal_position),
                ],
            },
            CourseCompletionPreset::DepositAllGems => PuzzleObjective::DepositAllGems,
            CourseCompletionPreset::DepositAllDiamonds => PuzzleObjective::DepositAllDiamonds,
            CourseCompletionPreset::DepositAllGemsAndReachGoal => PuzzleObjective::All {
                conditions: vec![
                    PuzzleObjective::DepositAllGems,
                    reach_goal(level_id, goal_position),
                ],
            },
            CourseCompletionPreset::DepositAllDiamondsAndReachGoal => PuzzleObjective::All {
                conditions: vec![
                    PuzzleObjective::DepositAllDiamonds,
                    reach_goal(level_id, goal_position),
                ],
            },
        },
        CourseCompletion::Objective(spec) => parse_spec(level_id, spec, goal_position),
    }
}

fn parse_ascii_map(level_id: &str, rows: &[String]) -> (Grid, BotState, Option<Position>) {
    assert!(
        !rows.is_empty(),
        "Level '{level_id}' map must have at least one row"
    );

    let height = rows.len() as u32;
    let width = rows[0].chars().count() as u32;
    assert!(width > 0, "Level '{level_id}' map rows must not be empty");

    let mut start: Option<BotState> = None;
    let mut goal_position: Option<Position> = None;
    let mut tiles: Vec<Vec<Tile>> = Vec::with_capacity(rows.len());

    for (y, row) in rows.iter().enumerate() {
        let row_width = row.chars().count() as u32;
        assert_eq!(
            row_width, width,
            "Level '{level_id}' map rows must all have equal width"
        );

        let mut parsed_row = Vec::with_capacity(width as usize);
        for (x, c) in row.chars().enumerate() {
            let pos = Position::new(x as i32, y as i32);
            let tile = match c {
                '.' => Tile::floor(),
                '#' => Tile::wall(),
                'P' => Tile {
                    tile_type: TileType::Pit,
                    item: None,
                },
                'D' => Tile {
                    tile_type: TileType::LockedDoor,
                    item: None,
                },
                'V' => Tile {
                    tile_type: TileType::GemVault,
                    item: None,
                },
                'W' => Tile {
                    tile_type: TileType::DiamondVault,
                    item: None,
                },
                'G' => {
                    if goal_position.is_none() {
                        goal_position = Some(pos);
                    }
                    Tile::goal()
                }
                '*' => Tile::floor().with_gem(),
                '+' => {
                    if goal_position.is_none() {
                        goal_position = Some(pos);
                    }
                    Tile::goal().with_gem()
                }
                'K' => {
                    let mut tile = Tile::floor();
                    tile.item = Some(TileItem::Key);
                    tile
                }
                'R' => {
                    let mut tile = Tile::floor();
                    tile.item = Some(TileItem::Diamond);
                    tile
                }
                'S' => {
                    assert!(
                        start.is_none(),
                        "Level '{level_id}' map must contain only one start tile"
                    );
                    start = Some(BotState::new(pos, Direction::Right));
                    Tile::floor()
                }
                '^' => {
                    assert!(
                        start.is_none(),
                        "Level '{level_id}' map must contain only one start tile"
                    );
                    start = Some(BotState::new(pos, Direction::Up));
                    Tile::floor()
                }
                'v' => {
                    assert!(
                        start.is_none(),
                        "Level '{level_id}' map must contain only one start tile"
                    );
                    start = Some(BotState::new(pos, Direction::Down));
                    Tile::floor()
                }
                '<' => {
                    assert!(
                        start.is_none(),
                        "Level '{level_id}' map must contain only one start tile"
                    );
                    start = Some(BotState::new(pos, Direction::Left));
                    Tile::floor()
                }
                '>' => {
                    assert!(
                        start.is_none(),
                        "Level '{level_id}' map must contain only one start tile"
                    );
                    start = Some(BotState::new(pos, Direction::Right));
                    Tile::floor()
                }
                _ => panic!("Level '{level_id}' map has unsupported tile '{c}'"),
            };
            parsed_row.push(tile);
        }
        tiles.push(parsed_row);
    }

    let bot_start = start.unwrap_or_else(|| {
        panic!("Level '{level_id}' map must contain one start tile (S, <, >, ^, or v)")
    });

    (
        Grid {
            width,
            height,
            tiles,
        },
        bot_start,
        goal_position,
    )
}

pub fn get_hardcoded_worlds() -> Vec<HardcodedWorldDefinition> {
    vec![
        first_steps_pack().world.clone(),
        bop_foundations_pack().world.clone(),
        bop_operations_pack().world.clone(),
    ]
}

pub fn get_all_world_ids() -> Vec<String> {
    get_hardcoded_worlds()
        .into_iter()
        .map(|w| w.world_id)
        .collect()
}

/// Get all available puzzles
pub fn get_all_puzzles() -> Vec<PuzzleConfig> {
    let mut puzzles = Vec::new();
    puzzles.extend(first_steps_pack().puzzles.clone());
    puzzles.extend(bop_foundations_pack().puzzles.clone());
    puzzles.extend(bop_operations_pack().puzzles.clone());
    puzzles
}

/// Get a puzzle by ID
pub fn get_puzzle(puzzle_id: &str) -> Option<PuzzleConfig> {
    if puzzle_id == "demo" {
        return Some(demo_puzzle());
    }

    first_steps_pack()
        .puzzles_by_id
        .get(puzzle_id)
        .cloned()
        .or_else(|| bop_foundations_pack().puzzles_by_id.get(puzzle_id).cloned())
        .or_else(|| bop_operations_pack().puzzles_by_id.get(puzzle_id).cloned())
}

/// Returns all puzzle IDs belonging to a given world.
pub fn get_world_puzzle_ids(world_id: &str) -> Vec<String> {
    for pack_fn in [first_steps_pack, bop_foundations_pack, bop_operations_pack] {
        let pack = pack_fn();
        if pack.world.world_id == world_id {
            return pack.puzzle_ids.clone();
        }
    }
    vec![]
}

// ─── Frontend-friendly public API ──────────────────────────────────────────

pub fn get_worlds() -> Vec<WorldInfo> {
    let mut worlds = get_hardcoded_worlds();
    worlds.sort_by_key(|w| w.sort_order);
    worlds
        .into_iter()
        .map(|w| {
            let level_count = get_world_puzzle_ids(&w.world_id).len() as u32;
            WorldInfo {
                world_id: w.world_id,
                title: w.title,
                description: w.description,
                story_intro: w.story_intro,
                theme: w.theme,
                sort_order: w.sort_order,
                unlock: w.unlock,
                level_count,
            }
        })
        .collect()
}

pub fn get_world_levels(world_id: &str) -> Vec<LevelSummary> {
    let puzzle_ids = get_world_puzzle_ids(world_id);
    puzzle_ids
        .into_iter()
        .filter_map(|id| {
            get_puzzle(&id).map(|p| LevelSummary {
                puzzle_id: p.puzzle_id,
                title: p.title,
                description: p.description,
            })
        })
        .collect()
}

// ─── Helper ────────────────────────────────────────────────────────────────

/// Build a grid that's all walls, then carve floors.
fn wall_grid(width: u32, height: u32) -> Grid {
    let tiles = (0..height)
        .map(|_| (0..width).map(|_| Tile::wall()).collect())
        .collect();
    Grid {
        width,
        height,
        tiles,
    }
}


// ─── Demo (start-page only, not part of any world) ───────────────────────

/// A 12×8 snake-path puzzle used on the public start page.
/// Bot zigzags: right across row 0, down to row 2, left across row 2,
/// down to row 4, right across row 4, down to row 6, left across row 6,
/// down to goal at (1, 7).
/// Gems placed at turns and midpoints to encourage grab()/if gem_here().
fn demo_puzzle() -> PuzzleConfig {
    let mut grid = wall_grid(12, 8);

    // Row 0: x=0..10  (right)
    for x in 0..11 {
        grid.set_tile(Position::new(x, 0), Tile::floor());
    }
    grid.set_tile(Position::new(5, 0), Tile::floor().with_gem());

    // Connector: x=10, y=1
    grid.set_tile(Position::new(10, 1), Tile::floor());

    // Row 2: x=1..10  (left)
    for x in 1..11 {
        grid.set_tile(Position::new(x, 2), Tile::floor());
    }
    grid.set_tile(Position::new(10, 2), Tile::floor().with_gem());
    grid.set_tile(Position::new(3, 2), Tile::floor().with_gem());

    // Connector: x=1, y=3
    grid.set_tile(Position::new(1, 3), Tile::floor());

    // Row 4: x=1..10  (right)
    for x in 1..11 {
        grid.set_tile(Position::new(x, 4), Tile::floor());
    }
    grid.set_tile(Position::new(1, 4), Tile::floor().with_gem());
    grid.set_tile(Position::new(7, 4), Tile::floor().with_gem());

    // Connector: x=10, y=5
    grid.set_tile(Position::new(10, 5), Tile::floor());

    // Row 6: x=1..10  (left)
    for x in 1..11 {
        grid.set_tile(Position::new(x, 6), Tile::floor());
    }
    grid.set_tile(Position::new(6, 6), Tile::floor().with_gem());

    // Connector: x=1, y=7  (goal)
    grid.set_tile(Position::new(1, 7), Tile::goal());

    PuzzleConfig {
        puzzle_id: "demo".to_string(),
        title: "Snake Run".to_string(),
        description: "Navigate the winding path and collect gems along the way!".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 1, y: 7 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxSteps { steps: 55 },
        ],
        starter_code: r#"// Navigate the winding path!
        fn grab_if_gem() {
    if gem_here() {
        grab()
    }
}

let turns = ["right", "right", "left", "left"]

for t in turns {
    while path_ahead() {
        move("forward") // or "right", "up", etc.
        grab_if_gem()
    }
    turn(t)
}
"#
        .to_string(),
        hint: None,
        tutorial: None,
    }
}
