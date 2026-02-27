use std::{collections::HashMap, sync::OnceLock};

use crate::models::{
    BotState, Direction, Grid, LevelSummary, Position, PuzzleConfig, PuzzleObjective, Tile,
    TileItem, TileType, WorldInfo, WorldTheme, WorldUnlock,
};
use serde::Deserialize;

const FIRST_STEPS_WORLD_ID: &str = "first-steps";
const BOP_FOUNDATIONS_DATA: &str = include_str!("../data/courses/bop-foundations.json");
const BOP_OPERATIONS_DATA: &str = include_str!("../data/courses/bop-operations.json");

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

static BOP_FOUNDATIONS_PACK: OnceLock<ParsedCoursePack> = OnceLock::new();
static BOP_OPERATIONS_PACK: OnceLock<ParsedCoursePack> = OnceLock::new();

fn first_steps_world() -> HardcodedWorldDefinition {
    HardcodedWorldDefinition {
        world_id: FIRST_STEPS_WORLD_ID.to_string(),
        title: "First Steps".to_string(),
        description: "A faster introduction to Bop for learners who want to move quickly."
            .to_string(),
        story_intro: "Move fast through Bop fundamentals with 14 compact challenges.".to_string(),
        theme: WorldTheme::GrassyPlains,
        sort_order: 10,
        unlock: WorldUnlock::Open,
    }
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

    let completion = parse_course_completion(&level.level_id, level.completion, goal_position);

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
        bop_foundations_pack().world.clone(),
        bop_operations_pack().world.clone(),
        first_steps_world(),
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
    let mut puzzles = vec![
        puzzle_1_walk_to_goal(),
        puzzle_2_collect_gem(),
        puzzle_3_around_wall(),
        puzzle_4_repeat_after_me(),
        puzzle_5_gem_trail(),
        puzzle_6_the_l_shape(),
        puzzle_7_while_you_can(),
        puzzle_8_check_before_you_grab(),
        puzzle_9_wall_hugger(),
        puzzle_10_gem_collector(),
        puzzle_11_for_the_win(),
        puzzle_12_helper_bot(),
        puzzle_13_maze_runner(),
        puzzle_14_the_grand_tour(),
    ];
    puzzles.extend(bop_foundations_pack().puzzles.clone());
    puzzles.extend(bop_operations_pack().puzzles.clone());
    puzzles
}

/// Get a puzzle by ID
pub fn get_puzzle(puzzle_id: &str) -> Option<PuzzleConfig> {
    let core = match puzzle_id {
        "puzzle-1" => Some(puzzle_1_walk_to_goal()),
        "puzzle-2" => Some(puzzle_2_collect_gem()),
        "puzzle-3" => Some(puzzle_3_around_wall()),
        "puzzle-4" => Some(puzzle_4_repeat_after_me()),
        "puzzle-5" => Some(puzzle_5_gem_trail()),
        "puzzle-6" => Some(puzzle_6_the_l_shape()),
        "puzzle-7" => Some(puzzle_7_while_you_can()),
        "puzzle-8" => Some(puzzle_8_check_before_you_grab()),
        "puzzle-9" => Some(puzzle_9_wall_hugger()),
        "puzzle-10" => Some(puzzle_10_gem_collector()),
        "puzzle-11" => Some(puzzle_11_for_the_win()),
        "puzzle-12" => Some(puzzle_12_helper_bot()),
        "puzzle-13" => Some(puzzle_13_maze_runner()),
        "puzzle-14" => Some(puzzle_14_the_grand_tour()),
        "demo" => Some(demo_puzzle()),
        _ => None,
    };

    if core.is_some() {
        core
    } else {
        bop_foundations_pack()
            .puzzles_by_id
            .get(puzzle_id)
            .cloned()
            .or_else(|| bop_operations_pack().puzzles_by_id.get(puzzle_id).cloned())
    }
}

