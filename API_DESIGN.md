# 修仙模拟器 HTTP API 设计

## 基础信息

- **协议**: HTTP/1.1
- **数据格式**: JSON
- **端口**: 3000
- **CORS**: 允许所有来源（开发环境）
- **API 版本**: 1.0.7
- **框架**: Axum 0.6 + Tokio

## 通用响应格式

所有 API 响应都使用统一的包装格式：

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

错误时：
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述",
    "details": "详细信息"
  }
}
```

## API 端点

### 0. 系统信息

#### 获取 API 版本
```
GET /api/version

Response:
{
  "success": true,
  "data": {
    "version": "1.0.7",
    "name": "修仙模拟器"
  }
}
```

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
  "success": true,
  "data": {
    "game_id": "uuid-string",
    "sect": {
      "name": "青云宗",
      "year": 0,
      "resources": 1000,
      "reputation": 100,
      "disciples_count": 3
    },
    "state": "Running"
  }
}
```

#### 获取游戏状态
```
GET /api/game/{game_id}

Response:
{
  "success": true,
  "data": {
    "game_id": "uuid-string",
    "sect": {
      "name": "青云宗",
      "year": 5,
      "resources": 2450,
      "reputation": 350,
      "disciples_count": 8
    },
    "state": "Running"
  }
}

注：state 可能的值: "Running" | "Victory" | "Defeat"
```

### 2. 回合管理

#### 开始新回合
```
POST /api/game/{game_id}/turn/start

Response:
{
  "success": true,
  "data": {
    "year": 6,
    "events": [
      "新弟子风清扬加入宗门",
      "获得资源收入 290"
    ],
    "tasks": [...],  // 详见任务管理部分
    "disciples": [...]  // 详见弟子管理部分
  }
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

注：请求体中的 assignments 目前仅作为文档参数，实际任务分配通过任务分配 API 完成

Response:
{
  "success": true,
  "data": {
    "results": [],  // 任务结果处理在游戏内部完成
    "game_state": "Running"
  }
}
```

### 3. 弟子管理

#### 获取所有弟子
```
GET /api/game/{game_id}/disciples

Response:
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "云飞扬",
      "disciple_type": "Inner",
      "cultivation": {
        "level": "Foundation",
        "sub_level": "中期",  // 初期 | 中期 | 大圆满
        "progress": 65,
        "cultivation_path": {
          "total_tasks": 5,
          "completed_tasks": [1, 2, 5],
          "remaining_tasks": 2
        }
      },
      "age": 85,
      "lifespan": 300,
      "dao_heart": 75,
      "energy": 80,  // 0-100
      "constitution": 65,  // 0-100
      "talents": [
        { "talent_type": "Fire", "level": 7 },
        { "talent_type": "Sword", "level": 8 }
      ],
      "heritage": {
        "name": "xxx的传承",
        "tribulation_bonus": 0.1
      },
      "dao_companion": {
        "name": "仙子名",
        "cultivation_level": "GoldenCore"
      },
      "children_count": 2,
      "current_task_info": {
        "task_id": 1,
        "task_name": "讨伐噬魂虎",
        "progress": 2,
        "duration": 3
      }
    }
  ]
}
```

#### 获取单个弟子详情
```
GET /api/game/{game_id}/disciples/{disciple_id}

Response:
{
  "success": true,
  "data": {
    "id": 1,
    "name": "云飞扬",
    "disciple_type": "Inner",
    "cultivation": {
      "level": "Foundation",
      "sub_level": "中期",
      "progress": 65,
      "cultivation_path": {
        "total_tasks": 5,
        "completed_tasks": [1, 2, 5],
        "remaining_tasks": 2
      }
    },
    "age": 85,
    "lifespan": 300,
    "dao_heart": 75,
    "energy": 80,
    "constitution": 65,
    "talents": [
      { "talent_type": "Fire", "level": 7 },
      { "talent_type": "Sword", "level": 8 }
    ],
    "heritage": {
      "name": "xxx的传承",
      "tribulation_bonus": 0.1
    },
    "dao_companion": {
      "name": "仙子名",
      "cultivation_level": "GoldenCore"
    },
    "children_count": 2,
    "current_task_info": {
      "task_id": 1,
      "task_name": "讨伐噬魂虎",
      "progress": 2,
      "duration": 3
    }
  }
}
```

### 4. 任务管理

