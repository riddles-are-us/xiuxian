# Static Map System Guide

## Overview

The game now uses a **static map** system for early access instead of procedurally generated maps. All the old procedural generation logic has been preserved in comments for future reference.

## Core Position vs Collision Positions

The map system now supports **multi-tile elements** with two types of positions:

1. **`core_position`** - The interaction point (e.g., village entrance, faction gate)
   - This is where disciples interact with the element
   - Tasks are assigned to this position
   - Only one core position per element

2. **`positions`** - The collision/occupied space (all tiles the element occupies)
   - These tiles are impassable (except for monsters)
   - Terrain is completely impassable
   - Buildings block movement but disciples can interact at the core position

### Example:
A 3x3 faction might have:
- `core_position`: (10, 15) - the main gate
- `positions`: [(9,14), (10,14), (11,14), (9,15), (10,15), (11,15), (9,16), (10,16), (11,16)] - all 9 tiles occupied

## What Changed

### New Static Map Structures

The following new data structures are available in `src/map.rs`:

1. **`StaticMapData`** - Main container for all static map data
   - `width`, `height` - Map dimensions
   - `villages` - List of villages with positions
   - `factions` - List of factions with positions
   - `dangerous_locations` - List of dangerous locations
   - `secret_realms` - List of secret realms
   - `monsters` - List of initial monsters
   - `terrains` - List of terrain features

2. **Individual Element Structures**:
   - `StaticVillage` - Name, core_position, positions (Vec), population, prosperity
   - `StaticFaction` - Name, core_position, positions (Vec), power_level, relationship
   - `StaticDangerousLocation` - Name, core_position, positions (Vec), danger_level
   - `StaticSecretRealm` - Name, core_position, positions (Vec), realm_type (string), difficulty
   - `StaticMonster` - Name, core_position (single tile, mobile), level, is_demon
   - `StaticTerrain` - Name, core_position, positions (Vec), terrain_type (string)

### New Map Initialization Method

**`GameMap::initialize_from_static(static_data: StaticMapData)`**

This method replaces the old `initialize()` method and sets up the map from static data.

### Default Map

**`GameMap::create_default_static_map()`**

This function creates a default static map with sample data:
- 2 villages: 青云村 (1x1), 桃花村 (1x1)
- 1 faction: 天剑宗 (2x2) - **Factions are always 2x2**
- 1 dangerous location: 黑风谷 (1x1) - **Dangerous locations are always 1x1**
- 1 secret realm: 烈焰洞天 (1x1)
- 1 monster: 山野妖兽 (1x1, mobile)
- 4 terrain features: 太行山 (1x1 or 1x2, random), 昆仑山 (1x1 or 1x2, random), 玄水湖 (1x1), 青松林 (1x1)
  - **Mountains are randomly either 1x1 or 1x2 (horizontal)**

## How to Customize the Map

### Option 1: Modify the Default Map Function

Edit the `create_default_static_map()` function in `src/map.rs` (lines 670-777):

```rust
pub fn create_default_static_map() -> StaticMapData {
    StaticMapData {
        width: 20,
        height: 20,
        villages: vec![
            StaticVillage {
                name: "Your Village Name".to_string(),
                core_position: Position { x: 5, y: 5 },  // Village position
                // Village occupies single tile
                positions: vec![Position { x: 5, y: 5 }],
                population: 1000,
                prosperity: 50,
            },
            // Add more villages...
        ],
        factions: vec![
            StaticFaction {
                name: "Your Faction Name".to_string(),
                core_position: Position { x: 10, y: 15 },  // Main gate
                // Faction occupies 2x2 area (ALWAYS 2x2)
                positions: vec![
                    Position { x: 10, y: 15 }, Position { x: 11, y: 15 },
                    Position { x: 10, y: 16 }, Position { x: 11, y: 16 },
                ],
                power_level: 50,
                relationship: 20,  // -100 to 100
            },
            // Add more factions...
        ],
        dangerous_locations: vec![
            StaticDangerousLocation {
                name: "Your Dangerous Place".to_string(),
                core_position: Position { x: 3, y: 12 },
                // Dangerous locations are ALWAYS 1x1
                positions: vec![Position { x: 3, y: 12 }],
                danger_level: 30,
            },
        ],
        terrains: vec![
            StaticTerrain {
                name: "Mountain Range".to_string(),
                core_position: Position { x: 2, y: 2 },
                // Mountains are 1x1 or 1x2 (horizontal)
                positions: vec![
                    Position { x: 2, y: 2 },
                    Position { x: 3, y: 2 },  // 1x2 horizontal
                ],
                terrain_type: "Mountain".to_string(),
            },
            StaticTerrain {
                name: "Small Peak".to_string(),
                core_position: Position { x: 5, y: 3 },
                // Or just 1x1
                positions: vec![Position { x: 5, y: 3 }],
                terrain_type: "Mountain".to_string(),
            },
        ],
        // ... continue with other elements
    }
}
```

### Option 2: Create Custom Maps Programmatically

In `src/game.rs` or `src/interactive.rs`, you can create custom map data:

```rust
// Example custom map
let custom_map = StaticMapData {
    width: 30,
    height: 30,
    villages: vec![
        StaticVillage {
            name: "起始村".to_string(),
            core_position: Position { x: 10, y: 10 },
            positions: vec![Position { x: 10, y: 10 }],  // Single tile village
            population: 500,
            prosperity: 30,
        },
    ],
    terrains: vec![
        StaticTerrain {
            name: "Impassable Mountains".to_string(),
            core_position: Position { x: 5, y: 5 },
            positions: vec![
                Position { x: 5, y: 5 }, Position { x: 6, y: 5 },  // 2x1 mountain range
            ],
            terrain_type: "Mountain".to_string(),
        },
    ],
    monsters: vec![],
    factions: vec![],
    dangerous_locations: vec![],
    secret_realms: vec![],
};

map.initialize_from_static(custom_map);
```

