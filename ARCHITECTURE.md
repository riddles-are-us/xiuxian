# 修仙模拟器架构设计

## 整体架构

本项目采用模块化设计，将游戏系统分为以下核心模块：

```
┌─────────────────────────────────────────┐
│           Game (游戏主循环)              │
│  - 年度循环                              │
│  - 状态管理                              │
│  - 胜负判定                              │
└──────────┬──────────────────────────────┘
           │
    ┌──────┴──────┬──────────┬──────────┐
    │             │          │          │
┌───▼───┐   ┌────▼────┐ ┌──▼────┐ ┌───▼────┐
│ Sect  │   │  Map    │ │ Event │ │ Recruit│
│(宗门) │   │ (地图)  │ │(事件) │ │ (招募) │
└───┬───┘   └────┬────┘ └───────┘ └────────┘
    │            │
┌───▼─────┐  ┌──▼──────┐
│Disciple │  │ Element │
│ (弟子)  │  │(地图元素)│
└───┬─────┘  └────┬────┘
    │             │
┌───▼─────┐  ┌───▼────┐
│Cultivate│  │  Task  │
│ (修为)  │  │ (任务) │
└─────────┘  └────────┘
```

## 核心模块说明

### 1. cultivation.rs - 修为系统

**职责**：定义修仙等级和相关规则

**核心类型**：
- `CultivationLevel`: 修为等级枚举
  - 练气、筑基、结丹、凝婴、化神、练虚、飞升

**核心方法**：
- `base_lifespan()`: 返回该等级的基础寿元
- `requires_tribulation()`: 是否需要渡劫
- `next()`: 返回下一个等级

**设计要点**：
- 使用枚举保证类型安全
- 实现 PartialOrd 支持等级比较
- 不可变数据，纯函数式设计

### 2. disciple.rs - 弟子系统

**职责**：弟子的所有属性和行为

**核心类型**：
```rust
Disciple {
    id: usize,
    name: String,
    disciple_type: DiscipleType,  // 外门/内门/亲传
    cultivation: CultivationProgress,
    talents: Vec<Talent>,
    age: u32,
    lifespan: u32,
    dao_heart: u32,
    heritage: Option<Heritage>,
    dao_companion: Option<DaoCompanion>,
    children: Vec<usize>,
}
```

**核心方法**：
- `is_alive()`: 检查是否存活
- `is_immortal()`: 是否已飞升
- `attempt_tribulation()`: 尝试渡劫
- `breakthrough()`: 尝试突破
- `complete_task()`: 完成任务
- `generate_heritage()`: 生成传承

**设计要点**：
- 所有状态封装在结构体内
- 通过方法保证状态一致性
- 资质系统影响任务效率
- 道心影响渡劫成功率

### 3. task.rs - 任务系统

**职责**：定义各类任务和奖励

**核心类型**：
```rust
TaskType {
    Gathering,    // 采集
    Combat,       // 战斗
    Exploration,  // 探索
    Auxiliary,    // 辅助
    Investment,   // 投资
}

Task {
    id: usize,
    name: String,
    task_type: TaskType,
    progress_reward: u32,     // 修为进度奖励
    resource_reward: u32,     // 资源奖励
    reputation_reward: i32,   // 声望奖励
    dao_heart_impact: i32,    // 道心影响
}
```

**核心方法**：
- `is_suitable_for_disciple()`: 检查弟子是否适合

**设计要点**：
- 使用枚举+结构体组合表示不同任务类型
- 每种任务类型有特定的属性
- 奖励多样化（修为、资源、声望、道心）

### 4. map.rs - 地图系统

**职责**：管理地图元素和任务生成

**核心类型**：
```rust
MapElement {
    Village,            // 村庄
    Faction,            // 势力
    DangerousLocation,  // 险要
    SecretRealm,        // 秘境
    Monster,            // 怪物
}
```

**核心方法**：
- `generate_tasks()`: 生成对应任务
- `get_resource_income()`: 获取资源收入
- `update()`: 地图更新（怪物成长、新怪物出现）
- `has_demon()`: 检查是否有怪物成魔

**设计要点**：
- 每种元素有独特的任务生成逻辑
- 声望影响资源产出
- 怪物会动态成长
- 随机生成新元素

### 5. sect.rs - 宗门系统

**职责**：管理宗门整体状态

**核心类型**：
```rust
Sect {
    name: String,
    disciples: Vec<Disciple>,
    resources: u32,
    reputation: i32,
    is_immortal_sect: bool,
    heritages: Vec<Heritage>,
    year: u32,
}
```

**核心方法**：
- `recruit_disciple()`: 招募弟子
- `alive_disciples()`: 获取存活弟子
- `check_immortal_sect()`: 检查是否成为仙门
- `is_destroyed()`: 检查是否灭门
- `yearly_update()`: 年度更新
- `get_statistics()`: 获取统计信息

