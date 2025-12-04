# Terrain Variant Type Guide

## Overview

Added `variant_type` field to terrain elements for displaying different textures/appearances on the frontend while maintaining the same backend logic.

## What Changed

### Backend Changes

#### 1. Data Structures (`src/map.rs`)

**`StaticTerrain`** - Added variant_type field:
```rust
pub struct StaticTerrain {
    pub name: String,
    pub core_position: Position,
    pub positions: Vec<Position>,
    pub terrain_type: String,     // "Mountain", "Water", "Forest", "Plain"
    pub variant_type: String,      // NEW: "mountain", "river", "lake", "forest", etc.
}
```

**`Terrain`** - Added variant_type field:
```rust
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub name: String,
    pub variant_type: String,      // NEW: specific variant for frontend display
}
```

#### 2. API Types (`src/api_types.rs`)

**`MapElementDetails::Terrain`** - Now includes variant_type:
```rust
Terrain {
    terrain_type: String,
    variant_type: String  // NEW
}
```

#### 3. Default Map (`src/map.rs:763-796`)

Current terrain configuration:
```rust
terrains: vec![
    StaticTerrain {
        name: "太行山",
        terrain_type: "Mountain",
        variant_type: "mountain",  // Mountain variant
        // ...
    },
    StaticTerrain {
        name: "玄水湖",
        terrain_type: "Water",
        variant_type: "lake",      // Lake variant
        // ...
    },
]
```

### Frontend Changes

#### 1. TypeScript Types (`frontend/src/api/gameApi.ts`)

Added variant_type to MapElement details:
```typescript
export interface MapElement {
  // ...
  details: {
    terrain_type?: string;
    variant_type?: string;  // NEW: terrain variant
    // ...
  };
}
```

#### 2. Icon Display (`frontend/src/MapView.tsx`)

Updated `getElementIcon()` to use variant_type:
```typescript
case 'Terrain': {
  const variantType = details?.variant_type;

  // Priority: use variant_type
  if (variantType === 'mountain') return '⛰️';
  if (variantType === 'river') return '🌊';
  if (variantType === 'lake') return '💧';
  if (variantType === 'forest') return '🌲';

  // Fallback to terrain_type
  // ...
}
```

#### 3. Visual Styling (`frontend/src/MapView.css`)

Added terrain variant CSS classes:
```css
.tile-terrain-mountain {
  background: linear-gradient(135deg, #a0aec0 0%, #718096 100%);
  border-color: #4a5568;
}

.tile-terrain-river {
  background: linear-gradient(135deg, #81e6d9 0%, #4fd1c5 100%);
  border-color: #38b2ac;
}

.tile-terrain-lake {
  background: linear-gradient(135deg, #90cdf4 0%, #63b3ed 100%);
  border-color: #3182ce;
}

.tile-terrain-forest {
  background: linear-gradient(135deg, #9ae6b4 0%, #68d391 100%);
  border-color: #38a169;
}
```

#### 4. Element Details Display

Added variant display in terrain details panel:
```typescript
<div className="detail-row">
  <span className="detail-label">变体:</span>
  <span className="detail-value">
    {details.variant_type === 'mountain' && '山脉 ⛰️'}
    {details.variant_type === 'river' && '河流 🌊'}
    {details.variant_type === 'lake' && '湖泊 💧'}
    {details.variant_type === 'forest' && '森林 🌲'}
  </span>
</div>
```

## Available Terrain Variants

### Current Variants

| terrain_type | variant_type | Icon | Size | Color Scheme | Use Case |
|--------------|--------------|------|------|--------------|----------|
| **Mountain** | small_mountain | 🗻 | 1x1 | Light gray | Small peaks, hills |
| **Mountain** | mid_mountain | ⛰️ | 1x2 | Medium gray | Mountain ranges |
| **Mountain** | large_mountain | 🏔️ | 2x2 | Dark gray | Massive mountains |
| **Mountain** | mountain | ⛰️ | Any | Gray gradient | Generic mountain |
| **Water** | river | 🌊 | Sequential | Teal gradient | Flowing rivers (can turn/branch) |
| **Water** | small_lake | 💧 | 1x1 | Light blue | Small ponds |
| **Water** | large_lake | 🏞️ | 2x2 | Deep blue | Large lakes |
| **Water** | lake | 💧 | Any | Blue gradient | Generic water body |
| **Forest** | forest | 🌲 | Any | Green gradient | Forest areas |

