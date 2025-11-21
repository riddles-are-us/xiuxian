# Modifier系统使用指南

## 概述

Modifier系统是一个强大的数值修饰系统，用于从native（原始）数值计算effective（有效）数值。所有事件判定都应该先通过modifier从native数值算出effective数值，然后再进行判定。

## 核心概念

### 1. ModifierTarget - 修饰目标

定义modifier作用于哪个数值：

```rust
pub enum ModifierTarget {
    // 基础属性
    DaoHeart,              // 道心
    Energy,                // 精力
    Constitution,          // 体魄

    // 资质相关
    TalentBonus(String),   // 资质加成（如 "Sword", "Wood"等）

    // 判定相关
    TribulationSuccessRate,  // 渡劫成功率
    TaskReward,              // 任务奖励
    TaskSuitability,         // 任务适配性
    TaskDifficulty,          // 任务难度

    // 收入相关
    Income,                  // 收入

    // 消耗相关
    EnergyConsumption,       // 精力消耗
    ConstitutionConsumption, // 体魄消耗

    // 修炼相关
    CultivationSpeed,        // 修炼速度
}
```

### 2. ModifierApplication - 应用方式

定义modifier如何修改数值：

```rust
pub enum ModifierApplication {
    Additive(f32),        // 固定值加成: effective = native + value
    Multiplicative(f32),  // 百分比加成: effective = native * (1.0 + value)
    Override(f32),        // 覆盖原值: effective = value
}
```

### 3. ModifierSource - 来源

定义modifier的来源：

```rust
pub enum ModifierSource {
    Talent,      // 天赋
    Equipment,   // 装备
    Buff,        // 增益状态
    Debuff,      // 减益状态
    Pill,        // 丹药效果
    Heritage,    // 传承
    Environment, // 环境影响
    System,      // 系统效果
}
```

## 计算流程

```
Native Value
  → Apply Additive Modifiers (加法)
  → Apply Multiplicative Modifiers (乘法)
  → Effective Value

特殊情况：
  → Override Modifier会直接覆盖，忽略其他所有modifier
```

## 使用示例

### 示例1: 增加渡劫成功率

```rust
use crate::modifier::{Modifier, ModifierTarget, ModifierApplication, ModifierSource};

// 创建一个+10%渡劫成功率的buff
let tribulation_buff = Modifier::new(
    "仙丹加持",
    ModifierTarget::TribulationSuccessRate,
    ModifierApplication::Additive(0.1),  // +10%
    ModifierSource::Pill,
);

// 添加到弟子
disciple.modifiers.add_modifier(tribulation_buff);

// 计算渡劫成功率时会自动应用modifier
let success_rate = disciple.tribulation_success_rate();
// 如果native成功率是40%，应用后变成50%
```

### 示例2: 减少精力消耗

```rust
// 创建一个-20%精力消耗的装备效果
let energy_saving_equipment = Modifier::new(
    "节能法宝",
    ModifierTarget::EnergyConsumption,
    ModifierApplication::Multiplicative(-0.2),  // -20%消耗
    ModifierSource::Equipment,
);

disciple.modifiers.add_modifier(energy_saving_equipment);

// 当消耗精力时会自动应用modifier
disciple.consume_energy(10);
// 实际只消耗 10 * (1.0 - 0.2) = 8点精力
```

### 示例3: 提升任务奖励

```rust
// 创建一个+50%任务奖励的临时buff（持续3回合）
let reward_buff = Modifier::new_temporary(
    "顿悟状态",
    ModifierTarget::TaskReward,
    ModifierApplication::Multiplicative(0.5),  // +50%奖励
    ModifierSource::Buff,
    3,  // 持续3回合
);

disciple.modifiers.add_modifier(reward_buff);

// 完成任务时会自动应用modifier
let reward = disciple.complete_task(&task);
// 如果native奖励是10点修为，应用后获得15点
```

### 示例4: 提升任务适配性

