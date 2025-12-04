# Frontend Map Changes Summary

## Overview

The frontend has been updated to support the new multi-tile map system with `core_position` (interaction point) and `positions` (occupied tiles).

## Changes Made

### 1. API Types (`frontend/src/api/gameApi.ts`)

**Updated `MapElement` interface:**
```typescript
export interface MapElement {
  element_type: string;
  name: string;
  position: {  // Core position - for interaction
    x: number;
    y: number;
  };
  positions?: Array<{  // All occupied positions - for collision/rendering
    x: number;
    y: number;
  }>;
  details: { ... };
}
```

- `position` - The core position (entrance/interaction point)
- `positions` - Optional array of all occupied positions (for multi-tile elements)
- Backwards compatible: if `positions` is not provided, falls back to `position`

### 2. Map View Component (`frontend/src/MapView.tsx`)

**New Helper Functions:**

1. **`elementOccupiesPosition(element, x, y)`**
   - Checks if an element occupies a specific tile
   - Uses `positions` array if available, otherwise uses core `position`
   - Used for rendering and collision detection

2. **`getElementAt(x, y)`**
   - Gets any element occupying the position (for rendering)
   - Shows elements on all their occupied tiles

3. **`getElementAtCorePosition(x, y)`**
   - Gets element whose core position is at (x, y)
   - Used for interaction/task assignment

**Tile Click Behavior:**
- Prioritizes core position for interactions
- Falls back to occupied position for visual feedback
- Disciples interact with elements at their core position

**Visual Indicators:**
- Multi-tile elements show a ⭐ marker on their core position tile
- All occupied tiles display the element icon
- Core position is where tasks are assigned and interactions occur

### 3. Backend API Response (`src/web_server.rs`, `src/api_types.rs`)

**Updated `MapElementDto`:**
```rust
pub struct MapElementDto {
    pub element_type: String,
    pub name: String,
    pub position: PositionDto,       // Core position
    pub positions: Vec<PositionDto>,  // All occupied positions
    pub details: MapElementDetails,
}
```

The backend now sends both:
- `position` - The core position for interaction
- `positions` - Array of all occupied tiles

## How It Works

### Rendering Multi-Tile Elements

1. **Element Rendering:**
   - Elements are rendered on ALL tiles they occupy
   - Each occupied tile shows the element icon
   - The core position tile shows an additional ⭐ marker (for multi-tile elements)

2. **Interaction:**
   - Clicking on ANY occupied tile selects the element
   - But tasks are created at the **core position**
   - Disciples need to be at the core position to interact

3. **Example: 3x3 Faction**
   ```
   [Faction] [Faction] [Faction]
   [Faction] [Faction⭐] [Faction]  ← ⭐ marks core position (10, 15)
   [Faction] [Faction] [Faction]
   ```
   - All 9 tiles show faction icon
   - Center tile (10, 15) has star marker
   - Disciples interact at (10, 15)

### Collision Detection

The map now properly handles:
- **Terrain** - All occupied tiles are impassable
- **Buildings** - All occupied tiles block movement
- **Monsters** - Single tile, don't block movement
- **Villages** - Currently 1x1 (can be expanded)

### Backwards Compatibility

The system is backwards compatible:
- If `positions` array is not provided, it uses core `position`
- Old single-tile elements work without changes
- Frontend gracefully handles both formats

## Current Default Map Layout

Based on `create_default_static_map()`:

| Element | Size | Core Position | Notes |
|---------|------|---------------|-------|
| 青云村 | 1x1 | (5, 5) | Single tile village |
| 桃花村 | 1x1 | (15, 10) | Single tile village |
| 天剑宗 | 2x2 | (10, 15) | **Factions always 2x2** |
| 黑风谷 | 1x1 | (3, 12) | **Dangerous locations always 1x1** |
| 烈焰洞天 | 1x1 | (18, 18) | Single tile secret realm |
| 山野妖兽 | 1x1 | (7, 8) | Mobile monster |
| 太行山 | 1x1 or 1x2 | (2, 2) | **Random size, horizontal** |
| 昆仑山 | 1x1 or 1x2 | (17, 3) | **Random size, horizontal** |
| 玄水湖 | 1x1 | (12, 6) | Impassable terrain |
| 青松林 | 1x1 | (8, 1) | Impassable terrain |

### Element Size Rules:
- **Factions**: Always 2x2
- **Dangerous Locations**: Always 1x1
- **Mountains**: Randomly either 1x1 or 1x2 (horizontal) each time map is created
- **Other elements**: Size can be customized

### Collision Detection:
The backend automatically checks for position collisions when initializing the map:
- Warns if any two elements occupy the same position
- Prevents overlapping elements
- Checks all positions in the `positions` array
- Warning message displayed in server console if collision detected

## Testing the Frontend

To test the changes:

1. **Install dependencies:**
   ```bash
   cd frontend
   npm install
   ```

2. **Run development server:**
   ```bash
   npm start
   ```

3. **Verify:**
   - Multi-tile elements (faction, terrain) show on multiple tiles
   - Core positions show ⭐ marker
   - Clicking any occupied tile selects the element
   - Terrain blocks movement
   - Disciples can interact at core positions

## Future Enhancements

Potential improvements:
- Visual indication of occupied vs core tiles (different opacity)
- Pathfinding that respects terrain collision
- Multi-tile buildings for sect structures
- Dynamic element resizing based on prosperity/power
