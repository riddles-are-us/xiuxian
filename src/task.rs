use crate::disciple::TalentType;
use crate::modifier::ModifierTarget;
use crate::map::Position;

/// 任务资格检查结果
#[derive(Debug, Clone)]
pub struct TaskEligibility {
    pub eligible: bool,
    pub reason: Option<String>,
}

impl TaskEligibility {
    pub fn eligible() -> Self {
        Self {
            eligible: true,
            reason: None,
        }
    }

    pub fn ineligible(reason: &str) -> Self {
        Self {
            eligible: false,
            reason: Some(reason.to_string()),
        }
    }
}

/// 任务类型
#[derive(Debug, Clone)]
pub enum TaskType {
    Gathering(GatheringTask),     // 采集任务
    Combat(CombatTask),           // 战斗任务
    Exploration(ExplorationTask), // 探索任务
    Auxiliary(AuxiliaryTask),     // 辅助任务
    Investment(InvestmentTask),   // 投资任务
}

/// 采集任务
#[derive(Debug, Clone)]
pub struct GatheringTask {
    pub resource_type: String,
    pub difficulty: u32,
}

/// 战斗任务
#[derive(Debug, Clone)]
pub struct CombatTask {
    pub enemy_id: Option<usize>,  // 怪物唯一ID（None表示势力战斗，不需要移除）
    pub enemy_name: String,       // 怪物名称（用于显示）
    pub enemy_level: u32,
    pub difficulty: u32,
}

/// 探索任务
#[derive(Debug, Clone)]
pub struct ExplorationTask {
    pub location: String,
    pub danger_level: u32,
}

/// 辅助任务
#[derive(Debug, Clone)]
pub struct AuxiliaryTask {
    pub task_name: String,
    pub skill_required: Option<TalentType>,
}

/// 投资任务
#[derive(Debug, Clone)]
pub struct InvestmentTask {
    pub resource_cost: u32,
    pub description: String,
}

/// 任务
#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub task_type: TaskType,
    pub progress_reward: u32, // 完成后获得的修为进度
    pub resource_reward: u32, // 完成后获得的资源
    pub reputation_reward: i32, // 完成后获得的声望
    pub dao_heart_impact: i32,  // 对道心的影响
    pub duration: u32,          // 任务执行时间（回合数）
    pub expiry_turns: u32,      // 任务失效时间（回合数）
    pub created_turn: u32,      // 任务创建时的回合数
    pub energy_cost: u32,       // 精力消耗（每回合）
    pub constitution_cost: u32, // 体魄消耗（每回合）
    pub location_id: Option<String>, // 任务关联的地点ID（用于确保同一地点同一类型任务唯一性）
    pub position: Option<Position>,  // 任务位置（需要弟子到达才能执行）
    pub max_participants: u32,  // 最大参与人数（1=单人任务，>1=多人任务）
}

impl Task {
    pub fn new(
        id: usize,
        name: String,
        task_type: TaskType,
        progress_reward: u32,
        resource_reward: u32,
    ) -> Self {
        // 根据任务类型设置默认执行时间、消耗和最大参与人数
        let (duration, energy_cost, constitution_cost, max_participants) = match &task_type {
            TaskType::Gathering(_) => (1, 5, 2, 2),      // 采集任务：最多2人
            TaskType::Combat(_) => (2, 15, 10, 3),       // 战斗任务：最多3人
            TaskType::Exploration(_) => (3, 10, 5, 2),   // 探索任务：最多2人
            TaskType::Auxiliary(_) => (1, 5, 3, 1),      // 辅助任务：单人
            TaskType::Investment(_) => (4, 3, 1, 1),     // 投资任务：单人
        };

        Self {
            id,
            name,
            task_type,
            progress_reward,
            resource_reward,
            reputation_reward: 0,
            dao_heart_impact: 0,
            duration,
            expiry_turns: 5,  // 默认5回合后失效
            created_turn: 0,   // 将在生成时设置
            energy_cost,
            constitution_cost,
            location_id: None,  // 默认无地点关联
            position: None,     // 默认无位置要求
            max_participants,
        }
    }