**设计要点**：
- 集中管理所有弟子
- 资源和声望全局管理
- 提供统计和查询接口
- 处理弟子死亡和传承

### 6. event.rs - 事件系统

**职责**：事件生成、处理和弟子招募

**核心类型**：
```rust
GameEvent {
    TaskAvailable,
    TaskCompleted,
    DiscipleRecruited,
    DiscipleBreakthrough,
    DiscipleTribulation,
    DiscipleDeath,
    YearlyIncome,
    MapUpdate,
}
```

**核心系统**：
- `EventSystem`: 事件队列和处理
- `RecruitmentSystem`: 弟子生成和招募

**设计要点**：
- 事件驱动架构
- 解耦游戏逻辑
- 统一的事件处理流程
- 随机招募机制

### 7. game.rs - 游戏主循环

**职责**：游戏流程控制和状态管理

**核心类型**：
```rust
GameState {
    Running,  // 运行中
    Victory,  // 胜利
    Defeat,   // 失败
}

Game {
    sect: Sect,
    map: GameMap,
    event_system: EventSystem,
    recruitment_system: RecruitmentSystem,
    state: GameState,
}
```

**核心方法**：
- `yearly_cycle()`: 年度循环
- `auto_assign_tasks()`: 自动分配任务
- `execute_task()`: 执行任务
- `check_breakthroughs()`: 检查突破
- `check_game_state()`: 检查游戏状态
- `run()`: 运行游戏主循环

**年度循环流程**：
```
1. 计算年度收入
2. 尝试招募弟子
3. 生成并分配任务
4. 弟子年龄增长
5. 检查突破和渡劫
6. 地图更新
7. 处理事件
8. 检查游戏状态
9. 显示统计
```

**设计要点**：
- 单一职责：只负责流程控制
- 状态机模式管理游戏状态
- 清晰的游戏循环
- 自动化任务分配

## 数据流

### 1. 任务执行流程
```
地图元素
  → 生成任务
  → 分配给弟子
  → 弟子完成任务
  → 获得奖励（修为、资源、声望）
  → 更新宗门状态
```

### 2. 弟子成长流程
```
完成任务
  → 累积修为进度
  → 达到100%
  → 检查是否需要渡劫
    → 是：计算渡劫成功率 → 成功/失败
    → 否：直接突破
  → 更新修为等级
  → 重置进度
```

### 3. 事件处理流程
```
游戏循环
  → 生成事件
  → 加入事件队列
  → 统一处理事件
  → 更新游戏状态
  → 触发新事件
```

## 设计模式应用

### 1. 策略模式
- `TaskType`: 不同任务类型有不同的执行策略
- `MapElement`: 不同地图元素有不同的任务生成策略

### 2. 状态模式
- `GameState`: 游戏的不同状态
- `CultivationLevel`: 修为的不同阶段

### 3. 观察者模式
- `EventSystem`: 事件发布-订阅机制

### 4. 工厂模式
- `RecruitmentSystem`: 弟子生成工厂

## Rust 特性应用

### 1. 所有权系统
- 避免数据竞争
- 明确的资源管理
- 零成本抽象

### 2. 类型系统
- 枚举表示有限状态
- Option 处理可选值
- Result 处理错误（未来扩展）

### 3. 模式匹配
- 枚举处理
- Option/Result 处理
- 类型安全的分支

### 4. 迭代器
- 函数式编程风格
- 零成本抽象
- 链式调用

## 扩展性设计

### 1. 新增修为等级
- 在 `CultivationLevel` 枚举中添加
- 更新相关的 match 分支

### 2. 新增任务类型
- 在 `TaskType` 枚举中添加
- 实现对应的任务结构体
- 在地图元素中生成

### 3. 新增地图元素
- 在 `MapElement` 枚举中添加
- 实现任务生成逻辑
- 在地图初始化中添加

### 4. 新增事件类型
- 在 `GameEvent` 枚举中添加
- 在事件处理中添加处理逻辑

## 性能考虑

### 1. 内存使用
- 使用 Vec 存储动态数据
- 避免不必要的克隆
- 使用引用传递大对象

### 2. 计算优化
- 懒惰求值（Iterator）
- 缓存计算结果
- 避免重复计算

### 3. 随机性
- 使用 `rand` crate
- 单次创建 RNG
- 避免频繁初始化

## 未来优化方向

1. **并发处理**：使用 Rayon 并行处理任务
2. **序列化**：使用 Serde 保存/加载游戏
3. **配置系统**：使用 TOML/JSON 配置游戏参数
4. **日志系统**：使用 log/env_logger 记录游戏事件
5. **测试**：添加单元测试和集成测试
6. **GUI**：使用 egui 或其他 GUI 框架
7. **性能分析**：使用 criterion 进行基准测试