```rust
// 创建一个+2等级的任务适配modifier
// 让弟子可以接受更高难度的任务
let suitability_boost = Modifier::new(
    "越阶挑战",
    ModifierTarget::TaskSuitability,
    ModifierApplication::Additive(2.0),  // +2个修为等级
    ModifierSource::Buff,
);

disciple.modifiers.add_modifier(suitability_boost);

// 检查任务适配性时会自动应用
let suitable = task.is_suitable_for_disciple(&disciple);
// 练气期(0)弟子可以接受筑基期(2)的任务
```

### 示例5: 增强特定资质

```rust
// 创建一个+0.3的剑道天赋加成
let sword_talent_boost = Modifier::new(
    "剑意领悟",
    ModifierTarget::TalentBonus("Sword".to_string()),
    ModifierApplication::Additive(0.3),
    ModifierSource::Heritage,
);

disciple.modifiers.add_modifier(sword_talent_boost);

// 获取资质加成时会自动应用
let sword_bonus = disciple.get_talent_bonus(&TalentType::Sword);
// 如果原本剑道天赋等级是5（加成0.5），应用后变成0.8
```

## 管理Modifier

### 添加Modifier

```rust
// 添加永久modifier
disciple.modifiers.add_modifier(modifier);

// 添加临时modifier（会自动计时）
let temp_modifier = Modifier::new_temporary(
    "临时加成",
    target,
    application,
    source,
    duration_turns,
);
disciple.modifiers.add_modifier(temp_modifier);
```

### 移除Modifier

```rust
// 按ID移除
disciple.modifiers.remove_modifier("modifier_id");

// 按来源移除所有modifier
disciple.modifiers.remove_modifiers_by_source(&ModifierSource::Pill);

// 清除所有modifier
disciple.modifiers.clear();
```

### 更新Modifier（每回合）

```rust
// 每回合需要调用tick()来更新持续时间
let expired_count = disciple.modifiers.tick();
// 返回过期移除的modifier数量
```

## 已应用Modifier的判定点

所有以下判定已经自动应用modifier系统：

1. **资质加成** (`disciple.rs:212`) - `get_talent_bonus()`
   - 使用 `ModifierTarget::TalentBonus(talent_type)`

2. **渡劫成功率** (`disciple.rs:227`) - `tribulation_success_rate()`
   - 使用 `ModifierTarget::DaoHeart` 和 `ModifierTarget::TribulationSuccessRate`

3. **任务奖励** (`disciple.rs:290`) - `complete_task()`
   - 使用 `ModifierTarget::TaskReward`

4. **精力消耗** (`disciple.rs:350`) - `consume_energy()`
   - 使用 `ModifierTarget::EnergyConsumption`

5. **体魄消耗** (`disciple.rs:372`) - `consume_constitution()`
   - 使用 `ModifierTarget::ConstitutionConsumption`

6. **任务适配性** (`task.rs:145`) - `is_suitable_for_disciple()`
   - 使用 `ModifierTarget::TaskSuitability`

## 设计原则

1. **Native → Effective**: 所有判定都应该先计算native值，然后应用modifier得到effective值
2. **透明应用**: modifier的应用应该对调用者透明，在判定方法内部自动处理
3. **分层应用**: Additive → Multiplicative → Override的顺序
4. **优先级**: 可以通过`with_priority()`设置优先级，高优先级先应用

## 扩展建议

未来可以添加的modifier类型：

- 修炼速度加成
- 收入加成
- 寿命延长
- 突破成功率
- 特定任务类型的加成
- 战斗力加成
- 灵根亲和度

## 注意事项

1. 确保在每回合结束时调用 `tick()` 来更新临时modifier
2. Override类型的modifier会忽略所有其他modifier
3. modifier是叠加的，多个相同类型的modifier会按顺序全部应用
4. 百分比modifier的值是相对于1.0的，例如0.2表示+20%，-0.3表示-30%