    /// 创建带有所有参数的任务（包括创建回合）
    pub fn new_with_turn(
        id: usize,
        name: String,
        task_type: TaskType,
        progress_reward: u32,
        resource_reward: u32,
        reputation_reward: i32,
        dao_heart_impact: i32,
        created_turn: u32,
    ) -> Self {
        // 根据任务类型设置默认执行时间、消耗和最大参与人数
        let (duration, energy_cost, constitution_cost, max_participants) = match &task_type {
            TaskType::Gathering(_) => (1, 5, 2, 2),
            TaskType::Combat(_) => (2, 15, 10, 3),
            TaskType::Exploration(_) => (3, 10, 5, 2),
            TaskType::Auxiliary(_) => (1, 5, 3, 1),
            TaskType::Investment(_) => (4, 3, 1, 1),
        };

        Self {
            id,
            name,
            task_type,
            progress_reward,
            resource_reward,
            reputation_reward,
            dao_heart_impact,
            duration,
            expiry_turns: 20,  // 修炼路径任务有更长的失效时间
            created_turn,
            energy_cost,
            constitution_cost,
            location_id: None,  // 默认无地点关联
            position: None,     // 默认无位置要求
            max_participants,
        }
    }

    /// 检查任务是否已失效
    pub fn is_expired(&self, current_turn: u32) -> bool {
        current_turn >= self.created_turn + self.expiry_turns
    }

    /// 检查弟子是否适合此任务（应用modifier后的有效判定）
    pub fn is_suitable_for_disciple(&self, disciple: &crate::disciple::Disciple) -> bool {
        self.is_suitable_for_disciple_with_sect_modifiers(disciple, &[])
    }

    /// 检查弟子是否适合此任务（应用modifier后的有效判定，包含宗门modifiers）
    pub fn is_suitable_for_disciple_with_sect_modifiers(
        &self,
        disciple: &crate::disciple::Disciple,
        sect_modifiers: &[&crate::modifier::Modifier],
    ) -> bool {
        match &self.task_type {
            TaskType::Combat(_) => {
                // 战斗任务不再有等级限制，任何弟子都可以接受
                // 成功率由等级差距决定
                true
            }
            TaskType::Exploration(exploration) => {
                // 1. 获取native修为等级
                let native_level = disciple.cultivation.current_level as u32 as f32;

                // 2. 应用TaskSuitability modifier获取effective等级（包含宗门modifiers）
                let effective_level = disciple.modifiers.calculate_effective_with_extras(
                    &ModifierTarget::TaskSuitability,
                    native_level,
                    sect_modifiers
                ) as u32;

                // 3. 检查修为是否足够应对危险
                effective_level * 10 >= exploration.danger_level
            }
            TaskType::Auxiliary(auxiliary) => {
                // 检查是否有对应的资质（不受modifier影响）
                if let Some(ref skill) = auxiliary.skill_required {
                    disciple.talents.iter().any(|t| &t.talent_type == skill)
                } else {
                    true
                }
            }
            _ => true,
        }
    }

    /// 获取任务需要的技能
    pub fn get_skill_required(&self) -> Option<String> {
        match &self.task_type {
            TaskType::Auxiliary(auxiliary) => {
                auxiliary.skill_required.as_ref().map(|skill| format!("{:?}", skill))
            }
            _ => None,
        }
    }

    /// 检查弟子是否可以接受此任务，返回详细的结果和原因
    pub fn check_eligibility(
        &self,
        disciple: &crate::disciple::Disciple,
        sect_modifiers: &[&crate::modifier::Modifier],
        is_at_position: bool,
        is_busy: bool,
        is_already_assigned: bool,
        current_assigned_count: usize,
    ) -> TaskEligibility {
        // 1. 检查是否已分配
        if is_already_assigned {
            return TaskEligibility::ineligible("已接受此任务");
        }

        // 2. 检查任务人数是否已满
        if current_assigned_count >= self.max_participants as usize {
            return TaskEligibility::ineligible("任务人数已满");
        }

        // 3. 检查位置（如果任务有位置要求）
        if self.position.is_some() && !is_at_position {
            if let Some(pos) = &self.position {
                return TaskEligibility::ineligible(&format!(
                    "需要前往任务位置 ({}, {})",
                    pos.x, pos.y
                ));
            }
        }

        // 4. 检查弟子是否正在执行其他任务
        if is_busy {
            return TaskEligibility::ineligible("正在执行其他任务");
        }

        // 5. 检查精力
        if disciple.energy < self.energy_cost {
            return TaskEligibility::ineligible(&format!(
                "精力不足 (需要{}, 当前{})",
                self.energy_cost, disciple.energy
            ));
        }

        // 6. 检查体魄
        if disciple.constitution < self.constitution_cost {
            return TaskEligibility::ineligible(&format!(
                "体魄不足 (需要{}, 当前{})",
                self.constitution_cost, disciple.constitution
            ));
        }

        // 7. 检查任务类型特定条件
        match &self.task_type {
            TaskType::Combat(_) => {
                // 战斗任务不再有等级限制，成功率由等级差距决定
                // 在 check_eligibility 中允许接受任务
            }
            TaskType::Exploration(exploration) => {
                let native_level = disciple.cultivation.current_level as u32 as f32;
                let effective_level = disciple.modifiers.calculate_effective_with_extras(
                    &ModifierTarget::TaskSuitability,
                    native_level,
                    sect_modifiers
                ) as u32;

                if effective_level * 10 < exploration.danger_level {
                    let required_level = (exploration.danger_level + 9) / 10; // 向上取整
                    return TaskEligibility::ineligible(&format!(
                        "修为不足以应对危险 (危险等级{}, 需要约{}级, 当前{:?}={})",
                        exploration.danger_level,
                        required_level,
                        disciple.cultivation.current_level,
                        native_level as u32
                    ));
                }
            }
            TaskType::Auxiliary(auxiliary) => {
                if let Some(ref skill) = auxiliary.skill_required {
                    let has_skill = disciple.talents.iter().any(|t| &t.talent_type == skill);
                    if !has_skill {
                        return TaskEligibility::ineligible(&format!(
                            "需要技能: {:?}",
                            skill
                        ));
                    }
                }
            }
            _ => {}
        }

        TaskEligibility::eligible()
    }

