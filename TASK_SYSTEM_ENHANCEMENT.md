# 任务系统增强 - 失效机制与执行进度

## 概述

为任务系统添加了完整的失效机制和执行进度追踪功能，使任务更加真实和具有时效性。

## 更新内容

### 后端更新

#### 1. Task 结构增强 (`src/task.rs:51-98`)

**新增字段：**

```rust
pub struct Task {
    // ... 原有字段
    pub duration: u32,          // 任务执行时间（回合数）
    pub expiry_turns: u32,      // 任务失效时间（回合数）
    pub created_turn: u32,      // 任务创建时的回合数
}
```

**任务执行时间默认值：**

| 任务类型 | 执行时间 |
|---------|---------|
| 采集任务 | 1回合 |
| 战斗任务 | 2回合 |
| 探索任务 | 3回合 |
| 辅助任务 | 1回合 |
| 投资任务 | 4回合 |

**失效检查方法：**

```rust
pub fn is_expired(&self, current_turn: u32) -> bool {
    current_turn >= self.created_turn + self.expiry_turns
}
```

#### 2. TaskAssignment 增强 (`src/interactive.rs:18-24`)

**新增字段追踪进度：**

```rust
pub struct TaskAssignment {
    pub task_id: usize,
    pub disciple_id: Option<usize>,
    pub started_turn: Option<u32>,  // 任务开始的回合数
    pub progress: u32,               // 已执行的回合数
}
```

#### 3. 任务进度系统 (`src/interactive.rs:451-506`)

**execute_turn 逻辑更新：**

```rust
pub fn execute_turn(&mut self) {
    // 更新任务进度并收集完成的任务
    let mut completed_tasks = Vec::new();

    for assignment in &mut self.task_assignments {
        if let Some(disciple_id) = assignment.disciple_id {
            // 如果任务刚开始，设置开始回合
            if assignment.started_turn.is_none() {
                assignment.started_turn = Some(self.sect.year);
            }

            // 增加进度
            assignment.progress += 1;

            // 检查任务是否完成
            if let Some(task) = self.current_tasks.iter().find(|t| t.id == assignment.task_id) {
                if assignment.progress >= task.duration {
                    completed_tasks.push((disciple_id, task.clone()));
                }
            }
        }
    }

    // 执行完成的任务并移除
    // ...
}
```

#### 4. 任务失效清理 (`src/interactive.rs:636-669`)

**remove_expired_tasks 方法：**

```rust
fn remove_expired_tasks(&mut self) {
    let current_turn = self.sect.year;
    let expired_task_ids: Vec<usize> = self
        .current_tasks
        .iter()
        .filter(|t| t.is_expired(current_turn))
        .map(|t| t.id)
        .collect();

    if !expired_task_ids.is_empty() {
        // 移除过期任务
        self.current_tasks.retain(|t| !expired_task_ids.contains(&t.id));
        self.task_assignments.retain(|a| !expired_task_ids.contains(&a.task_id));
        // 清除弟子的current_task
        // ...
    }
}
```

**调用时机：**
- 每个新回合开始时 (`start_turn()`)

#### 5. API类型更新 (`src/api_types.rs`)

**TaskDto 增强：**

```rust
pub struct TaskDto {
    // ... 原有字段
    pub duration: u32,           // 任务执行时间（回合数）
    pub progress: u32,            // 当前执行进度（回合数）
    pub expiry_turns: u32,        // 失效时间
    pub created_turn: u32,        // 创建回合
    pub remaining_turns: u32,     // 剩余回合数直到失效
}
```

**DiscipleDto 增强：**

```rust
pub struct DiscipleDto {
    // ... 原有字段
    pub current_task_info: Option<CurrentTaskInfo>,
}

pub struct CurrentTaskInfo {
    pub task_id: usize,
    pub task_name: String,
    pub duration: u32,
    pub progress: u32,
}
```

#### 6. Web API 更新 (`src/web_server.rs`)

**get_disciples 增强：**
- 填充 `current_task_info` 字段
- 包含任务名称、执行时间和当前进度

**get_tasks 增强：**
- 计算 `remaining_turns`（距离失效的剩余回合数）
- 返回任务进度信息

### 前端更新

#### 1. TypeScript 接口 (`frontend/src/api/gameApi.ts`)

**Disciple 接口更新：**

