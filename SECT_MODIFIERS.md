# 宗门条件Modifier系统使用指南

## 概述

宗门条件Modifier系统允许你为宗门定义全局的modifier，这些modifier只对满足特定条件的弟子生效。这是一个强大的系统，可以用来实现：

- 内门弟子专属加成
- 高修为弟子的额外奖励
- 特定资质弟子的加成
- 年龄相关的buff/debuff
- 复杂的组合条件

## 核心概念

### 1. ModifierCondition - 判定条件

定义一个弟子是否满足特定条件：

```rust
pub enum ModifierCondition {
    // 修为相关
    CultivationLevelEquals(CultivationLevel),
    CultivationLevelGreaterThan(CultivationLevel),
    CultivationLevelLessThan(CultivationLevel),
    CultivationLevelGreaterOrEqual(CultivationLevel),
    CultivationLevelLessOrEqual(CultivationLevel),

    // 弟子类型
    DiscipleTypeEquals(DiscipleType),

    // 属性相关
    DaoHeartGreaterThan(u32),
    DaoHeartLessThan(u32),
    EnergyGreaterThan(u32),
    EnergyLessThan(u32),
    ConstitutionGreaterThan(u32),
    ConstitutionLessThan(u32),

    // 年龄相关
    AgeGreaterThan(u32),
    AgeLessThan(u32),
    AgeEquals(u32),

    // 资质相关
    HasTalent(TalentType),
    TalentLevelGreaterThan(TalentType, u32),
    TalentLevelEquals(TalentType, u32),

    // 组合条件
    And(Vec<ModifierCondition>),  // 所有条件都满足
    Or(Vec<ModifierCondition>),   // 任一条件满足
    Not(Box<ModifierCondition>),  // 条件不满足

    // 总是生效
    Always,
}
```

### 2. ConditionalModifier - 条件modifier

将判定条件和modifier组合在一起：

```rust
pub struct ConditionalModifier {
    pub condition: ModifierCondition,
    pub modifier: Modifier,
}
```

### 3. Sect - 宗门管理

宗门结构包含一个条件modifier列表：

```rust
pub struct Sect {
    // ... 其他字段
    pub sect_modifiers: Vec<ConditionalModifier>,
}
```

## 使用方法

### 基本用法

```rust
use crate::modifier::{Modifier, ModifierTarget, ModifierApplication, ModifierSource, ModifierCondition, ConditionalModifier};
use crate::cultivation::CultivationLevel;
use crate::disciple::DiscipleType;

// 1. 创建一个modifier
let modifier = Modifier::new(
    "内门弟子修炼加成",
    ModifierTarget::TaskReward,
    ModifierApplication::Multiplicative(0.2),  // +20%任务奖励
    ModifierSource::System,
);

// 2. 创建条件（只对内门弟子生效）
let condition = ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner);

// 3. 创建条件modifier
let conditional_modifier = ConditionalModifier::new(condition, modifier);

// 4. 添加到宗门
sect.add_sect_modifier(conditional_modifier);

// 5. 在判定时应用
let sect_mods = sect.get_applicable_modifiers(&disciple);
let success_rate = disciple.tribulation_success_rate_with_sect_modifiers(&sect_mods);
```

## 使用示例

### 示例1: 内门弟子加成

```rust
// 内门弟子获得+20%任务奖励
let inner_disciple_bonus = ConditionalModifier::new(
    ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
    Modifier::new(
        "内门弟子修炼加成",
        ModifierTarget::TaskReward,
        ModifierApplication::Multiplicative(0.2),
        ModifierSource::System,
    )
);

sect.add_sect_modifier(inner_disciple_bonus);
```

### 示例2: 高修为弟子渡劫加成

```rust
// 筑基及以上弟子渡劫成功率+10%
let high_level_tribulation_bonus = ConditionalModifier::new(
    ModifierCondition::CultivationLevelGreaterOrEqual(CultivationLevel::Foundation),
    Modifier::new(
        "高阶修士渡劫加成",
        ModifierTarget::TribulationSuccessRate,
        ModifierApplication::Additive(0.1),
        ModifierSource::System,
    )
);

sect.add_sect_modifier(high_level_tribulation_bonus);
```

### 示例3: 剑道天赋弟子加成