    /// 获取任务类型的字符串表示（用于比较）
    pub fn get_task_type_str(&self) -> &'static str {
        match &self.task_type {
            TaskType::Gathering(_) => "Gathering",
            TaskType::Combat(_) => "Combat",
            TaskType::Exploration(_) => "Exploration",
            TaskType::Auxiliary(_) => "Auxiliary",
            TaskType::Investment(_) => "Investment",
        }
    }

    /// 计算弟子的有效战斗等级
    /// 等级 = 大境界 × 4 + 小境界 + 1
    /// 练气初期=1, 中期=2, 圆满=3
    /// 筑基初期=5, 中期=6, 圆满=7 (渡劫+2)
    /// 金丹初期=9, 中期=10, 圆满=11
    pub fn calculate_disciple_combat_level(disciple: &crate::disciple::Disciple) -> u32 {
        use crate::cultivation::SubLevel;

        let major_level = disciple.cultivation.current_level as u32;
        let sub_level = match disciple.cultivation.sub_level {
            SubLevel::Early => 0,
            SubLevel::Middle => 1,
            SubLevel::Perfect => 2,
        };

        // 每个大境界贡献4级（3个小境界 + 渡劫跳2级 - 1）
        major_level * 4 + sub_level + 1
    }

    /// 计算战斗任务的成功率
    /// 基于弟子等级和敌人等级的差距
    /// 返回 0.0 到 1.0 之间的概率
    pub fn calculate_combat_success_rate(&self, disciple: &crate::disciple::Disciple) -> f64 {
        match &self.task_type {
            TaskType::Combat(combat) => {
                let disciple_level = Self::calculate_disciple_combat_level(disciple);
                let enemy_level = combat.enemy_level;

                // 等级差 = 弟子等级 - 敌人等级
                // 正数表示弟子更强，负数表示敌人更强
                let level_diff = disciple_level as i32 - enemy_level as i32;

                // 基础成功率 70%
                // 每高一级 +10%，每低一级 -15%
                // 最低 5%，最高 95%
                let base_rate = 0.7;
                let rate = if level_diff >= 0 {
                    // 弟子等级 >= 敌人等级
                    base_rate + (level_diff as f64 * 0.10)
                } else {
                    // 弟子等级 < 敌人等级
                    base_rate + (level_diff as f64 * 0.15)
                };

                rate.clamp(0.05, 0.95)
            }
            _ => 0.8, // 非战斗任务默认 80% 成功率
        }
    }

    /// 获取战斗任务的敌人等级
    pub fn get_enemy_level(&self) -> Option<u32> {
        match &self.task_type {
            TaskType::Combat(combat) => Some(combat.enemy_level),
            _ => None,
        }
    }

    /// 获取任务难度（用于奖励计算）
    /// 返回难度值，范围通常为 0-100
    pub fn get_difficulty(&self) -> u32 {
        match &self.task_type {
            TaskType::Gathering(g) => g.difficulty,
            TaskType::Combat(c) => c.difficulty.max(c.enemy_level),  // 取战斗难度和敌人等级的最大值
            TaskType::Exploration(e) => e.danger_level,
            TaskType::Auxiliary(_) => 20,  // 辅助任务默认中等难度
            TaskType::Investment(_) => 15,  // 投资任务难度较低
        }
    }
}