```typescript
export interface Disciple {
  // ... 原有字段
  current_task_info: {
    task_id: number;
    task_name: string;
    duration: number;
    progress: number;
  } | null;
}
```

**Task 接口更新：**

```typescript
export interface Task {
  // ... 原有字段
  duration: number;
  progress: number;
  expiry_turns: number;
  created_turn: number;
  remaining_turns: number;
}
```

#### 2. 弟子卡片进度显示 (`frontend/src/App.tsx:224-239`)

**当前任务进度条：**

```tsx
{d.current_task_info && (
  <div className="current-task">
    <div className="task-name">📋 {d.current_task_info.task_name}</div>
    <div className="task-progress-container">
      <div className="task-progress-bar">
        <div
          className="task-progress-fill"
          style={{width: `${(d.current_task_info.progress / d.current_task_info.duration) * 100}%`}}
        ></div>
      </div>
      <span className="task-progress-text">
        {d.current_task_info.progress}/{d.current_task_info.duration} 回合
      </span>
    </div>
  </div>
)}
```

#### 3. 任务卡片增强 (`frontend/src/App.tsx:250-284`)

**任务头部：**
- 任务名称
- 失效倒计时（剩余2回合或更少时变红色并闪烁）

**任务详情：**
- 执行时间显示
- 已分配任务显示进度条

```tsx
<div className="task-header">
  <h3>{t.name}</h3>
  <span className={`task-expiry ${t.remaining_turns <= 2 ? 'urgent' : ''}`}>
    ⏰ {t.remaining_turns}回合后失效
  </span>
</div>
<div className="task-duration">
  ⏱️ 需要执行 {t.duration} 回合
</div>
{t.assigned_to && t.progress > 0 && (
  <div className="task-progress-container">
    <div className="task-progress-bar">
      <div
        className="task-progress-fill"
        style={{width: `${(t.progress / t.duration) * 100}%`}}
      ></div>
    </div>
    <span className="task-progress-text">
      进度: {t.progress}/{t.duration}
    </span>
  </div>
)}
```

#### 4. CSS样式 (`frontend/src/App.css`)

**进度条样式：**

```css
.task-progress-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.25rem;
}

.task-progress-bar {
  flex: 1;
  height: 12px;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 6px;
  overflow: hidden;
}

.task-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #4299e1 0%, #3182ce 100%);
  transition: width 0.3s ease;
  border-radius: 6px;
}
```

**失效时间样式：**

```css
.task-expiry {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  background: #e2e8f0;
  color: #4a5568;
  font-weight: 600;
  white-space: nowrap;
  margin-left: 0.5rem;
}

.task-expiry.urgent {
  background: #fed7d7;
  color: #c53030;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}
```

## 功能特点

### 1. 任务失效机制

- ✅ 任务默认5回合后失效
- ✅ 每回合开始时自动清理过期任务
- ✅ 过期任务从任务列表中移除
- ✅ 执行过期任务的弟子自动解除任务
- ✅ 前端显示剩余回合数倒计时
- ✅ 即将失效（≤2回合）时红色闪烁提醒

### 2. 任务执行进度

- ✅ 不同类型任务有不同的执行时间
  - 采集任务：1回合
  - 战斗任务：2回合
  - 探索任务：3回合
  - 投资任务：4回合
- ✅ 任务分配后开始计时
- ✅ 每回合自动增加进度
- ✅ 进度达到duration时任务完成
- ✅ 完成的任务自动移除并发放奖励

### 3. 可视化进度

- ✅ 弟子卡片显示当前任务和进度条
- ✅ 任务卡片显示分配任务的执行进度
- ✅ 进度条使用渐变色
- ✅ 进度数值显示（如 2/3 回合）
- ✅ 流畅的动画过渡效果

### 4. 用户体验优化

- ✅ 即将失效的任务醒目提醒
- ✅ 清晰的视觉反馈
- ✅ 进度一目了然
- ✅ 紧迫感增强游戏策略性

## 游戏机制说明

### 任务生命周期

```
1. 创建任务（记录created_turn）
   ↓
2. 任务可被分配（显示剩余失效时间）
   ↓
3. 弟子接受任务（started_turn设置，progress=0）
   ↓
4. 每回合progress+1
   ↓
5a. progress >= duration → 任务完成 → 发放奖励 → 移除
5b. current_turn >= created_turn + expiry_turns → 任务过期 → 移除
```

### 策略要素