## Element Size Guidelines

**Fixed Sizes:**
- **Factions**: Always 2x2
- **Dangerous Locations**: Always 1x1
- **Villages**: 1x1 (can be customized)
- **Secret Realms**: 1x1 (can be customized)
- **Monsters**: 1x1 (mobile)

**Variable Sizes:**
- **Mountains**: 1x1 or 1x2 (horizontal), randomly chosen at map creation
- **Other Terrain**: Any size (Water, Forest, Plain)

## Collision Detection

The map system automatically checks for position collisions when initializing from static data.

**What it checks:**
- No two elements can occupy the same position
- All positions in the `positions` array are checked
- Monsters are checked at their core position

**Behavior:**
- If a collision is detected, a warning is printed to console
- The map will still load, but overlapping elements may cause issues
- **Recommendation**: Always ensure your element positions don't overlap

**Example Warning:**
```
⚠ 地图初始化警告: 位置冲突: 势力 '天剑宗' 在位置 (10, 15) 与其他元素重叠
   建议: 请调整元素位置以避免重叠
```

## Map Element Properties

### Villages
- **name**: Village name (String)
- **core_position**: Entrance/interaction point (x, y coordinates)
- **positions**: All tiles occupied by the village (Vec<Position>)
- **population**: Number of inhabitants (affects income)
- **prosperity**: Wealth level (affects income and available tasks)
- Note: Disciples interact at core_position, but all positions block movement

### Factions
- **name**: Faction name (String)
- **core_position**: Main gate/interaction point
- **positions**: All tiles occupied by the faction (Vec<Position>)
- **power_level**: Strength level (affects task difficulty)
- **relationship**: -100 (hostile) to 100 (friendly)
  - ≥ 0: Generates friendly tasks
  - < -30: Generates hostile combat tasks

### Dangerous Locations
- **name**: Location name (String)
- **core_position**: Entrance/interaction point
- **positions**: All tiles occupied (Vec<Position>)
- **danger_level**: Difficulty of tasks generated

### Secret Realms
- **name**: Realm name (String)
- **core_position**: Entrance/interaction point
- **positions**: All tiles occupied (Vec<Position>)
- **realm_type**: "Fire", "Water", "Wood", "Metal", "Earth", "Sword", "Alchemy", "Formation", "Medical"
- **difficulty**: Task difficulty level

### Monsters
- **name**: Monster name (String)
- **core_position**: Current location (single tile, will move around)
- **level**: Strength level
- **is_demon**: Whether already transformed into demon
- Note: Monsters occupy only one tile and can move. They don't block movement.

### Terrains (IMPASSABLE)
- **name**: Terrain name (String)
- **core_position**: Center point of the terrain
- **positions**: All tiles occupied (Vec<Position>)
- **terrain_type**: "Mountain", "Water", "Forest", "Plain"
- **IMPORTANT**: Terrains are completely impassable. Disciples cannot move through terrain tiles.
- Note: Terrains are decorative and don't generate tasks

## Commented Out Features

The following procedural generation features are preserved in comments:

1. **`GameMap::initialize()`** (lines 839-914) - Config-based map loading with random terrain
2. **`GameMap::generate_terrain()`** (lines 917-943) - Random terrain generation
3. **Random monster spawning** in `GameMap::update()` (lines 1006-1034)

These can be uncommented and re-enabled for future procedural generation features.

## Task Templates

Task templates are still loaded from the config files (`game_config.json`). The static map only defines:
- What elements exist
- Where they are located
- Their basic properties

The actual tasks generated come from the config file's task templates.

## New Helper Methods

### Collision Detection

**`GameMap::is_position_passable(&self, pos: &Position) -> bool`**

Checks if a position is passable (for pathfinding and movement):
- Returns `false` if out of bounds
- Returns `false` if position occupied by terrain (impassable)
- Returns `false` if position occupied by buildings (village, faction, etc.)
- Returns `true` if position occupied by monster (can fight at same position)
- Returns `true` if position is empty

### Element Interaction

**`GameMap::get_elements_at_core_position(&self, pos: &Position) -> Vec<&MapElement>`**

Gets all elements at a specific core position (for interaction):
- Returns all elements whose `core_position` matches the given position
- Used for determining which elements disciples can interact with

### Position Queries

**`PositionedElement::occupies_position(&self, pos: &Position) -> bool`**

Checks if an element occupies a specific position (collision check).

**`PositionedElement::is_core_position(&self, pos: &Position) -> bool`**

Checks if a position is the element's core interaction point.

## Future Enhancements

When ready to re-enable procedural generation:
1. Uncomment the preserved code blocks
2. Add a flag to choose between static and procedural maps
3. Implement a hybrid system that combines static base elements with procedural events

## Current Game Behavior

- **Monster Movement**: Monsters move randomly, respecting terrain collision
  - Monsters cannot move through terrain (mountains, water, etc.)
  - Monsters can move to building locations (triggers invasion)
  - Monsters don't block each other
- **Monster Growth**: Monsters still grow and can become demons
- **Monster Spawning**: NOW DISABLED (was random spawning)
- **Defense Tasks**: Still generated when monsters invade locations
- **Task Generation**: Based on static map elements + config templates
- **Movement System**: All tiles occupied by buildings and terrain are impassable
