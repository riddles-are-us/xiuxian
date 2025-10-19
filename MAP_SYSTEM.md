# 地图系统 - 2D坐标与可视化

## 概述

为游戏添加了完整的2D地图系统，包括后端坐标支持和前端tile地图可视化。

## 更新内容

### 后端更新

#### 1. 地图坐标系统 (`src/map.rs`)

**新增结构：**

```rust
/// 地图坐标
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// 带坐标的地图元素
#[derive(Debug, Clone)]
pub struct PositionedElement {
    pub element: MapElement,
    pub position: Position,
}
```

**GameMap 增强：**

```rust
pub struct GameMap {
    pub elements: Vec<PositionedElement>,  // 改为存储带坐标的元素
    pub width: i32,   // 地图宽度（20格）
    pub height: i32,  // 地图高度（20格）
}
```

**初始元素位置分布：**

- 清风镇: (5, 5)
- 灵泉村: (15, 8)
- 青云派: (10, 10)
- 迷雾森林: (3, 15)
- 火焰洞窟: (17, 3)
- 噬魂虎: (8, 12)

#### 2. API类型定义 (`src/api_types.rs`)

**地图元素DTO：**

```rust
#[derive(Debug, Serialize, Clone)]
pub struct MapElementDto {
    pub element_type: String,
    pub name: String,
    pub position: PositionDto,
    pub details: MapElementDetails,
}

#[derive(Debug, Serialize, Clone)]
pub struct PositionDto {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum MapElementDetails {
    Village { population: u32, prosperity: u32 },
    Faction { power_level: u32, relationship: i32 },
    DangerousLocation { danger_level: u32 },
    SecretRealm { realm_type: String, difficulty: u32 },
    Monster { level: u32, is_demon: bool },
}

#[derive(Debug, Serialize)]
pub struct MapDataResponse {
    pub width: i32,
    pub height: i32,
    pub elements: Vec<MapElementDto>,
}
```

#### 3. Web API端点 (`src/web_server.rs`)

**新增端点：**

```
GET /api/game/:game_id/map
```

**响应示例：**

```json
{
  "success": true,
  "data": {
    "width": 20,
    "height": 20,
    "elements": [
      {
        "element_type": "Village",
        "name": "清风镇",
        "position": { "x": 5, "y": 5 },
        "details": {
          "type": "Village",
          "population": 1000,
          "prosperity": 50
        }
      }
    ]
  }
}
```

### 前端更新

#### 1. TypeScript接口 (`frontend/src/api/gameApi.ts`)

```typescript
export interface MapElement {
  element_type: string;
  name: string;
  position: {
    x: number;
    y: number;
  };
  details: {
    type: string;
    population?: number;
    prosperity?: number;
    power_level?: number;
    relationship?: number;
    danger_level?: number;
    realm_type?: string;
    difficulty?: number;
    level?: number;
    is_demon?: boolean;
  };
}

export interface MapData {
  width: number;
  height: number;
  elements: MapElement[];
}
```

**新增API方法：**

```typescript
getMap: async (gameId: string): Promise<MapData>
```

#### 2. MapView组件 (`frontend/src/MapView.tsx`)

**核心功能：**

- 20x20 tile网格地图
- 交互式元素显示
- 点击元素显示详细信息
- 悬停高亮效果
- 不同元素类型的图标和颜色

**元素图标映射：**

| 元素类型 | 图标 | 颜色 |
|---------|------|------|
| Village | 🏘️ | 绿色渐变 |
| Faction | ⚔️ | 橙色渐变 |
| DangerousLocation | ⚠️ | 红色渐变 |
| SecretRealm | 🌀 | 紫色渐变 |
| Monster | 👹 | 粉色渐变 |

**组件特性：**

```tsx
<MapView mapData={mapData} />
```

- 自动生成20x20网格
- 每个tile显示坐标
- 点击tile显示详情面板
- 详情面板显示位置和属性信息
- 响应式设计

#### 3. CSS样式 (`frontend/src/MapView.css`)

**主要样式类：**

- `.map-grid` - Grid布局，动态列数
- `.map-tile` - 50x50px tile，悬停放大效果
- `.tile-village` / `.tile-faction` 等 - 元素类型专属渐变色
- `.element-details-panel` - 侧边详情面板
- `.tile-coords` - 坐标标签

**视觉效果：**

- 渐变背景色区分元素类型
- 悬停时tile放大110%并显示阴影
- 详情面板渐变色头部
- 响应式布局适配移动端

