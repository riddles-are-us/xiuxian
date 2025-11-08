use crate::cultivation::{CultivationLevel, SubLevel, CultivationPath};
use crate::task::{Task, TaskType};

/// 弟子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscipleType {
    Outer,      // 外门
    Inner,      // 内门
    Personal,   // 亲传
}

/// 资质类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TalentType {
    Fire,           // 火灵根
    Water,          // 水灵根
    Wood,           // 木灵根
    Metal,          // 金灵根
    Earth,          // 土灵根
    Thunder,        // 雷灵根
    Ice,            // 冰灵根
    Wind,           // 风灵根
    Sword,          // 剑道天赋
    Alchemy,        // 炼丹天赋
    Formation,      // 阵法天赋
    Beast,          // 御兽天赋
    Medical,        // 医道天赋
}

/// 资质
#[derive(Debug, Clone)]
pub struct Talent {
    pub talent_type: TalentType,
    pub level: u32, // 资质等级 1-10
}

/// 传承
#[derive(Debug, Clone)]
pub struct Heritage {
    pub name: String,
    pub level: CultivationLevel,
    pub tribulation_bonus: f32, // 渡劫成功率加成
}

/// 道侣关系
#[derive(Debug, Clone)]
pub struct DaoCompanion {
    pub companion_id: usize,
    pub affinity: u32, // 亲密度
}

/// 修行进度
#[derive(Debug, Clone)]
pub struct CultivationProgress {
    pub current_level: CultivationLevel,
    pub sub_level: SubLevel,         // 小境界
    pub progress: u32,                // 当前小境界进度 0-100
    pub cultivation_path: Option<CultivationPath>,  // 修炼路径（大圆满时需要）
}

impl CultivationProgress {
    pub fn new(level: CultivationLevel) -> Self {
        Self {
            current_level: level,
            sub_level: SubLevel::Early,
            progress: 0,
            cultivation_path: Some(CultivationPath::new()), // 每个境界都有修炼路径
        }
    }

    /// 是否达到当前小境界的完成状态
    pub fn is_sub_level_complete(&self) -> bool {
        self.progress >= 100
    }

    /// 是否可以进行渡劫（大圆满且完成修炼路径）
    pub fn can_tribulate(&self) -> bool {
        self.sub_level == SubLevel::Perfect &&
        self.cultivation_path.as_ref().map(|p| p.is_completed()).unwrap_or(false)
    }

    /// 增加修为进度
    pub fn add_progress(&mut self, amount: u32) {
        self.progress = (self.progress + amount).min(100);
    }

    /// 尝试突破小境界
    pub fn try_sublevel_breakthrough(&mut self) -> bool {
        if !self.is_sub_level_complete() {
            return false;
        }

        match self.sub_level.next() {
            Some(next_sub) => {
                self.sub_level = next_sub;
                self.progress = 0;
                true
            }
            None => false,
        }
    }

    /// 尝试完成修炼路径任务（按任务类型）
    pub fn try_complete_path_task_by_type(&mut self, task_type: &str) -> bool {
        if let Some(ref mut path) = self.cultivation_path {
            path.complete_task_by_type(task_type)
        } else {
            false
        }
    }

    /// 突破到下一个大境界
    pub fn breakthrough_major_level(&mut self, new_level: CultivationLevel) {
        self.current_level = new_level;
        self.sub_level = SubLevel::Early;
        self.progress = 0;
        // 创建新的空修炼路径（任务将由InteractiveGame从当前任务中选择）
        self.cultivation_path = Some(CultivationPath::new());
    }
}

/// 弟子
#[derive(Debug, Clone)]
pub struct Disciple {
    pub id: usize,
    pub name: String,
    pub disciple_type: DiscipleType,
    pub cultivation: CultivationProgress,
    pub talents: Vec<Talent>,
    pub age: u32,
    pub lifespan: u32,
    pub dao_heart: u32,  // 道心 0-100
    pub energy: u32,     // 精力 0-100
    pub constitution: u32, // 体魄 0-100
    pub heritage: Option<Heritage>,
    pub dao_companion: Option<DaoCompanion>,
    pub children: Vec<usize>, // 子女ID列表
}

impl Disciple {
    pub fn new(id: usize, name: String, disciple_type: DiscipleType, talents: Vec<Talent>) -> Self {
        let lifespan = CultivationLevel::QiRefining.base_lifespan();

        Self {
            id,
            name,
            disciple_type,
            cultivation: CultivationProgress::new(CultivationLevel::QiRefining),
            talents,
            age: 16,
            lifespan,
            dao_heart: 50,
            energy: 100,        // 初始精力满值
            constitution: 100,  // 初始体魄满值
            heritage: None,
            dao_companion: None,
            children: Vec::new(),
        }
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.age < self.lifespan && self.constitution > 0
    }

    /// 是否达到仙道
    pub fn is_immortal(&self) -> bool {
        self.cultivation.current_level == CultivationLevel::Ascension
    }

    /// 增加年龄
    pub fn age_one_year(&mut self) {
        self.age += 1;
    }