#### 获取当前可用任务
```
GET /api/game/{game_id}/tasks

Response:
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "讨伐噬魂虎",
      "task_type": {
        "type": "Combat",
        "data": {
          "enemy_name": "噬魂虎",
          "enemy_level": 2
        }
      },
      "rewards": {
        "progress": 15,
        "resources": 40,
        "reputation": 25
      },
      "dao_heart_impact": 3,
      "assigned_to": 1,
      "duration": 3,  // 任务需要的回合数
      "progress": 2,  // 已完成的回合数
      "energy_cost": 10,  // 每回合消耗的能量
      "constitution_cost": 5,  // 每回合消耗的体质
      "expiry_turns": 10,  // 任务到期回合数
      "created_turn": 5,  // 任务创建回合
      "remaining_turns": 5  // 剩余可用回合数
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

Response (成功):
{
  "success": true,
  "data": {
    "task_id": 1,
    "disciple_id": 5,
    "message": "已将任务分配给云飞扬"
  }
}

Response (弟子不存在 - 404):
{
  "success": false,
  "data": null,
  "error": {
    "code": "DISCIPLE_NOT_FOUND",
    "message": "弟子不存在"
  }
}

Response (弟子忙碌 - 409 Conflict):
{
  "success": false,
  "data": null,
  "error": {
    "code": "DISCIPLE_BUSY",
    "message": "该弟子已被分配到其他任务"
  }
}

注：
- 分配前会检查弟子是否存在
- 分配前会检查弟子是否已被分配到其他任务
- 同一弟子不能同时执行多个任务
```

#### 取消任务分配
```
DELETE /api/game/{game_id}/tasks/{task_id}/assign

Response:
{
  "success": true,
  "data": "已取消任务分配"
}
```

#### 自动分配所有任务
```
POST /api/game/{game_id}/tasks/auto-assign

Response:
{
  "success": true,
  "data": "自动分配完成"
}

注：实际分配详情在游戏内部处理
```

### 5. 渡劫管理

#### 获取可渡劫弟子
```
GET /api/game/{game_id}/tribulation/candidates

Response:
{
  "success": true,
  "data": {
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
}
```

#### 执行渡劫
```
POST /api/game/{game_id}/tribulation

Request:
{
  "disciple_id": 3
}

Response (成功):
{
  "success": true,
  "data": {
    "success": true,
    "disciple_id": 3,
    "name": "云飞扬",
    "old_level": "Foundation",
    "new_level": "GoldenCore",
    "message": "云飞扬渡劫成功，晋升至结丹期！"
  }
}

Response (失败):
{
  "success": true,
  "data": {
    "success": false,
    "disciple_id": 3,
    "name": "云飞扬",
    "old_level": "Foundation",
    "message": "云飞扬渡劫失败，陨落了..."
  }
}
```

### 6. 统计信息

#### 获取宗门统计
```
GET /api/game/{game_id}/statistics

Response:
{
  "success": true,
  "data": {
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
}
```

### 7. 地图系统

#### 获取地图数据
```
GET /api/game/{game_id}/map

Response:
{
  "success": true,
  "data": {
    "elements": [
      {
        "element_type": "Village",
        "name": "青木村",
        "position": { "x": 10, "y": 20 },
        "details": {
          "population": 500,
          "prosperity": 65
        }
      },
      {
        "element_type": "Faction",
        "name": "天剑派",
        "position": { "x": 50, "y": 30 },
        "details": {
          "power_level": 8,
          "relation": "Neutral"  // Friendly | Neutral | Hostile
        }
      },
      {
        "element_type": "DangerousLocation",
        "name": "幽冥森林",
        "position": { "x": 80, "y": 40 },
        "details": {
          "danger_level": 7,
          "description": "阴气森森，妖兽横行"
        }
      },
      {
        "element_type": "SecretRealm",
        "name": "玄天秘境",
        "position": { "x": 120, "y": 60 },
        "details": {
          "entry_requirement": "Foundation",
          "treasure_level": "High",
          "is_open": true
        }
      },
      {
        "element_type": "Monster",
        "name": "噬魂虎",
        "position": { "x": 90, "y": 50 },
        "details": {
          "level": 5,
          "threat": "Medium",
          "reward_tier": 2
        }
      }
    ]
  }
}
```

### 8. 丹药系统

#### 获取丹药库存
```
GET /api/game/{game_id}/pills

Response:
{
  "success": true,
  "data": {
    "pills": [
      {
        "pill_type": "QiRecovery",
        "name": "回气丹",
        "count": 5,
        "effects": {
          "energy_restore": 30,
          "constitution_restore": 0
        }
      },
      {
        "pill_type": "BodyStrength",
        "name": "强体丹",
        "count": 3,
        "effects": {
          "energy_restore": 0,
          "constitution_restore": 25
        }
      },
      {
        "pill_type": "VitalityElixir",
        "name": "生机灵液",
        "count": 2,
        "effects": {
          "energy_restore": 50,
          "constitution_restore": 50
        }
      },
      {
        "pill_type": "CultivationBoost",
        "name": "悟道丹",
        "count": 1,
        "effects": {
          "energy_restore": 20,
          "constitution_restore": 10
        }
      }
    ]
  }
}
```

#### 使用丹药
```
POST /api/game/{game_id}/pills/use

Request:
{
  "disciple_id": 5,
  "pill_type": "QiRecovery"
}

Response:
{
  "success": true,
  "data": {
    "disciple_name": "云飞扬",
    "pill_type": "QiRecovery",
    "pill_name": "回气丹",
    "before": {
      "energy": 40,
      "constitution": 60
    },
    "after": {
      "energy": 70,
      "constitution": 60
    },
    "message": "云飞扬服用回气丹，恢复了 30 点能量"
  }
}
```

## 错误响应