/// Returns all puzzle IDs belonging to a given world.
pub fn get_world_puzzle_ids(world_id: &str) -> Vec<String> {
    match world_id {
        FIRST_STEPS_WORLD_ID => vec![
            "puzzle-1".to_string(),
            "puzzle-2".to_string(),
            "puzzle-3".to_string(),
            "puzzle-4".to_string(),
            "puzzle-5".to_string(),
            "puzzle-6".to_string(),
            "puzzle-7".to_string(),
            "puzzle-8".to_string(),
            "puzzle-9".to_string(),
            "puzzle-10".to_string(),
            "puzzle-11".to_string(),
            "puzzle-12".to_string(),
            "puzzle-13".to_string(),
            "puzzle-14".to_string(),
        ],
        world_id if world_id == bop_foundations_pack().world.world_id => {
            bop_foundations_pack().puzzle_ids.clone()
        }
        world_id if world_id == bop_operations_pack().world.world_id => {
            bop_operations_pack().puzzle_ids.clone()
        }
        _ => vec![],
    }
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

// ─── Puzzle 1 ──────────────────────────────────────────────────────────────

/// Walk to the Goal — teaches move()
fn puzzle_1_walk_to_goal() -> PuzzleConfig {
    let mut grid = Grid::new(8, 1);
    grid.set_tile(Position::new(7, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-1".to_string(),
        title: "Walk to the Goal".to_string(),
        description: "Help the bot reach the glowing goal tile! Use the move() function with a direction.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 0 },
        star_objectives: vec![
            PuzzleObjective::MaxInstructions { instructions: 7 },
            PuzzleObjective::MaxSteps { steps: 7 },
        ],
        starter_code: r#"// Move the bot to the goal!
// Use: move("right") to move right

move("right")
"#.to_string(),
        hint: Some("You need to move right 7 times to reach the goal.".to_string()),
        tutorial: Some(r#"# Welcome to First Steps!

You control a **bot** on a grid by writing code. Your goal is to guide the bot to the **green goal tile**.

## Your first command: `move()`

The `move()` function moves the bot one tile in a direction:

```
move("right")
move("left")
move("up")
move("down")
```

Each `move()` call moves the bot **one step**. To move multiple tiles, call `move()` multiple times.

## Your mission

The goal tile is 7 steps to the right. Write enough `move("right")` calls to get there!
"#.to_string()),
    }
}

// ─── Puzzle 2 ──────────────────────────────────────────────────────────────

/// Collect the Gem — teaches grab(), sequencing
fn puzzle_2_collect_gem() -> PuzzleConfig {
    let mut grid = Grid::new(8, 1);
    grid.set_tile(Position::new(4, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(7, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-2".to_string(),
        title: "Collect the Gem".to_string(),
        description: "Grab the gem and then reach the goal! Use grab() when standing on a gem.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 0 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 10 },
            PuzzleObjective::MaxSteps { steps: 10 },
        ],
        starter_code: r#"// Collect the gem and reach the goal!
// Use: grab() to pick up a gem when standing on it

move("right")
"#.to_string(),
        hint: Some("Move to the gem first, grab() it, then continue to the goal.".to_string()),
        tutorial: Some(r#"# Collecting Gems

Some tiles have **gems** on them. You can pick them up with the `grab()` function.

## How `grab()` works

When your bot is standing on a tile with a gem, call:

```
grab()
```

This picks up the gem. If there's no gem on the tile, `grab()` will cause an error — so only use it when you're on a gem!

## Sequencing instructions

Your code runs **top to bottom**, one instruction at a time:

```
move("right")   // step 1
move("right")   // step 2
grab()           // step 3
move("right")   // step 4
```

## Your mission

Move to the gem, grab it, then continue to the goal.
"#.to_string()),
    }
}

// ─── Puzzle 3 ──────────────────────────────────────────────────────────────

/// Around the Wall — teaches turn(), navigation
fn puzzle_3_around_wall() -> PuzzleConfig {
    let mut grid = Grid::new(8, 8);

    // Wall at x=4 from y=2 to y=6
    for y in 2..=6 {
        grid.set_tile(Position::new(4, y), Tile::wall());
    }

    grid.set_tile(Position::new(6, 4), Tile::floor().with_gem());
    grid.set_tile(Position::new(7, 4), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-3".to_string(),
        title: "Around the Wall".to_string(),
        description: "There's a wall in the way! Navigate around it to collect the gem and reach the goal.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 4), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 4 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 14 },
            PuzzleObjective::MaxSteps { steps: 16 },
        ],
        starter_code: r#"// Navigate around the wall!
// Use: move("up"), move("down"), move("left"), move("right")

move("right")
"#.to_string(),
        hint: Some("Move up to get around the wall, then right, then down to the gem.".to_string()),
        tutorial: Some(r#"# Navigating in All Directions

So far you've moved right. But the grid is 2D — you can move in **all four directions**:

```
move("up")
move("down")
move("left")
move("right")
```

## Walls

Grey tiles are **walls** — you can't walk through them. If you try, you'll get an error. You need to find a path *around* walls.

## Planning a path

Look at the grid and plan your route:
1. Which direction gets you closer to the goal?
2. If there's a wall, how can you go around it?

## Your mission

There's a wall blocking your path. Navigate around it, collect the gem, and reach the goal.
"#.to_string()),
    }
}

// ─── Puzzle 4 ──────────────────────────────────────────────────────────────

/// Repeat After Me — teaches repeat N { }
fn puzzle_4_repeat_after_me() -> PuzzleConfig {
    let mut grid = Grid::new(8, 1);
    grid.set_tile(Position::new(4, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(7, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-4".to_string(),
        title: "Repeat After Me".to_string(),
        description: "That's a long corridor! Instead of writing move() seven times, use repeat to do it in one line.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 0 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 5 },
            PuzzleObjective::MaxSteps { steps: 9 },
        ],
        starter_code: r#"// Use repeat to move efficiently!
// repeat N { ... } runs the code N times
// Example: repeat 3 { say("hi") }

move("right")
"#.to_string(),
        hint: Some("Try: repeat 7 { move(\"right\") }".to_string()),
        tutorial: Some(r#"# Loops: `repeat`

Writing `move("right")` seven times works, but there's a better way!

## The `repeat` block

`repeat` runs the same code multiple times:

```
repeat 3 {
    move("right")
}
```

This is the same as writing `move("right")` three times — but much shorter!

## How it works

- `repeat N { ... }` runs the code inside the braces **N times**
- N can be any whole number
- The code inside the `{ }` can be any instructions

## Your mission

Use `repeat` to reach the goal without writing `move()` seven separate times. Don't forget the gem!
"#.to_string()),
    }
}

// ─── Puzzle 5 ──────────────────────────────────────────────────────────────

/// Gem Trail — actions inside loops
fn puzzle_5_gem_trail() -> PuzzleConfig {
    let mut grid = Grid::new(8, 1);
    for x in 1..=6 {
        grid.set_tile(Position::new(x, 0), Tile::floor().with_gem());
    }
    grid.set_tile(Position::new(7, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-5".to_string(),
        title: "Gem Trail".to_string(),
        description: "A trail of gems leads to the goal. Collect them all using a loop!".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 0 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 4 },
            PuzzleObjective::MaxSteps { steps: 14 },
        ],
        starter_code: r#"// Collect all gems on the way to the goal!
// Put multiple actions inside a repeat block

move("right")
"#.to_string(),
        hint: Some("Try: repeat 6 { move(\"right\") grab() } then one more move.".to_string()),
        tutorial: Some(r#"# Multiple Actions in Loops

A `repeat` block can contain **more than one instruction**:

```
repeat 3 {
    move("right")
    grab()
}
```

This moves right **and** grabs, three times in a row. The bot does both actions in order, then repeats.

## Think in patterns

Look at the grid — is there a repeating pattern? If the bot needs to "move then grab" over and over, that's a perfect loop body.

## Your mission

There's a trail of gems leading to the goal. Use a loop with multiple actions to collect them all efficiently.
"#.to_string()),
    }
}

// ─── Puzzle 6 ──────────────────────────────────────────────────────────────

/// The L-Shape — multiple repeat blocks
fn puzzle_6_the_l_shape() -> PuzzleConfig {
    // 6×6 grid, L-shaped corridor: row 0 (y=0) + column 5 (x=5)
    let mut grid = wall_grid(6, 6);

    // Horizontal corridor at y=0
    for x in 0..6 {
        grid.set_tile(Position::new(x, 0), Tile::floor());
    }
    // Vertical corridor at x=5
    for y in 0..6 {
        grid.set_tile(Position::new(5, y), Tile::floor());
    }
    grid.set_tile(Position::new(5, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(5, 5), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-6".to_string(),
        title: "The L-Shape".to_string(),
        description: "Follow the L-shaped corridor to the goal. You'll need two repeat blocks!".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 5, y: 5 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 5 },
            PuzzleObjective::MaxSteps { steps: 11 },
        ],
        starter_code: r#"// Navigate the L-shaped path!
// Use two repeat blocks — one for each direction

move("right")
"#.to_string(),
        hint: Some("Move right 5 times, then down 5 times.".to_string()),
        tutorial: Some(r#"# Multiple Repeat Blocks

You're not limited to a single loop! You can use **multiple repeat blocks** in sequence:

```
repeat 5 { move("right") }
repeat 3 { move("down") }
```

The first loop runs to completion, then the second loop starts.

## Sequencing loops

Think of each loop as one "section" of your path. When the path changes direction, start a new loop.

## Your mission

The corridor turns from horizontal to vertical. Use two repeat blocks — one for each direction — and don't forget the gem at the corner!
"#.to_string()),
    }
}

// ─── Puzzle 7 ──────────────────────────────────────────────────────────────

/// While You Can — teaches while + path_ahead()
fn puzzle_7_while_you_can() -> PuzzleConfig {
    // 6×3 grid, corridor at y=1 with wall at (5,1)
    let mut grid = wall_grid(6, 3);

    // Floor corridor at y=1 from x=0 to x=4
    for x in 0..5 {
        grid.set_tile(Position::new(x, 1), Tile::floor());
    }
    // Wall at (5,1) is already wall from wall_grid
    grid.set_tile(Position::new(2, 1), Tile::floor());
    grid.set_tile(Position::new(4, 1), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-7".to_string(),
        title: "While You Can".to_string(),
        description:
            "Walk forward until you hit a wall! Use while and path_ahead() to stop automatically."
                .to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 1), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 4, y: 1 },
        star_objectives: vec![
            PuzzleObjective::MaxInstructions { instructions: 4 },
            PuzzleObjective::MaxSteps { steps: 6 },
        ],
        starter_code: r#"// Use while to keep moving until blocked!
// path_ahead() returns true if you can move forward
// wall_ahead() returns true if there's a wall ahead

move("right")
"#
        .to_string(),
        hint: Some("Try: while path_ahead() { move(\"right\") }".to_string()),
        tutorial: Some(
            r#"# While Loops and Sensors

Sometimes you don't know *exactly* how many steps to take. That's where `while` loops come in!

## The `while` loop

A `while` loop keeps running as long as its condition is **true**:

```
while path_ahead() {
    move("right")
}
```

This moves right until there's no more path ahead.

## Sensors

Your bot has built-in sensors that return `true` or `false`:

| Sensor | Returns `true` when... |
|---|---|
| `path_ahead()` | The tile ahead is walkable |
| `wall_ahead()` | The tile ahead is a wall |

## `while` vs `repeat`

- Use `repeat N` when you know the exact count
- Use `while condition` when you want to keep going until something changes

## Your mission

Walk forward until you hit a wall. Use `while` and a sensor to stop automatically!
"#
            .to_string(),
        ),
    }
}

// ─── Puzzle 8 ──────────────────────────────────────────────────────────────

/// Check Before You Grab — teaches if + gem_here()
fn puzzle_8_check_before_you_grab() -> PuzzleConfig {
    // 8×1 corridor, gems at odd x positions
    let mut grid = Grid::new(8, 1);
    grid.set_tile(Position::new(1, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(3, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(5, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(7, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-8".to_string(),
        title: "Check Before You Grab".to_string(),
        description: "Some tiles have gems, some don't. Use if and gem_here() to grab only when there's a gem!".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 0 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 5 },
            PuzzleObjective::MaxSteps { steps: 11 },
        ],
        starter_code: r#"// Grab gems only where they exist!
// gem_here() returns true if there's a gem on your tile
// if gem_here() { grab() }

move("right")
"#.to_string(),
        hint: Some("Try a repeat loop: move right, then if gem_here() { grab() }.".to_string()),
        tutorial: Some(r#"# Conditionals: `if`

What if you only want to do something *sometimes*? Use `if`!

## The `if` statement

`if` runs code only when a condition is true:

```
if gem_here() {
    grab()
}
```

This grabs **only** if there's a gem on the current tile. Otherwise it does nothing.

## The `gem_here()` sensor

| Sensor | Returns `true` when... |
|---|---|
| `gem_here()` | There's a gem on your current tile |

## Combining `if` with loops

You can put `if` inside a loop to check on every step:

```
repeat 5 {
    move("right")
    if gem_here() { grab() }
}
```

## Your mission

Gems are scattered along the path — but not on every tile. Use `if gem_here()` to grab only where gems exist.
"#.to_string()),
    }
}

// ─── Puzzle 9 ──────────────────────────────────────────────────────────────

/// Wall Hugger — wall_ahead() + turn()
fn puzzle_9_wall_hugger() -> PuzzleConfig {
    // 5×5 S-shaped zigzag
    let mut grid = wall_grid(5, 5);

    // Row 0: x=0..4
    for x in 0..5 {
        grid.set_tile(Position::new(x, 0), Tile::floor());
    }
    // Connect row 0 to row 2 at x=4
    grid.set_tile(Position::new(4, 1), Tile::floor());
    // Row 2: x=0..4
    for x in 0..5 {
        grid.set_tile(Position::new(x, 2), Tile::floor());
    }
    // Connect row 2 to row 4 at x=0
    grid.set_tile(Position::new(0, 3), Tile::floor());
    // Row 4: x=0..4
    for x in 0..5 {
        grid.set_tile(Position::new(x, 4), Tile::floor());
    }
    grid.set_tile(Position::new(2, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(4, 4), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-9".to_string(),
        title: "Wall Hugger".to_string(),
        description: "Navigate the zigzag path! Use wall_ahead() to know when to turn.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 4, y: 4 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 13 },
            PuzzleObjective::MaxSteps { steps: 20 },
        ],
        starter_code: r#"// Navigate the zigzag!
// wall_ahead() — true if the next tile is a wall
// path_ahead() — true if you can move forward
// turn("left") / turn("right") — change direction

move("right")
"#.to_string(),
        hint: Some("Move forward, and when you hit a wall, turn. The pattern is: right, down, left, down, right.".to_string()),
        tutorial: Some(r#"# Turning and Wall Detection

Your bot has a **facing direction**. The `turn()` function changes which way the bot faces, and `wall_ahead()` checks what's in front of it.

## Turning

```
turn("left")    // rotate 90° left
turn("right")   // rotate 90° right
```

After turning, `move("right")` still moves right on the grid — but `path_ahead()` and `wall_ahead()` now check the bot's new facing direction.

## The `wall_ahead()` sensor

| Sensor | Returns `true` when... |
|---|---|
| `wall_ahead()` | The tile ahead (in facing direction) is a wall |

## Navigating zigzag paths

When a path zigzags, you can break it into straight sections:

```
repeat 4 { move("right") }
repeat 2 { move("down") }
repeat 4 { move("left") }
```

## Your mission

Navigate the S-shaped zigzag path to reach the goal. Collect the gem along the way!
"#.to_string()),
    }
}

// ─── Puzzle 10 ─────────────────────────────────────────────────────────────

/// Gem Collector — while + if combined
fn puzzle_10_gem_collector() -> PuzzleConfig {
    // 7×1 corridor at y=0, gems at positions 1, 3, 5
    let mut grid = Grid::new(7, 1);
    grid.set_tile(Position::new(1, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(3, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(5, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(6, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-10".to_string(),
        title: "Gem Collector".to_string(),
        description: "Combine while and if to collect gems automatically until you reach the goal!"
            .to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 6, y: 0 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 5 },
            PuzzleObjective::MaxSteps { steps: 12 },
        ],
        starter_code: r#"// Use while and if together!
// path_ahead() returns true if you can keep moving
// gem_here() returns true when there's a gem

move("right")
"#
        .to_string(),
        hint: Some(
            "Try: while path_ahead() { move(\"right\") if gem_here() { grab() } }".to_string(),
        ),
        tutorial: Some(
            r#"# Combining `while` + `if`

Now you know both `while` loops and `if` conditionals. Let's combine them!

## Negation with `!`

Use `!` to flip a condition:

```
while !wall_ahead() {
    // keep going until we hit a wall
}
```

`!wall_ahead()` means "while there is NOT a wall ahead."

## Nesting `if` inside `while`

```
while path_ahead() {
    move("right")
    if gem_here() { grab() }
}
```

Each iteration: move, then check for a gem. This repeats until the bot can't move further.

## Your mission

Walk to the goal and collect every gem along the way using `while` and `if` together.
"#
            .to_string(),
        ),
    }
}

// ─── Puzzle 11 ─────────────────────────────────────────────────────────────

/// For the Win — teaches for + range()
fn puzzle_11_for_the_win() -> PuzzleConfig {
    // 5×5 staircase: gems at (1,0),(2,1),(3,2),(4,3), goal at (4,4)
    let mut grid = wall_grid(5, 5);

    // Carve the staircase path
    grid.set_tile(Position::new(0, 0), Tile::floor());
    grid.set_tile(Position::new(1, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(1, 1), Tile::floor());
    grid.set_tile(Position::new(2, 1), Tile::floor().with_gem());
    grid.set_tile(Position::new(2, 2), Tile::floor());
    grid.set_tile(Position::new(3, 2), Tile::floor().with_gem());
    grid.set_tile(Position::new(3, 3), Tile::floor());
    grid.set_tile(Position::new(4, 3), Tile::floor().with_gem());
    grid.set_tile(Position::new(4, 4), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-11".to_string(),
        title: "For the Win".to_string(),
        description: "Descend the staircase! Each step goes right then down. Use a for loop to repeat the pattern.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 4, y: 4 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 6 },
            PuzzleObjective::MaxSteps { steps: 13 },
        ],
        starter_code: r#"// Use a for loop to descend the staircase!
// for i in range(N) { ... } repeats N times
// Each step: move right, grab, move down

move("right")
"#.to_string(),
        hint: Some("Try: for i in range(4) { move(\"right\") grab() move(\"down\") }".to_string()),
        tutorial: Some(r#"# For Loops

The `for` loop gives you a **counter variable** that changes each iteration.

## The `for` loop

```
for i in range(4) {
    say(i)
}
```

This runs 4 times, with `i` being 0, 1, 2, 3.

## `for` vs `repeat`

- `repeat 4 { ... }` — runs 4 times, no counter
- `for i in range(4) { ... }` — runs 4 times, with `i` counting up

Use `for` when you might need the counter, or when it reads more clearly.

## Recognizing patterns

Look at the staircase — each step is the same: move right, grab gem, move down. That's a perfect loop body!

## Your mission

Descend the staircase by repeating the step pattern with a `for` loop. Collect all the gems!
"#.to_string()),
    }
}

// ─── Puzzle 12 ─────────────────────────────────────────────────────────────

/// Helper Bot — teaches fn definitions
fn puzzle_12_helper_bot() -> PuzzleConfig {
    // 8×1 corridor with two gem clusters
    let mut grid = Grid::new(8, 1);
    grid.set_tile(Position::new(1, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(2, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(5, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(6, 0), Tile::floor().with_gem());
    grid.set_tile(Position::new(7, 0), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-12".to_string(),
        title: "Helper Bot".to_string(),
        description: "Two clusters of gems! Define a function to collect a pair, then reuse it.".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 0 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 7 },
            PuzzleObjective::MaxSteps { steps: 16 },
        ],
        starter_code: r#"// Define a function to avoid repeating yourself!
// fn my_function() { ... }
// Then call it: my_function()

move("right")
"#.to_string(),
        hint: Some("Define fn collect_pair() { move(\"right\") grab() move(\"right\") grab() } and call it twice with moves between.".to_string()),
        tutorial: Some(r#"# Functions

When you have a sequence of steps you want to **reuse**, define a function.

## Defining a function

```
fn collect_pair() {
    move("right")
    grab()
    move("right")
    grab()
}
```

This creates a new command called `collect_pair` but doesn't run it yet.

## Calling a function

```
collect_pair()
```

Now the code inside runs. You can call it as many times as you want!

## Functions with parameters

Functions can also take inputs:

```
fn walk(n) {
    repeat n { move("right") }
}
walk(3)   // moves right 3 times
walk(5)   // moves right 5 times
```

## Your mission

There are two clusters of gems with a gap between them. Define a function to collect a pair of gems, then reuse it!
"#.to_string()),
    }
}

// ─── Puzzle 13 ─────────────────────────────────────────────────────────────

/// Maze Runner — loops + sensors in a simple maze
fn puzzle_13_maze_runner() -> PuzzleConfig {
    // 7×7 simple maze, right-hand-rule solvable
    let mut grid = wall_grid(7, 7);

    // Carve a path through the maze
    // Start area
    grid.set_tile(Position::new(0, 0), Tile::floor());
    grid.set_tile(Position::new(1, 0), Tile::floor());
    grid.set_tile(Position::new(2, 0), Tile::floor());
    grid.set_tile(Position::new(3, 0), Tile::floor());
    grid.set_tile(Position::new(4, 0), Tile::floor());

    // Down from (4,0)
    grid.set_tile(Position::new(4, 1), Tile::floor());
    grid.set_tile(Position::new(4, 2), Tile::floor());

    // Left along row 2
    grid.set_tile(Position::new(3, 2), Tile::floor());
    grid.set_tile(Position::new(2, 2), Tile::floor().with_gem());

    // Down from (2,2)
    grid.set_tile(Position::new(2, 3), Tile::floor());
    grid.set_tile(Position::new(2, 4), Tile::floor().with_gem());

    // Right along row 4
    grid.set_tile(Position::new(3, 4), Tile::floor());
    grid.set_tile(Position::new(4, 4), Tile::floor());
    grid.set_tile(Position::new(5, 4), Tile::floor());
    grid.set_tile(Position::new(6, 4), Tile::floor().with_gem());

    // Down from (6,4) to goal
    grid.set_tile(Position::new(6, 5), Tile::floor());
    grid.set_tile(Position::new(6, 6), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-13".to_string(),
        title: "Maze Runner".to_string(),
        description: "Navigate the maze and collect all gems! Use sensors to find your way."
            .to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 6, y: 6 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 12 },
            PuzzleObjective::MaxSteps { steps: 30 },
        ],
        starter_code: r#"// Navigate the maze!
// Use path_ahead(), wall_ahead(), gem_here()
// Combine while loops with if statements

move("right")
"#
        .to_string(),
        hint: Some(
            "You can solve it step by step, or try a while loop with wall detection to navigate."
                .to_string(),
        ),
        tutorial: Some(
            r#"# Putting It All Together: Maze

Time to combine everything you've learned!

## Your toolbox

| Tool | What it does |
|---|---|
| `move(dir)` | Move one tile in a direction |
| `turn(dir)` | Turn left or right |
| `grab()` | Pick up a gem |
| `repeat N { }` | Run code N times |
| `while cond { }` | Loop while condition is true |
| `if cond { }` | Run code only if condition is true |
| `for i in range(N) { }` | Loop with counter |
| `fn name() { }` | Define a reusable function |

## Sensors

| Sensor | Returns `true` when... |
|---|---|
| `path_ahead()` | Tile ahead is walkable |
| `wall_ahead()` | Tile ahead is a wall |
| `gem_here()` | Gem on current tile |
| `facing()` | Returns current direction |

## Strategy

Break the maze into sections. Navigate each section, checking for gems along the way.

## Your mission

Navigate the maze, collect all gems, and reach the goal!
"#
            .to_string(),
        ),
    }
}

// ─── Puzzle 14 ─────────────────────────────────────────────────────────────

/// The Grand Tour — final challenge combining all concepts
fn puzzle_14_the_grand_tour() -> PuzzleConfig {
    // 8×8 complex winding path with gems
    let mut grid = wall_grid(8, 8);

    // Top corridor: y=0, x=0..6
    for x in 0..7 {
        grid.set_tile(Position::new(x, 0), Tile::floor());
    }
    grid.set_tile(Position::new(3, 0), Tile::floor().with_gem());

    // Down right side: x=6, y=0..4
    for y in 0..5 {
        grid.set_tile(Position::new(6, y), Tile::floor());
    }
    grid.set_tile(Position::new(6, 2), Tile::floor().with_gem());

    // Left along y=4: x=2..6
    for x in 2..7 {
        grid.set_tile(Position::new(x, 4), Tile::floor());
    }
    grid.set_tile(Position::new(4, 4), Tile::floor().with_gem());

    // Down from (2,4): y=4..6
    grid.set_tile(Position::new(2, 5), Tile::floor());
    grid.set_tile(Position::new(2, 6), Tile::floor().with_gem());

    // Right along y=6: x=2..7
    for x in 2..8 {
        grid.set_tile(Position::new(x, 6), Tile::floor());
    }
    grid.set_tile(Position::new(5, 6), Tile::floor().with_gem());

    // Down to goal: x=7, y=6..7
    grid.set_tile(Position::new(7, 6), Tile::floor());
    grid.set_tile(Position::new(7, 7), Tile::goal());

    PuzzleConfig {
        puzzle_id: "puzzle-14".to_string(),
        title: "The Grand Tour".to_string(),
        description: "The ultimate challenge! Navigate a winding path, collect all gems, and reach the goal. Use everything you've learned!".to_string(),
        grid,
        bot_start: BotState::new(Position::new(0, 0), Direction::Right),
        completion: PuzzleObjective::ReachPosition { x: 7, y: 7 },
        star_objectives: vec![
            PuzzleObjective::CollectAllGems,
            PuzzleObjective::MaxInstructions { instructions: 15 },
            PuzzleObjective::MaxSteps { steps: 40 },
        ],
        starter_code: r#"// The Grand Tour — combine everything!
// move(), turn(), grab(), repeat, while, if, for, fn
// Use sensors: path_ahead(), wall_ahead(), gem_here(), facing()

move("right")
"#.to_string(),
        hint: Some("Break it into sections: top corridor, down the right side, left along the middle, and so on.".to_string()),
        tutorial: Some(r#"# The Grand Tour

This is the final challenge — a winding path with gems scattered throughout.

## Strategy tips

1. **Break it into sections** — look at the path and identify straight segments
2. **Use functions** — if you repeat a pattern (like "walk and grab"), make it a function
3. **Use loops** — `repeat` for known distances, `while` for sensor-based movement
4. **Check for gems** — use `if gem_here() { grab() }` in your loops

## Example approach

```
fn walk(n) {
    repeat n {
        move(facing())
        if gem_here() { grab() }
    }
}

walk(6)
turn("right")
walk(4)
// ... continue for each section
```

The `facing()` sensor returns your current direction, so `move(facing())` always moves forward.

## Your mission

Navigate the entire winding path, collect all gems, and reach the goal. Use everything you've learned!
"#.to_string()),
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