    /// 获取资质加成
    pub fn get_talent_bonus(&self, talent_type: &TalentType) -> f32 {
        self.talents
            .iter()
            .find(|t| &t.talent_type == talent_type)
            .map(|t| t.level as f32 * 0.1)
            .unwrap_or(0.0)
    }

    /// 计算渡劫成功率
    pub fn tribulation_success_rate(&self) -> f32 {
        let base_rate = 0.3; // 基础成功率30%
        let dao_heart_bonus = self.dao_heart as f32 * 0.005; // 道心加成
        let heritage_bonus = self.heritage
            .as_ref()
            .map(|h| h.tribulation_bonus)
            .unwrap_or(0.0);

        (base_rate + dao_heart_bonus + heritage_bonus).min(0.95)
    }

    /// 尝试渡劫
    /// 渡劫
    pub fn attempt_tribulation(&mut self) -> bool {
        // 检查是否满足渡劫条件
        if !self.cultivation.can_tribulate() {
            return false;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let success_rate = self.tribulation_success_rate();
        let roll: f32 = rng.gen();

        if roll < success_rate {
            if let Some(next_level) = self.cultivation.current_level.next() {
                self.cultivation.breakthrough_major_level(next_level);
                self.lifespan = next_level.base_lifespan();
                return true;
            }
        }

        false
    }

    /// 尝试突破（现在只用于练气期突破到筑基）
    pub fn breakthrough(&mut self) -> bool {
        // 只有练气期可以直接突破（不需要渡劫）
        if self.cultivation.current_level == CultivationLevel::QiRefining &&
           self.cultivation.can_tribulate() {
            if let Some(next_level) = self.cultivation.current_level.next() {
                self.cultivation.breakthrough_major_level(next_level);
                self.lifespan = next_level.base_lifespan();
                return true;
            }
        }
        false
    }

    /// 完成任务
    pub fn complete_task(&mut self, task: &Task) -> u32 {
        let talent_bonus = match &task.task_type {
            TaskType::Gathering(_) => self.get_talent_bonus(&TalentType::Wood),
            TaskType::Combat(_) => self.get_talent_bonus(&TalentType::Sword),
            TaskType::Exploration(_) => 0.0,
            TaskType::Auxiliary(_) => self.get_talent_bonus(&TalentType::Formation),
            TaskType::Investment(_) => 0.0,
        };

        let base_progress = task.progress_reward;
        let actual_progress = (base_progress as f32 * (1.0 + talent_bonus)) as u32;

        // 添加修为进度
        self.cultivation.add_progress(actual_progress);

        // 尝试完成修炼路径任务（按任务类型）
        let task_type_str = match &task.task_type {
            TaskType::Combat(_) => "Combat",
            TaskType::Exploration(_) => "Exploration",
            TaskType::Gathering(_) => "Gathering",
            TaskType::Auxiliary(_) => "Auxiliary",
            TaskType::Investment(_) => "Investment",
        };
        self.cultivation.try_complete_path_task_by_type(task_type_str);

        // 自动检查并突破小境界
        if self.cultivation.is_sub_level_complete() {
            self.cultivation.try_sublevel_breakthrough();
        }

        actual_progress
    }

    /// 消耗精力
    pub fn consume_energy(&mut self, amount: u32) {
        if self.energy >= amount {
            self.energy -= amount;
        } else {
            self.energy = 0;
        }

        // 如果精力降到0，减少1年寿命
        if self.energy == 0 && self.lifespan > 0 {
            self.lifespan = self.lifespan.saturating_sub(1);
            println!("   ⚠️ {}精力耗尽，寿命减少1年（剩余{}年）", self.name, self.lifespan - self.age);
        }
    }

    /// 消耗体魄
    pub fn consume_constitution(&mut self, amount: u32) {
        if self.constitution >= amount {
            self.constitution -= amount;
        } else {
            self.constitution = 0;
        }

        // 如果体魄降到0，弟子会死亡（在is_alive中检查）
        if self.constitution == 0 {
            println!("   💀 {}体魄耗尽，死亡", self.name);
        }
    }

    /// 恢复精力
    pub fn restore_energy(&mut self, amount: u32) {
        self.energy = (self.energy + amount).min(100);
    }

    /// 恢复体魄
    pub fn restore_constitution(&mut self, amount: u32) {
        self.constitution = (self.constitution + amount).min(100);
    }

    /// 每回合自然恢复
    pub fn natural_recovery(&mut self) {
        // 每回合恢复5点精力和2点体魄
        self.restore_energy(5);
        self.restore_constitution(2);
    }

    /// 死亡后生成传承
    pub fn generate_heritage(&self) -> Option<Heritage> {
        if self.cultivation.current_level >= CultivationLevel::NascentSoul {
            Some(Heritage {
                name: format!("{}的传承", self.name),
                level: self.cultivation.current_level,
                tribulation_bonus: match self.cultivation.current_level {
                    CultivationLevel::NascentSoul => 0.1,
                    CultivationLevel::SpiritSevering => 0.15,
                    CultivationLevel::VoidRefinement => 0.2,
                    _ => 0.05,
                },
            })
        } else {
            None
        }
    }
}