所有错误响应格式：
```json
{
  "success": false,
  "data": null,
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
- `DISCIPLE_BUSY` - 弟子已被分配到其他任务
- `TASK_NOT_FOUND` - 任务不存在
- `ASSIGNMENT_NOT_FOUND` - 任务分配不存在
- `INVALID_PILL_TYPE` - 无效的丹药类型
- `NO_PILLS` - 丹药数量不足
- `INTERNAL_ERROR` - 服务器内部错误

### HTTP 状态码
- `200 OK` - 请求成功（即使业务逻辑失败，如渡劫失败）
- `400 Bad Request` - 请求参数错误
- `404 Not Found` - 资源不存在
- `409 Conflict` - 资源冲突（如弟子已被分配）
- `500 Internal Server Error` - 服务器内部错误

## 数据模型

### DiscipleType
```typescript
type DiscipleType = "Outer" | "Inner" | "Personal";
```

### CultivationLevel
```typescript
type CultivationLevel =
  | "QiRefining"      // 炼气期
  | "Foundation"      // 筑基期
  | "GoldenCore"      // 结丹期
  | "NascentSoul"     // 元婴期
  | "SpiritSevering"  // 化神期
  | "VoidRefinement"  // 炼虚期
  | "Ascension";      // 飞升期
```

### CultivationSubLevel
```typescript
type CultivationSubLevel = "初期" | "中期" | "大圆满";
```

### TalentType
```typescript
type TalentType =
  // 五行灵根
  | "Fire" | "Water" | "Wood" | "Metal" | "Earth"
  // 特殊灵根
  | "Thunder" | "Ice" | "Wind"
  // 修行方向
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

### PillType
```typescript
type PillType =
  | "QiRecovery"        // 回气丹 - 恢复能量
  | "BodyStrength"      // 强体丹 - 恢复体质
  | "VitalityElixir"    // 生机灵液 - 同时恢复能量和体质
  | "CultivationBoost"; // 悟道丹 - 小幅恢复并提升修炼
```

### MapElementType
```typescript
type MapElementType =
  | "Village"            // 村庄
  | "Faction"            // 势力/门派
  | "DangerousLocation"  // 危险地点
  | "SecretRealm"        // 秘境
  | "Monster";           // 妖兽

type FactionRelation = "Friendly" | "Neutral" | "Hostile";
```

### CultivationPath
```typescript
interface CultivationPath {
  total_tasks: number;       // 突破所需完成的总任务数
  completed_tasks: number[]; // 已完成的任务ID列表
  remaining_tasks: number;   // 还需完成的任务数
}
```

### Heritage (传承)
```typescript
interface Heritage {
  name: string;              // 传承名称
  tribulation_bonus: number; // 渡劫成功率加成 (0.0 - 1.0)
}
```

### DaoCompanion (道侣)
```typescript
interface DaoCompanion {
  name: string;                    // 道侣姓名
  cultivation_level: CultivationLevel; // 修为境界
}
```

### MapPosition
```typescript
interface MapPosition {
  x: number;
  y: number;
}
```

## API 端点总结

共 17 个端点，分为 9 大类：

**系统** (1)
- GET `/api/version` - 获取 API 版本信息

**游戏管理** (2)
- POST `/api/game/new` - 创建新游戏
- GET `/api/game/:game_id` - 获取游戏状态

**回合管理** (2)
- POST `/api/game/:game_id/turn/start` - 开始新回合
- POST `/api/game/:game_id/turn/end` - 结束回合

**弟子管理** (2)
- GET `/api/game/:game_id/disciples` - 获取所有弟子
- GET `/api/game/:game_id/disciples/:disciple_id` - 获取单个弟子详情

**任务管理** (4)
- GET `/api/game/:game_id/tasks` - 获取可用任务
- POST `/api/game/:game_id/tasks/:task_id/assign` - 分配任务
- DELETE `/api/game/:game_id/tasks/:task_id/assign` - 取消任务分配
- POST `/api/game/:game_id/tasks/auto-assign` - 自动分配所有任务

**渡劫管理** (2)
- GET `/api/game/:game_id/tribulation/candidates` - 获取可渡劫弟子
- POST `/api/game/:game_id/tribulation` - 执行渡劫

**统计信息** (1)
- GET `/api/game/:game_id/statistics` - 获取宗门统计

**地图系统** (1)
- GET `/api/game/:game_id/map` - 获取地图数据

**丹药系统** (2)
- GET `/api/game/:game_id/pills` - 获取丹药库存
- POST `/api/game/:game_id/pills/use` - 使用丹药

## 技术架构

### 后端
- **语言**: Rust
- **Web 框架**: Axum 0.6
- **异步运行时**: Tokio
- **序列化**: Serde + Serde JSON
- **状态管理**: DashMap (线程安全的 HashMap)
- **游戏存储**: `Arc<Mutex<InteractiveGame>>`

### 服务器配置
- **地址**: `0.0.0.0:3000`
- **CORS**: 允许所有来源、方法、头部（开发模式）
- **启动方式**: 命令行参数 `--web` 或 `-w`
