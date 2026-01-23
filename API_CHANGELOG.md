# API Changelog

## Changes from commit 729066a to current

### ⚠️ Breaking Changes

#### Modified Response Types (New Required Fields)

**TaskResultDto** - Added required fields:
```typescript
{
  // ... existing fields
  disciple_name: string;  // NEW - Name of the disciple
  disciple_died: bool;    // NEW - Whether disciple died during task
}
```

**UsePillResponse** - Added required fields:
```typescript
{
  // ... existing fields
  progress_before: number;  // NEW - Cultivation progress before pill use
  progress_after: number;   // NEW - Cultivation progress after pill use
}
```

**MoveDiscipleResponse** - Added optional field:
```typescript
{
  // ... existing fields
  collected_herb?: {        // NEW - Herb collected during movement
    name: string;
    quality: string;
  }
}
```

**MapElementDetails** - Added new variant:
```typescript
enum MapElementDetails {
  // ... existing variants
  Herb {                   // NEW - Herb nodes on map
    herb_id: string;
    quality: string;
    growth_stage: number;
    max_growth: number;
    is_mature: boolean;
  }
}
```

---

### New Endpoints

#### Herb System
- **GET `/api/game/:game_id/herbs`** - Get herb inventory
  - Returns: `HerbInventoryResponse`
  ```typescript
  {
    total_count: number;
    herbs: Array<{
      name: string;
      quality: string;
      count: number;
    }>;
  }
  ```
  
#### Alchemy System  
- **GET `/api/game/:game_id/recipes`** - Get all pill recipes
  - Returns: `AllRecipesResponse`
  ```typescript
  {
    recipes: Array<{
      pill_type: string;
      name: string;
      description: string;
      required_herb_quality: string;
      required_herb_count: number;
      resource_cost: number;
      success_rate: number;
      output_count: number;
      can_craft: boolean;
      reason?: string;
    }>;
  }
  ```
  
- **POST `/api/game/:game_id/refine`** - Refine pills from herbs
  - Request: `{ pill_type: string }`
  - Response:
  ```typescript
  {
    success: boolean;
    message: string;
    pill_name?: string;
    output_count?: number;
  }
  ```

#### Task Eligibility
- **POST `/api/game/:game_id/tasks/check-eligibility`** - Check if disciple is eligible for task
  - Request: `{ task_id: number, disciple_id: number }`
  - Response:
  ```typescript
  {
    task_id: number;
    task_name: string;
    disciple_id: number;
    disciple_name: string;
    eligible: boolean;
    reason?: string;
    success_rate?: number;      // For combat tasks (0.0 - 1.0)
    disciple_combat_level?: number;
    enemy_level?: number;
  }
  ```

---

### Migration Guide

#### For Frontend Developers

1. **TaskResultDto Changes**
   ```typescript
   // Before
   interface TaskResult {
     task_id: number;
     disciple_id: number;
     success: boolean;
     rewards?: TaskRewards;
     message: string;
   }
   
   // After
   interface TaskResult {
     task_id: number;
     disciple_id: number;
     disciple_name: string;      // NEW
     success: boolean;
     rewards?: TaskRewards;
     message: string;
     disciple_died: boolean;     // NEW
   }
   ```

2. **UsePillResponse Changes**
   ```typescript
   // Before
   interface UsePillResponse {
     success: boolean;
     message: string;
     disciple_name: string;
     energy_before: number;
     energy_after: number;
     constitution_before: number;
     constitution_after: number;
   }
   
   // After - Add progress tracking
   interface UsePillResponse {
     // ... all previous fields
     progress_before: number;    // NEW
     progress_after: number;     // NEW
   }
   ```

3. **MoveDiscipleResponse Changes**
   ```typescript
   // After - Handle optional herb collection
   interface MoveDiscipleResponse {
     // ... existing fields
     collected_herb?: {          // NEW
       name: string;
       quality: string;
     }
   }
   ```

4. **MapElementDetails - Handle new Herb variant**
   ```typescript
   type MapElementDetails = 
     | { type: 'Village', ... }
     | { type: 'Faction', ... }
     | { type: 'Herb',          // NEW
         herb_id: string,
         quality: string,
         growth_stage: number,
         max_growth: number,
         is_mature: boolean }
     | ...
   ```

---

### Feature Summary

✅ **Herb Collection System** - Disciples can collect herbs from the map  
✅ **Alchemy System** - Refine herbs into pills with recipes  
✅ **Enhanced Combat** - Check task eligibility and success rates  
✅ **Death Tracking** - Track disciple deaths in task results  
✅ **Cultivation Progress** - Track cultivation changes from pills  