```rust
// 拥有剑道天赋的弟子战斗力提升
let sword_talent_boost = ConditionalModifier::new(
    ModifierCondition::HasTalent(TalentType::Sword),
    Modifier::new(
        "剑道宗门传承",
        ModifierTarget::TalentBonus("Sword".to_string()),
        ModifierApplication::Additive(0.3),
        ModifierSource::Heritage,
    )
);

sect.add_sect_modifier(sword_talent_boost);
```

### 示例4: 年轻弟子修炼速度加成

```rust
// 年龄小于30岁的弟子获得+30%任务奖励（年轻潜力大）
let youth_bonus = ConditionalModifier::new(
    ModifierCondition::AgeLessThan(30),
    Modifier::new(
        "青年潜力",
        ModifierTarget::TaskReward,
        ModifierApplication::Multiplicative(0.3),
        ModifierSource::System,
    )
);

sect.add_sect_modifier(youth_bonus);
```

### 示例5: 组合条件 - 高道心内门弟子

```rust
// 道心>70且为内门弟子的人，渡劫成功率额外+15%
let elite_inner_disciple = ConditionalModifier::new(
    ModifierCondition::And(vec![
        ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
        ModifierCondition::DaoHeartGreaterThan(70),
    ]),
    Modifier::new(
        "精英内门弟子",
        ModifierTarget::TribulationSuccessRate,
        ModifierApplication::Additive(0.15),
        ModifierSource::System,
    )
);

sect.add_sect_modifier(elite_inner_disciple);
```

### 示例6: 复杂条件 - 天才弟子识别

```rust
// 满足以下任一条件的弟子获得"天才"buff：
// - 年龄<25且修为>=结丹
// - 拥有剑道天赋且等级>=7
// - 道心>90
let genius_condition = ModifierCondition::Or(vec![
    ModifierCondition::And(vec![
        ModifierCondition::AgeLessThan(25),
        ModifierCondition::CultivationLevelGreaterOrEqual(CultivationLevel::GoldenCore),
    ]),
    ModifierCondition::TalentLevelGreaterThan(TalentType::Sword, 6),
    ModifierCondition::DaoHeartGreaterThan(90),
]);

let genius_buff = ConditionalModifier::new(
    genius_condition,
    Modifier::new(
        "天才弟子",
        ModifierTarget::TaskReward,
        ModifierApplication::Multiplicative(0.5),  // +50%修炼速度
        ModifierSource::System,
    )
);

sect.add_sect_modifier(genius_buff);
```

### 示例7: 低能量惩罚

```rust
// 精力低于30的弟子，任务奖励-20%（疲劳状态）
let fatigue_penalty = ConditionalModifier::new(
    ModifierCondition::EnergyLessThan(30),
    Modifier::new(
        "疲劳状态",
        ModifierTarget::TaskReward,
        ModifierApplication::Multiplicative(-0.2),
        ModifierSource::Debuff,
    )
);

sect.add_sect_modifier(fatigue_penalty);
```

## 在判定中应用宗门Modifiers

所有支持宗门modifiers的方法都有一个`_with_sect_modifiers`版本：

```rust
// 1. 从宗门获取对该弟子生效的所有modifier
let sect_mods = sect.get_applicable_modifiers(&disciple);

// 2. 在判定时使用
let success_rate = disciple.tribulation_success_rate_with_sect_modifiers(&sect_mods);
let talent_bonus = disciple.get_talent_bonus_with_sect_modifiers(&TalentType::Sword, &sect_mods);
let suitable = task.is_suitable_for_disciple_with_sect_modifiers(&disciple, &sect_mods);
```

## 支持宗门Modifiers的判定方法

以下方法都有`_with_sect_modifiers`版本：

1. **Disciple方法**:
   - `get_effective_dao_heart_with_sect_modifiers()`
   - `get_effective_energy_with_sect_modifiers()`
   - `get_effective_constitution_with_sect_modifiers()`
   - `get_talent_bonus_with_sect_modifiers()`
   - `tribulation_success_rate_with_sect_modifiers()`

2. **Task方法**:
   - `is_suitable_for_disciple_with_sect_modifiers()`

## 管理宗门Modifiers

```rust
// 添加
sect.add_sect_modifier(conditional_modifier);

// 移除（按索引）
sect.remove_sect_modifier(index);

// 清除所有
sect.clear_sect_modifiers();

// 获取对指定弟子生效的modifiers
let applicable = sect.get_applicable_modifiers(&disciple);
```

