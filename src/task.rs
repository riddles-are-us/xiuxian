use crate::disciple::TalentType;

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
    pub enemy_name: String,
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
}

impl Task {
    pub fn new(
        id: usize,
        name: String,
        task_type: TaskType,
        progress_reward: u32,
        resource_reward: u32,
    ) -> Self {
        // 根据任务类型设置默认执行时间
        let duration = match &task_type {
            TaskType::Gathering(_) => 1,      // 采集任务1回合
            TaskType::Combat(_) => 2,         // 战斗任务2回合
            TaskType::Exploration(_) => 3,    // 探索任务3回合
            TaskType::Auxiliary(_) => 1,      // 辅助任务1回合
            TaskType::Investment(_) => 4,     // 投资任务4回合
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
        }
    }

    /// 检查任务是否已失效
    pub fn is_expired(&self, current_turn: u32) -> bool {
        current_turn >= self.created_turn + self.expiry_turns
    }

    /// 检查弟子是否适合此任务
    pub fn is_suitable_for_disciple(&self, disciple: &crate::disciple::Disciple) -> bool {
        match &self.task_type {
            TaskType::Combat(combat) => {
                // 检查战斗力是否足够
                disciple.cultivation.current_level as u32 >= combat.enemy_level
            }
            TaskType::Exploration(exploration) => {
                // 检查修为是否足够应对危险
                disciple.cultivation.current_level as u32 * 10 >= exploration.danger_level
            }
            TaskType::Auxiliary(auxiliary) => {
                // 检查是否有对应的资质
                if let Some(ref skill) = auxiliary.skill_required {
                    disciple.talents.iter().any(|t| &t.talent_type == skill)
                } else {
                    true
                }
            }
            _ => true,
        }
    }
}
