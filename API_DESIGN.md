# 修仙模拟器 HTTP API 设计

## 基础信息

- **协议**: HTTP/1.1
- **数据格式**: JSON
- **端口**: 3000
- **CORS**: 允许所有来源（开发环境）

## API 端点

### 1. 游戏管理

#### 创建新游戏
```
POST /api/game/new
Content-Type: application/json

Request:
{
  "sect_name": "青云宗"
}

Response:
{
  "game_id": "uuid-string",
  "sect_name": "青云宗",
  "year": 0,
  "disciples_count": 3
}
```

#### 获取游戏状态
```
GET /api/game/{game_id}

Response:
{
  "game_id": "uuid-string",
  "sect": {
    "name": "青云宗",
    "year": 5,
    "resources": 2450,
    "reputation": 350,
    "disciples_count": 8
  },
  "state": "Running" | "Victory" | "Defeat"
}
```

### 2. 回合管理

#### 开始新回合
```
POST /api/game/{game_id}/turn/start

Response:
{
  "year": 6,
  "events": [
    {
      "type": "Income",
      "data": { "amount": 290 }
    },
    {
      "type": "NewDisciple",
      "data": { "name": "风清扬", "type": "Outer" }
    }
  ],
  "tasks": [...],
  "disciples": [...]
}
```

#### 结束回合（执行任务）
```
POST /api/game/{game_id}/turn/end

Request:
{
  "assignments": [
    { "task_id": 1, "disciple_id": 5 },
    { "task_id": 2, "disciple_id": 3 }
  ]
}

Response:
{
  "results": [
    {
      "task_id": 1,
      "disciple_id": 5,
      "success": true,
      "rewards": {
        "progress": 15,
        "resources": 40,
        "reputation": 25
      }
    }
  ],
  "game_state": "Running"
}
```

### 3. 弟子管理

#### 获取所有弟子
```
GET /api/game/{game_id}/disciples

Response:
{
  "disciples": [
    {
      "id": 1,
      "name": "云飞扬",
      "type": "Inner",
      "cultivation": {
        "level": "Foundation",
        "progress": 65
      },
      "age": 85,
      "lifespan": 300,
      "dao_heart": 75,
      "talents": [
        { "type": "Fire", "level": 7 },
        { "type": "Sword", "level": 8 }
      ],
      "current_task": "讨伐噬魂虎" | null
    }
  ]
}
```

#### 获取单个弟子详情
```
GET /api/game/{game_id}/disciples/{disciple_id}

Response:
{
  "id": 1,
  "name": "云飞扬",
  "type": "Inner",
  "cultivation": {
    "level": "Foundation",
    "progress": 65,
    "completed_tasks": [1, 2, 5]
  },
  "age": 85,
  "lifespan": 300,
  "dao_heart": 75,
  "talents": [...],
  "heritage": {
    "name": "xxx的传承",
    "tribulation_bonus": 0.1
  } | null,
  "current_task": "讨伐噬魂虎" | null
}
```

### 4. 任务管理

#### 获取当前可用任务
```
GET /api/game/{game_id}/tasks

Response:
{
  "tasks": [
    {
      "id": 1,
      "name": "讨伐噬魂虎",
      "type": "Combat",
      "rewards": {
        "progress": 15,
        "resources": 40,
        "reputation": 25
      },
      "dao_heart_impact": 3,
      "suitable_disciples": {
        "free": [1, 3, 5],
        "busy": [2]
      },
      "assigned_to": 1 | null
    }
  ]
}
```

#### 分配任务
```
POST /api/game/{game_id}/tasks/{task_id}/assign

Request:
{
  "disciple_id": 5
}

Response:
{
  "success": true,
  "task_id": 1,
  "disciple_id": 5,
  "message": "已将任务分配给云飞扬"
}
```

#### 取消任务分配
```
DELETE /api/game/{game_id}/tasks/{task_id}/assign

Response:
{
  "success": true,
  "message": "已取消任务分配"
}
```

#### 自动分配所有任务
```
POST /api/game/{game_id}/tasks/auto-assign

Response:
{
  "assigned_count": 5,
  "assignments": [
    { "task_id": 1, "disciple_id": 3 },
    { "task_id": 2, "disciple_id": 5 }
  ]
}
```

### 5. 渡劫管理

#### 获取可渡劫弟子
```
GET /api/game/{game_id}/tribulation/candidates

Response:
{
  "candidates": [
    {
      "disciple_id": 3,
      "name": "云飞扬",
      "current_level": "Foundation",
      "success_rate": 0.625,
      "dao_heart": 75,
      "heritage_bonus": 0.1
    }
  ]
}
```

#### 执行渡劫
```
POST /api/game/{game_id}/tribulation

Request:
{
  "disciple_id": 3
}

Response:
{
  "success": true,
  "disciple_id": 3,
  "name": "云飞扬",
  "new_level": "GoldenCore",
  "message": "云飞扬渡劫成功，晋升至结丹期！"
}
```

### 6. 统计信息

#### 获取宗门统计
```
GET /api/game/{game_id}/statistics

Response:
{
  "year": 34,
  "total_disciples": 21,
  "disciples_by_type": {
    "outer": 12,
    "inner": 9,
    "personal": 0
  },
  "resources": 8510,
  "reputation": 2985,
  "cultivation_distribution": {
    "QiRefining": 18,
    "Foundation": 2,
    "Ascension": 1
  }
}
```

## WebSocket 端点（可选）

### 实时游戏更新
```
WS /ws/game/{game_id}

Messages:
{
  "type": "TurnStarted",
  "data": { ... }
}

{
  "type": "TaskCompleted",
  "data": { ... }
}

{
  "type": "DiscipleDied",
  "data": { ... }
}
```

## 错误响应

所有错误响应格式：
```json
{
  "error": {
    "code": "GAME_NOT_FOUND",
    "message": "游戏不存在",
    "details": "Game ID: xxx not found"
  }
}
```

### 错误码列表
- `GAME_NOT_FOUND` - 游戏不存在
- `DISCIPLE_NOT_FOUND` - 弟子不存在
- `TASK_NOT_FOUND` - 任务不存在
- `DISCIPLE_BUSY` - 弟子已被分配任务
- `INVALID_ASSIGNMENT` - 无效的任务分配
- `GAME_ENDED` - 游戏已结束
- `INTERNAL_ERROR` - 服务器内部错误

## 数据模型

### DiscipleType
```typescript
type DiscipleType = "Outer" | "Inner" | "Personal";
```

### CultivationLevel
```typescript
type CultivationLevel =
  | "QiRefining"
  | "Foundation"
  | "GoldenCore"
  | "NascentSoul"
  | "SpiritSevering"
  | "VoidRefinement"
  | "Ascension";
```

### TalentType
```typescript
type TalentType =
  | "Fire" | "Water" | "Wood" | "Metal" | "Earth"
  | "Thunder" | "Ice" | "Wind"
  | "Sword" | "Alchemy" | "Formation" | "Beast" | "Medical";
```

### TaskType
```typescript
type TaskType =
  | { type: "Gathering", data: { resource_type: string, difficulty: number } }
  | { type: "Combat", data: { enemy_name: string, enemy_level: number } }
  | { type: "Exploration", data: { location: string, danger_level: number } }
  | { type: "Auxiliary", data: { task_name: string } }
  | { type: "Investment", data: { resource_cost: number } };
```

### GameState
```typescript
type GameState = "Running" | "Victory" | "Defeat";
```