1. **时间压力**：玩家需要在任务失效前分配和完成
2. **资源分配**：长时间任务需要更合理的弟子调度
3. **优先级判断**：即将失效的高价值任务优先处理
4. **进度追踪**：实时了解任务执行状态

## 示例场景

### 场景1：采集任务

```
第10年：创建"在清风镇采集灵药"任务（duration=1, expiry_turns=5）
第10年：分配给弟子张三（progress=0/1）
第11年：执行回合，progress=1/1 → 任务完成 ✅
```

### 场景2：探索任务

```
第20年：创建"游历迷雾森林"任务（duration=3, expiry_turns=5）
第21年：分配给弟子李四（progress=0/3）
第22年：执行回合，progress=1/3
第23年：执行回合，progress=2/3
第24年：执行回合，progress=3/3 → 任务完成 ✅
```

### 场景3：任务过期

```
第30年：创建"讨伐噬魂虎"任务（duration=2, expiry_turns=5）
第31-34年：未分配
第35年：current_turn(35) >= created_turn(30) + expiry_turns(5)
       → 任务过期 ❌ 自动移除
```

## UI展示

### 弟子卡片中的任务进度

```
┌─────────────────────────────┐
│ 云飞扬         [内门弟子]   │
├─────────────────────────────┤
│ 修为: 筑基期 (65%)           │
│ ...                          │
│ 📋 讨伐噬魂虎                │
│ ████████░░░░░░░ 2/3 回合    │
└─────────────────────────────┘
```

### 任务列表中的信息

```
┌─────────────────────────────┐
│ 讨伐噬魂虎     ⏰ 3回合后失效│
├─────────────────────────────┤
│ 战斗任务                     │
│ ⏱️ 需要执行 2 回合           │
│ 修为+15 资源+40 声望+25      │
│ ✓ 已分配给 云飞扬            │
│ ████████░░░░░░░ 进度: 2/2   │
└─────────────────────────────┘
```

### 即将失效的任务（闪烁红色）

```
┌─────────────────────────────┐
│ 守卫清风镇     ⏰ 1回合后失效│ ← 红色闪烁
├─────────────────────────────┤
│ 辅助任务                     │
│ ...                          │
└─────────────────────────────┘
```

## 技术实现细节

### 后端任务进度计算

```rust
// 在execute_turn中
for assignment in &mut self.task_assignments {
    if let Some(disciple_id) = assignment.disciple_id {
        assignment.progress += 1;  // 每回合+1

        if let Some(task) = self.current_tasks.iter().find(...) {
            if assignment.progress >= task.duration {
                // 任务完成
                completed_tasks.push((disciple_id, task.clone()));
            }
        }
    }
}
```

### 前端进度条计算

```typescript
const progressPercentage = (progress / duration) * 100;

<div style={{width: `${progressPercentage}%`}} />
```

### 失效时间计算

```rust
let remaining_turns = if task.created_turn + task.expiry_turns > current_turn {
    task.created_turn + task.expiry_turns - current_turn
} else {
    0
};
```

## 文件更新列表

### 后端

- `src/task.rs` - Task结构，失效检查方法
- `src/interactive.rs` - TaskAssignment，进度追踪，失效清理
- `src/api_types.rs` - TaskDto, CurrentTaskInfo, DiscipleDto更新
- `src/web_server.rs` - API端点更新，填充进度信息

### 前端

- `frontend/src/api/gameApi.ts` - TypeScript接口更新
- `frontend/src/App.tsx` - UI组件更新，进度条显示
- `frontend/src/App.css` - 进度条和失效提醒样式

## 版本信息

- **版本：** v1.3.0
- **更新日期：** 2025-01-20
- **更新类型：** 任务系统增强

## 测试检查清单

- [x] Task添加duration, expiry_turns, created_turn字段
- [x] TaskAssignment添加started_turn, progress字段
- [x] 任务失效检查方法is_expired()
- [x] remove_expired_tasks()自动清理
- [x] execute_turn()进度追踪和完成检测
- [x] API返回progress, remaining_turns等信息
- [x] 前端显示弟子任务进度条
- [x] 前端显示任务失效倒计时
- [x] 即将失效任务红色闪烁提醒
- [x] 后端编译成功
- [x] 前端编译成功

---

**任务系统增强完成！** ⏰

现在任务具有真实的时效性和执行进度，增加了游戏的策略深度！
