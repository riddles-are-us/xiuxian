# 大型建筑系统文档

## 概述

大型建筑是占据多个格子的地图元素。弟子站在大型建筑占据的任意一个格子上，都可以接取该建筑的任务。

## 支持的尺寸

目前支持任意尺寸的建筑，常用配置：
- **1x1**: 默认尺寸，占据单个格子
- **2x2**: 大型建筑，占据4个格子

## 配置方式

在 `config/map_elements.json` 中为建筑添加 `size` 字段：

```json
{
  "name": "青云派",
  "power_level": 3,
  "relationship": 20,
  "position": { "x": 10, "y": 10 },
  "size": { "width": 2, "height": 2 },
  "friendly_task_templates": [...]
}
```

### 配置字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `size` | 对象（可选） | 建筑尺寸，不填则默认 1x1 |
| `size.width` | 数字 | 宽度（格子数） |
| `size.height` | 数字 | 高度（格子数） |

### 支持大型尺寸的建筑类型

- 村庄 (`villages`)
- 势力 (`factions`)
- 险地 (`dangerous_locations`)
- 秘境 (`secret_realms`)

## 位置计算

`position` 字段表示建筑的**左上角**位置。

例如，配置 `position: {x: 10, y: 10}, size: {width: 2, height: 2}` 的建筑会占据：
- (10, 10) - 左上角（主位置）
- (11, 10) - 右上
- (10, 11) - 左下
- (11, 11) - 右下

## 任务接取规则

弟子在大型建筑的**任意一个格子**上都可以接取该建筑的任务：

```
任务有效位置 = 建筑占据的所有格子
弟子位置 ∈ 任务有效位置 → 可以接取任务
```

### API 响应

任务的 API 响应中包含：
- `position`: 主位置（用于显示）
- `valid_positions`: 所有有效位置数组（用于位置检查）

```json
{
  "id": 1,
  "name": "与青云派交流",
  "position": { "x": 10, "y": 10 },
  "valid_positions": [
    { "x": 10, "y": 10 },
    { "x": 11, "y": 10 },
    { "x": 10, "y": 11 },
    { "x": 11, "y": 11 }
  ]
}
```

## 前端渲染

大型建筑在地图上显示为一个合并的大菱形：

- 2x2 建筑显示为 160x80 像素的菱形（普通格子的 2 倍）
- 只显示一个图标（放大版本）
- 建筑名称显示在底部

### 等轴测坐标偏移

为了正确显示大型建筑，需要计算等轴测坐标偏移：

```typescript
// 大型建筑的左偏移量补偿
const largeOffsetX = (width - 1) * (TILE_WIDTH / 2);
```

## 代码位置

| 功能 | 文件 | 结构/函数 |
|------|------|----------|
| 尺寸配置结构 | `src/config.rs` | `struct SizeConfig` |
| 位置元素 | `src/map.rs` | `struct PositionedElement` |
| 位置检查方法 | `src/map.rs` | `fn contains_position()` |
| 获取所有位置 | `src/map.rs` | `fn get_all_positions()` |
| 任务有效位置 | `src/task.rs` | `Task.valid_positions` |
| API 类型 | `src/api_types.rs` | `SizeDto`, `MapElementDto` |
| 前端渲染 | `frontend/src/MapView.tsx` | 大型建筑渲染逻辑 |
| 前端样式 | `frontend/src/MapView.css` | `.iso-large-2x2` |

## 当前大型建筑

默认配置中的大型建筑：

| 名称 | 类型 | 位置 | 尺寸 |
|------|------|------|------|
| 青云派 | 势力 | (10, 10) | 2x2 |
| 火焰洞窟 | 秘境 | (17, 3) | 2x2 |