#### 4. App集成 (`frontend/src/App.tsx`)

**状态管理：**

```typescript
const [mapData, setMapData] = useState<MapData | null>(null);
const [showMap, setShowMap] = useState(false);
```

**数据加载：**

```typescript
const [info, disciplesList, tasksList, map] = await Promise.all([
  gameApi.getGame(id),
  gameApi.getDisciples(id),
  gameApi.getTasks(id),
  gameApi.getMap(id)  // 新增
]);
```

**UI控制：**

```tsx
<button onClick={() => setShowMap(!showMap)}>
  {showMap ? '隐藏地图' : '显示地图'}
</button>

{showMap && mapData && <MapView mapData={mapData} />}
```

## 功能特点

### 1. 完整的坐标系统

- ✅ 所有地图元素都有明确的(x, y)坐标
- ✅ 新生成的怪物自动随机分配位置
- ✅ 20x20的地图网格（可扩展）

### 2. 可视化tile地图

- ✅ 直观的网格布局
- ✅ 图标化的元素显示
- ✅ 颜色编码的元素分类
- ✅ 坐标标签辅助定位

### 3. 交互式信息展示

- ✅ 悬停预览
- ✅ 点击查看详情
- ✅ 侧边详情面板
- ✅ 关闭按钮控制

### 4. 响应式设计

- ✅ 桌面端：地图+侧边详情
- ✅ 移动端：垂直布局
- ✅ 自适应tile大小
- ✅ 可滚动区域

## 使用说明

### 启动应用

**后端：**

```bash
cargo run --release -- --web
```

**前端：**

```bash
cd frontend
npm start
```

### 查看地图

1. 打开浏览器访问 http://localhost:3001
2. 创建或加载游戏
3. 点击"显示地图"按钮
4. 在tile地图上点击任意元素查看详情

### API使用示例

```bash
# 获取地图数据
curl http://localhost:3000/api/game/{game_id}/map
```

## 技术实现细节

### 后端坐标系统

1. **Position结构**：简单的(x, y)坐标
2. **PositionedElement**：包装元素+位置
3. **地图更新**：新元素自动获得随机坐标

### 前端渲染策略

1. **Grid布局**：CSS Grid动态生成tile网格
2. **元素查找**：O(n)遍历查找坐标处的元素
3. **条件渲染**：根据元素类型显示不同内容

### 性能优化

- 使用CSS transitions实现流畅动画
- 悬停状态通过独立state管理
- 详情面板仅在选中时渲染

## 地图元素详情展示

### Village（村庄）

- 人口数量
- 繁荣度

### Faction（势力）

- 实力等级
- 关系值（正值绿色，负值红色）

### DangerousLocation（险地）

- 危险等级

### SecretRealm（秘境）

- 秘境类型（Fire/Water等）
- 难度等级

### Monster（怪物）

- 等级
- 状态（正常/成魔，成魔显示为红色）

## 示例截图说明

地图视图包含：

```
┌─────────────────────────────────────┐
│  [20x20 Tile Grid]    [Details]    │
│  🏘️ 🏘️ ⚔️             ┌──────────┐ │
│  ⚠️ 🌀 👹             │ 清风镇    │ │
│  ...                  │ 类型:村庄 │ │
│                       │ 位置:(5,5)│ │
│                       │ 人口:1000 │ │
│                       └──────────┘ │
└─────────────────────────────────────┘
```

## 扩展建议

### 短期改进

1. **地图缩放**：添加zoom功能
2. **元素过滤**：按类型筛选显示
3. **路径显示**：显示任务路线

### 中期改进

1. **迷你地图**：小地图导航
2. **区域系统**：划分地图区域
3. **动态事件**：地图上显示事件标记

### 长期改进

1. **可编辑地图**：手动放置元素
2. **地形系统**：添加地形类型
3. **势力范围**：显示势力控制区域

## 版本信息

- **版本：** v1.2.0
- **更新日期：** 2025-01-20
- **更新类型：** 地图系统实现

## 测试检查清单

- [x] 后端Position和PositionedElement结构
- [x] 地图元素包含坐标
- [x] GET /map API端点
- [x] 前端MapData接口
- [x] MapView组件渲染
- [x] Tile grid显示正确
- [x] 元素图标和颜色
- [x] 点击显示详情
- [x] 悬停效果
- [x] 响应式布局
- [x] 前后端编译成功

---

**地图系统实现完成！** 🗺️

现在可以在tile地图上查看所有地图元素的位置和信息了！