## 条件判定逻辑

```rust
// 检查条件是否满足
if conditional_modifier.applies_to(&disciple) {
    // 该modifier对这个弟子生效
}

// 或者直接使用条件检查
if condition.check(&disciple) {
    // 条件满足
}
```

## 设计模式

### 模式1: 阶梯式加成

为不同修为等级设置递增的加成：

```rust
// 练气期：+10%
sect.add_sect_modifier(ConditionalModifier::new(
    ModifierCondition::CultivationLevelEquals(CultivationLevel::QiRefining),
    Modifier::new("练气加成", ModifierTarget::TaskReward,
        ModifierApplication::Multiplicative(0.1), ModifierSource::System)
));

// 筑基期：+20%
sect.add_sect_modifier(ConditionalModifier::new(
    ModifierCondition::CultivationLevelEquals(CultivationLevel::Foundation),
    Modifier::new("筑基加成", ModifierTarget::TaskReward,
        ModifierApplication::Multiplicative(0.2), ModifierSource::System)
));

// ... 以此类推
```

### 模式2: 角色专精

为不同角色类型设置专属加成：

```rust
// 剑修：战斗力+30%
// 丹修：炼丹天赋+50%
// 阵修：阵法天赋+50%
```

### 模式3: 动态平衡

根据弟子状态动态调整：

```rust
// 高道心：渡劫加成
// 低道心：修炼惩罚
// 高能量：效率提升
// 低能量：效率下降
```

## 注意事项

1. **条件检查性能**: 每次判定时都会检查所有宗门modifiers，如果条件很多可能影响性能
2. **条件叠加**: 多个条件modifier可以同时生效，它们会按照modifier系统的规则叠加
3. **优先级**: 宗门modifiers和个人modifiers是平等的，会一起参与计算
4. **序列化**: 所有类型都支持序列化，可以保存和加载

## 高级用法

### 临时启用/禁用宗门Modifier

```rust
// 保存当前的宗门modifiers
let saved_modifiers = sect.sect_modifiers.clone();

// 清除所有
sect.clear_sect_modifiers();

// 执行某些操作...

// 恢复
sect.sect_modifiers = saved_modifiers;
```

### 查询哪些弟子满足特定条件

```rust
let condition = ModifierCondition::CultivationLevelGreaterThan(CultivationLevel::Foundation);

let qualified_disciples: Vec<&Disciple> = sect.disciples
    .iter()
    .filter(|d| condition.check(d))
    .collect();

println!("有{}个弟子修为超过筑基期", qualified_disciples.len());
```

## 完整示例：建立内门制度

```rust
use crate::modifier::*;
use crate::cultivation::CultivationLevel;
use crate::disciple::DiscipleType;

// 建立内门制度
fn setup_inner_sect_system(sect: &mut Sect) {
    // 1. 内门弟子基础加成
    sect.add_sect_modifier(ConditionalModifier::new(
        ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
        Modifier::new(
            "内门弟子修炼加成",
            ModifierTarget::TaskReward,
            ModifierApplication::Multiplicative(0.2),
            ModifierSource::System,
        )
    ));

    // 2. 内门弟子精力消耗降低
    sect.add_sect_modifier(ConditionalModifier::new(
        ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
        Modifier::new(
            "内门弟子精力效率",
            ModifierTarget::EnergyConsumption,
            ModifierApplication::Multiplicative(-0.15),  // -15%消耗
            ModifierSource::System,
        )
    ));

    // 3. 亲传弟子超强加成
    sect.add_sect_modifier(ConditionalModifier::new(
        ModifierCondition::DiscipleTypeEquals(DiscipleType::Personal),
        Modifier::new(
            "亲传弟子",
            ModifierTarget::TaskReward,
            ModifierApplication::Multiplicative(0.5),  // +50%
            ModifierSource::System,
        )
    ));

    // 4. 亲传弟子渡劫加成
    sect.add_sect_modifier(ConditionalModifier::new(
        ModifierCondition::DiscipleTypeEquals(DiscipleType::Personal),
        Modifier::new(
            "掌门庇护",
            ModifierTarget::TribulationSuccessRate,
            ModifierApplication::Additive(0.15),  // +15%
            ModifierSource::System,
        )
    ));
}
```

这个系统为修仙模拟器提供了极大的灵活性，可以实现各种复杂的宗门制度和规则！