### Adding New Variants

To add a new terrain variant:

1. **Backend** - Set variant_type in `create_default_static_map()`:
   ```rust
   StaticTerrain {
       name: "Your Terrain Name",
       terrain_type: "Water",
       variant_type: "pond",  // New variant
       // ...
   }
   ```

2. **Frontend Icon** - Update `getElementIcon()` in `MapView.tsx`:
   ```typescript
   if (variantType === 'pond') return '🏞️';
   ```

3. **Frontend Styling** - Add CSS class in `MapView.css`:
   ```css
   .tile-terrain-pond {
     background: linear-gradient(135deg, #b3e5fc 0%, #81d4fa 100%);
     border-color: #29b6f6;
   }
   ```

4. **Frontend Logic** - Update `getElementColorClass()` in `MapView.tsx`:
   ```typescript
   if (variantType === 'pond') return 'tile-terrain-pond';
   ```

5. **Details Display** - Add to `renderElementDetails()` in `MapView.tsx`:
   ```typescript
   {details.variant_type === 'pond' && '池塘 🏞️'}
   ```

## Example Use Cases

### Different Mountain Types
```rust
// Snowy peak
variant_type: "snow_mountain"  // Could show ❄️⛰️

// Volcano
variant_type: "volcano"  // Could show 🌋

// Rocky hills
variant_type: "hills"  // Could show ⛰️ with different color
```

### Different Water Bodies
```rust
// Straight river (vertical or horizontal flow)
variant_type: "river"  // Shows 🌊
// Example: 清流河 - flows straight down

// L-shaped river (turns once)
variant_type: "river"  // Shows 🌊
// Example: 碧水江 - flows down then turns right

// Zigzag river (alternating turns)
variant_type: "river"  // Shows 🌊
// Example: 九曲溪 - alternates direction creating zigzag

// Calm lake
variant_type: "lake"  // Shows 💧

// Ocean
variant_type: "ocean"  // Could show 🌊 with darker blue

// Waterfall
variant_type: "waterfall"  // Could show 💦
```

### Different Forest Types
```rust
// Dense forest
variant_type: "forest"  // Shows 🌲

// Bamboo grove
variant_type: "bamboo"  // Could show 🎋

// Cherry blossom grove
variant_type: "cherry_blossom"  // Could show 🌸
```

## Backend Logic Independence

**Important**: The `variant_type` field is **purely visual** and does not affect backend game logic:

- ✅ All terrain with `terrain_type: "Water"` is impassable (regardless of variant)
- ✅ All terrain with `terrain_type: "Mountain"` is impassable (regardless of variant)
- ✅ Collision detection uses `terrain_type`, not `variant_type`
- ✅ Game mechanics treat all variants of the same terrain_type identically

This design allows for rich visual variety without complicating game logic.

## Visual Preview

### Current Default Map Terrain Display:

**Mountains (Gray)**
- 小石峰 (small_mountain, 1x1) - 🗻 with light gray gradient
- 太行山 (mid_mountain, 1x2) - ⛰️ with medium gray gradient
- 昆仑山 (large_mountain, 2x2) - 🏔️ with dark gray gradient

**Water (Blue/Teal)**
- 清泉小潭 (small_lake, 1x1) - 💧 with light blue gradient
- 玄水湖 (large_lake, 2x2) - 🏞️ with deep blue gradient
- 清流河 (river, straight vertical) - 🌊 with teal gradient (4 segments)
- 碧水江 (river, L-shaped) - 🌊 with teal gradient (5 segments)
- 九曲溪 (river, zigzag) - 🌊 with teal gradient (5 segments)

**Forest (Green)**
- 青松林 (forest, 1x1) - 🌲 with green gradient

## Benefits

1. **Visual Variety**: Multiple appearance options for same terrain type
2. **Easy Theming**: Can create themed maps (desert, tundra, tropical, etc.)
3. **No Logic Impact**: Backend behavior remains consistent
4. **Scalable**: Easy to add new variants without code changes
5. **Backwards Compatible**: Falls back to terrain_type if variant_type missing

## Future Enhancements

Potential additions:
- **Animated variants**: River flowing animation, swaying trees
- **Seasonal variants**: Snow-covered in winter, lush in summer
- **Biome-specific variants**: Desert mountains, jungle rivers, etc.
- **Dynamic variant assignment**: Change based on game events or time
- **Texture images**: Replace emojis with actual texture sprites
