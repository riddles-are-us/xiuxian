use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 导入必要的类型用于条件判定
use crate::cultivation::CultivationLevel;
use crate::disciple::{DiscipleType, TalentType};

/// Modifier系统 - 用于从native数值计算effective数值
///
/// 所有事件判定都应该先通过modifier从native数值算出effective数值，然后再进行判定

/// Modifier作用的目标类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModifierTarget {
    // 基础属性
    DaoHeart,
    Energy,
    Constitution,

    // 资质相关
    TalentBonus(String), // 资质类型的加成

    // 判定相关
    TribulationSuccessRate,  // 渡劫成功率
    TaskReward,              // 任务奖励
    TaskSuitability,         // 任务适配性（影响修为判定）
    TaskDifficulty,          // 任务难度

    // 收入相关
    Income,                  // 收入

    // 消耗相关
    EnergyConsumption,       // 精力消耗
    ConstitutionConsumption, // 体魄消耗

    // 修炼相关
    CultivationSpeed,        // 修炼速度
}

/// Modifier应用方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifierApplication {
    /// 固定值加成：effective = native + value
    Additive(f32),

    /// 百分比加成：effective = native * (1.0 + value)
    /// 例如：value = 0.2 表示 +20%
    Multiplicative(f32),

    /// 覆盖原值：effective = value
    Override(f32),
}

/// Modifier来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModifierSource {
    Talent,       // 天赋
    Equipment,    // 装备
    Buff,         // 增益状态
    Debuff,       // 减益状态
    Pill,         // 丹药效果
    Heritage,     // 传承
    Environment,  // 环境影响
    System,       // 系统效果
    Relationship, // 关系加成
}

/// Modifier条件 - 用于判断modifier是否对某个弟子生效
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // 关系相关
    HasDaoCompanion,                    // 有道侣
    HasMaster,                          // 有师父
    HasDisciples,                       // 有徒弟
    RelationLevelGreaterOrEqual(usize, crate::relationship::RelationDimension, crate::relationship::RelationLevel),

    // 组合条件
    And(Vec<ModifierCondition>),
    Or(Vec<ModifierCondition>),
    Not(Box<ModifierCondition>),

    // 总是生效
    Always,
}

impl ModifierCondition {
    /// 判断条件是否对指定弟子生效
    pub fn check(&self, disciple: &crate::disciple::Disciple) -> bool {
        match self {
            // 修为相关
            ModifierCondition::CultivationLevelEquals(level) => {
                disciple.cultivation.current_level == *level
            }
            ModifierCondition::CultivationLevelGreaterThan(level) => {
                disciple.cultivation.current_level > *level
            }
            ModifierCondition::CultivationLevelLessThan(level) => {
                disciple.cultivation.current_level < *level
            }
            ModifierCondition::CultivationLevelGreaterOrEqual(level) => {
                disciple.cultivation.current_level >= *level
            }
            ModifierCondition::CultivationLevelLessOrEqual(level) => {
                disciple.cultivation.current_level <= *level
            }

            // 弟子类型
            ModifierCondition::DiscipleTypeEquals(dtype) => {
                disciple.disciple_type == *dtype
            }

            // 属性相关
            ModifierCondition::DaoHeartGreaterThan(value) => {
                disciple.dao_heart > *value
            }
            ModifierCondition::DaoHeartLessThan(value) => {
                disciple.dao_heart < *value
            }
            ModifierCondition::EnergyGreaterThan(value) => {
                disciple.energy > *value
            }
            ModifierCondition::EnergyLessThan(value) => {
                disciple.energy < *value
            }
            ModifierCondition::ConstitutionGreaterThan(value) => {
                disciple.constitution > *value
            }
            ModifierCondition::ConstitutionLessThan(value) => {
                disciple.constitution < *value
            }

            // 年龄相关
            ModifierCondition::AgeGreaterThan(age) => {
                disciple.age > *age
            }
            ModifierCondition::AgeLessThan(age) => {
                disciple.age < *age
            }
            ModifierCondition::AgeEquals(age) => {
                disciple.age == *age
            }

            // 资质相关
            ModifierCondition::HasTalent(talent_type) => {
                disciple.talents.iter().any(|t| &t.talent_type == talent_type)
            }
            ModifierCondition::TalentLevelGreaterThan(talent_type, level) => {
                disciple.talents.iter()
                    .find(|t| &t.talent_type == talent_type)
                    .map(|t| t.level > *level)
                    .unwrap_or(false)
            }
            ModifierCondition::TalentLevelEquals(talent_type, level) => {
                disciple.talents.iter()
                    .find(|t| &t.talent_type == talent_type)
                    .map(|t| t.level == *level)
                    .unwrap_or(false)
            }

            // 关系相关
            ModifierCondition::HasDaoCompanion => {
                disciple.has_dao_companion()
            }
            ModifierCondition::HasMaster => {
                disciple.get_master_id().is_some()
            }
            ModifierCondition::HasDisciples => {
                !disciple.get_disciple_ids().is_empty()
            }
            ModifierCondition::RelationLevelGreaterOrEqual(target_id, dimension, level) => {
                disciple.get_relationship(*target_id)
                    .map(|rel| rel.scores.get_level(*dimension) >= *level)
                    .unwrap_or(false)
            }

            // 组合条件
            ModifierCondition::And(conditions) => {
                conditions.iter().all(|c| c.check(disciple))
            }
            ModifierCondition::Or(conditions) => {
                conditions.iter().any(|c| c.check(disciple))
            }
            ModifierCondition::Not(condition) => {
                !condition.check(disciple)
            }

            // 总是生效
            ModifierCondition::Always => true,
        }
    }
}

