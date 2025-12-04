# Static Map System Guide

## Overview

The game now uses a **static map** system for early access instead of procedurally generated maps. All the old procedural generation logic has been preserved in comments for future reference.

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
   - `StaticVillage` - Name, position, population, prosperity
   - `StaticFaction` - Name, position, power_level, relationship
   - `StaticDangerousLocation` - Name, position, danger_level
   - `StaticSecretRealm` - Name, position, realm_type (string), difficulty
   - `StaticMonster` - Name, position, level, is_demon
   - `StaticTerrain` - Name, position, terrain_type (string)

### New Map Initialization Method

**`GameMap::initialize_from_static(static_data: StaticMapData)`**

This method replaces the old `initialize()` method and sets up the map from static data.

### Default Map

**`GameMap::create_default_static_map()`**

This function creates a default static map with sample data:
- 2 villages (青云村, 桃花村)
- 1 faction (天剑宗)
- 1 dangerous location (黑风谷)
- 1 secret realm (烈焰洞天)
- 1 monster (山野妖兽)
- 3 terrain features (太行山, 玄水湖, 青松林)

## How to Customize the Map

### Option 1: Modify the Default Map Function

Edit the `create_default_static_map()` function in `src/map.rs` (lines 635-702):

```rust
pub fn create_default_static_map() -> StaticMapData {
    StaticMapData {
        width: 20,
        height: 20,
        villages: vec![
            StaticVillage {
                name: "Your Village Name".to_string(),
                position: Position { x: 5, y: 5 },
                population: 1000,
                prosperity: 50,
            },
            // Add more villages...
        ],
        factions: vec![
            StaticFaction {
                name: "Your Faction Name".to_string(),
                position: Position { x: 10, y: 15 },
                power_level: 50,
                relationship: 20,  // -100 to 100
            },
            // Add more factions...
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
            position: Position { x: 10, y: 10 },
            population: 500,
            prosperity: 30,
        },
    ],
    // ... add other elements
    monsters: vec![],
    terrains: vec![],
    factions: vec![],
    dangerous_locations: vec![],
    secret_realms: vec![],
};

map.initialize_from_static(custom_map);
```

## Map Element Properties

### Villages
- **name**: Village name (String)
- **position**: Location on map (x, y coordinates)
- **population**: Number of inhabitants (affects income)
- **prosperity**: Wealth level (affects income and available tasks)

### Factions
- **name**: Faction name (String)
- **position**: Location on map
- **power_level**: Strength level (affects task difficulty)
- **relationship**: -100 (hostile) to 100 (friendly)
  - ≥ 0: Generates friendly tasks
  - < -30: Generates hostile combat tasks

### Dangerous Locations
- **name**: Location name (String)
- **position**: Location on map
- **danger_level**: Difficulty of tasks generated

### Secret Realms
- **name**: Realm name (String)
- **position**: Location on map
- **realm_type**: "Fire", "Water", "Wood", "Metal", "Earth", "Sword", "Alchemy", "Formation", "Medical"
- **difficulty**: Task difficulty level

### Monsters
- **name**: Monster name (String)
- **position**: Starting location
- **level**: Strength level
- **is_demon**: Whether already transformed into demon

### Terrains
- **name**: Terrain name (String)
- **position**: Location on map
- **terrain_type**: "Mountain", "Water", "Forest", "Plain"
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

## Future Enhancements

When ready to re-enable procedural generation:
1. Uncomment the preserved code blocks
2. Add a flag to choose between static and procedural maps
3. Implement a hybrid system that combines static base elements with procedural events

## Current Game Behavior

- **Monster Movement**: Monsters still move randomly and can invade locations
- **Monster Growth**: Monsters still grow and can become demons
- **Monster Spawning**: NOW DISABLED (was random spawning)
- **Defense Tasks**: Still generated when monsters invade locations
- **Task Generation**: Based on static map elements + config templates