/// 条件Modifier - 包含判定条件和modifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalModifier {
    pub condition: ModifierCondition,
    pub modifier: Modifier,
}

impl ConditionalModifier {
    /// 创建一个新的条件modifier
    pub fn new(condition: ModifierCondition, modifier: Modifier) -> Self {
        Self { condition, modifier }
    }

    /// 检查是否对指定弟子生效
    pub fn applies_to(&self, disciple: &crate::disciple::Disciple) -> bool {
        self.condition.check(disciple)
    }

    /// 如果条件满足，返回modifier的引用
    pub fn get_modifier_if_applies(&self, disciple: &crate::disciple::Disciple) -> Option<&Modifier> {
        if self.applies_to(disciple) {
            Some(&self.modifier)
        } else {
            None
        }
    }
}

/// 单个Modifier定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub id: String,
    pub name: String,
    pub target: ModifierTarget,
    pub application: ModifierApplication,
    pub source: ModifierSource,

    /// 持续时间（回合数），None表示永久
    pub duration: Option<u32>,

    /// 优先级，用于排序（高优先级先应用）
    pub priority: i32,
}

impl Modifier {
    /// 创建一个新的modifier
    pub fn new(
        name: impl Into<String>,
        target: ModifierTarget,
        application: ModifierApplication,
        source: ModifierSource,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            target,
            application,
            source,
            duration: None,
            priority: 0,
        }
    }

    /// 创建一个临时modifier（有持续时间）
    pub fn new_temporary(
        name: impl Into<String>,
        target: ModifierTarget,
        application: ModifierApplication,
        source: ModifierSource,
        duration: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            target,
            application,
            source,
            duration: Some(duration),
            priority: 0,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 应用modifier到数值
    pub fn apply(&self, value: f32) -> f32 {
        match &self.application {
            ModifierApplication::Additive(add) => value + add,
            ModifierApplication::Multiplicative(mult) => value * (1.0 + mult),
            ModifierApplication::Override(new_val) => *new_val,
        }
    }
}

/// ModifierStack - 管理一个实体上的所有modifier
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModifierStack {
    modifiers: Vec<Modifier>,
}

impl ModifierStack {
    /// 创建一个空的modifier stack
    pub fn new() -> Self {
        Self {
            modifiers: Vec::new(),
        }
    }

    /// 添加一个modifier
    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
        // 按优先级排序（高优先级在前）
        self.modifiers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 移除指定ID的modifier
    pub fn remove_modifier(&mut self, id: &str) -> bool {
        let len_before = self.modifiers.len();
        self.modifiers.retain(|m| m.id != id);
        self.modifiers.len() < len_before
    }

    /// 移除指定来源的所有modifier
    pub fn remove_modifiers_by_source(&mut self, source: &ModifierSource) {
        self.modifiers.retain(|m| &m.source != source);
    }

    /// 获取所有指定目标的modifier
    pub fn get_modifiers_for_target(&self, target: &ModifierTarget) -> Vec<&Modifier> {
        self.modifiers.iter().filter(|m| &m.target == target).collect()
    }

    /// 计算effective值
    ///
    /// 应用顺序：
    /// 1. 先应用所有Additive modifier
    /// 2. 再应用所有Multiplicative modifier
    /// 3. 最后应用Override modifier（如果有，会覆盖前面的结果）
    pub fn calculate_effective(&self, target: &ModifierTarget, native: f32) -> f32 {
        self.calculate_effective_with_extras(target, native, &[])
    }

    /// 计算effective值（包含额外的modifiers，如宗门modifiers）
    ///
    /// 应用顺序：
    /// 1. 先应用所有Additive modifier（个人 + 额外）
    /// 2. 再应用所有Multiplicative modifier（个人 + 额外）
    /// 3. 最后应用Override modifier（如果有，会覆盖前面的结果）
    pub fn calculate_effective_with_extras(
        &self,
        target: &ModifierTarget,
        native: f32,
        extra_modifiers: &[&Modifier],
    ) -> f32 {
        // 合并个人modifiers和额外modifiers
        let mut all_modifiers: Vec<&Modifier> = self.get_modifiers_for_target(target);
        all_modifiers.extend(
            extra_modifiers
                .iter()
                .filter(|m| &m.target == target)
                .copied()
        );

        // 检查是否有Override modifier
        if let Some(override_mod) = all_modifiers.iter().find(|m| {
            matches!(m.application, ModifierApplication::Override(_))
        }) {
            return override_mod.apply(native);
        }

        let mut value = native;

        // 应用Additive modifiers
        for modifier in all_modifiers.iter() {
            if matches!(modifier.application, ModifierApplication::Additive(_)) {
                value = modifier.apply(value);
            }
        }

        // 应用Multiplicative modifiers
        for modifier in all_modifiers.iter() {
            if matches!(modifier.application, ModifierApplication::Multiplicative(_)) {
                value = modifier.apply(value);
            }
        }

        value
    }

    /// 更新所有临时modifier的持续时间
    /// 返回过期的modifier数量
    pub fn tick(&mut self) -> usize {
        let before_len = self.modifiers.len();

        // 减少持续时间
        for modifier in &mut self.modifiers {
            if let Some(duration) = modifier.duration.as_mut() {
                *duration = duration.saturating_sub(1);
            }
        }

        // 移除持续时间为0的modifier
        self.modifiers.retain(|m| {
            m.duration.map(|d| d > 0).unwrap_or(true)
        });

        before_len - self.modifiers.len()
    }

    /// 获取所有modifier的引用
    pub fn get_all_modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    /// 清除所有modifier
    pub fn clear(&mut self) {
        self.modifiers.clear();
    }

    /// 获取modifier数量
    pub fn len(&self) -> usize {
        self.modifiers.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_additive_modifier() {
        let modifier = Modifier::new(
            "Test Buff",
            ModifierTarget::DaoHeart,
            ModifierApplication::Additive(10.0),
            ModifierSource::Buff,
        );

        assert_eq!(modifier.apply(50.0), 60.0);
    }

    #[test]
    fn test_multiplicative_modifier() {
        let modifier = Modifier::new(
            "Test Buff",
            ModifierTarget::DaoHeart,
            ModifierApplication::Multiplicative(0.2), // +20%
            ModifierSource::Buff,
        );

        let result = modifier.apply(50.0);
        assert!((result - 60.0).abs() < 0.001, "Expected ~60.0, got {}", result);
    }

    #[test]
    fn test_override_modifier() {
        let modifier = Modifier::new(
            "Test Override",
            ModifierTarget::DaoHeart,
            ModifierApplication::Override(100.0),
            ModifierSource::System,
        );

        assert_eq!(modifier.apply(50.0), 100.0);
    }

    #[test]
    fn test_modifier_stack_calculate() {
        let mut stack = ModifierStack::new();

        // 添加 +10 固定值
        stack.add_modifier(Modifier::new(
            "Buff 1",
            ModifierTarget::DaoHeart,
            ModifierApplication::Additive(10.0),
            ModifierSource::Buff,
        ));

        // 添加 +20% 百分比
        stack.add_modifier(Modifier::new(
            "Buff 2",
            ModifierTarget::DaoHeart,
            ModifierApplication::Multiplicative(0.2),
            ModifierSource::Buff,
        ));

        // Native: 50
        // After Additive: 50 + 10 = 60
        // After Multiplicative: 60 * 1.2 = 72
        let effective = stack.calculate_effective(&ModifierTarget::DaoHeart, 50.0);
        assert_eq!(effective, 72.0);
    }

    #[test]
    fn test_modifier_stack_override() {
        let mut stack = ModifierStack::new();

        stack.add_modifier(Modifier::new(
            "Buff 1",
            ModifierTarget::DaoHeart,
            ModifierApplication::Additive(10.0),
            ModifierSource::Buff,
        ));

        stack.add_modifier(Modifier::new(
            "Override",
            ModifierTarget::DaoHeart,
            ModifierApplication::Override(100.0),
            ModifierSource::System,
        ));

        // Override应该直接返回100，忽略其他modifier
        let effective = stack.calculate_effective(&ModifierTarget::DaoHeart, 50.0);
        assert_eq!(effective, 100.0);
    }

    #[test]
    fn test_modifier_duration() {
        let mut stack = ModifierStack::new();

        stack.add_modifier(Modifier::new_temporary(
            "Temp Buff",
            ModifierTarget::DaoHeart,
            ModifierApplication::Additive(10.0),
            ModifierSource::Buff,
            2, // 持续2回合
        ));

        assert_eq!(stack.len(), 1);

        // 第一回合
        stack.tick();
        assert_eq!(stack.len(), 1);

        // 第二回合
        stack.tick();
        assert_eq!(stack.len(), 0); // 应该被移除
    }

    #[test]
    fn test_remove_by_source() {
        let mut stack = ModifierStack::new();

        stack.add_modifier(Modifier::new(
            "Buff",
            ModifierTarget::DaoHeart,
            ModifierApplication::Additive(10.0),
            ModifierSource::Buff,
        ));

        stack.add_modifier(Modifier::new(
            "Equipment",
            ModifierTarget::DaoHeart,
            ModifierApplication::Additive(5.0),
            ModifierSource::Equipment,
        ));

        assert_eq!(stack.len(), 2);

        stack.remove_modifiers_by_source(&ModifierSource::Buff);
        assert_eq!(stack.len(), 1);
    }
}
