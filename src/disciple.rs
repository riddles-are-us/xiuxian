use crate::cultivation::CultivationLevel;
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
    pub progress: u32,           // 当前等级进度 0-100
    pub completed_tasks: Vec<usize>, // 已完成任务ID
    pub pending_tasks: Vec<Task>,    // 待完成任务
}

impl CultivationProgress {
    pub fn new(level: CultivationLevel) -> Self {
        Self {
            current_level: level,
            progress: 0,
            completed_tasks: Vec::new(),
            pending_tasks: Vec::new(),
        }
    }

    /// 是否达到大圆满
    pub fn is_perfect(&self) -> bool {
        self.progress >= 100 && self.pending_tasks.is_empty()
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
    pub heritage: Option<Heritage>,
    pub dao_companion: Option<DaoCompanion>,
    pub children: Vec<usize>, // 子女ID列表
    pub current_task: Option<String>, // 当前执行的任务名称
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
            heritage: None,
            dao_companion: None,
            children: Vec::new(),
            current_task: None,
        }
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.age < self.lifespan
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
    pub fn attempt_tribulation(&mut self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let success_rate = self.tribulation_success_rate();
        let roll: f32 = rng.gen();

        if roll < success_rate {
            if let Some(next_level) = self.cultivation.current_level.next() {
                self.cultivation.current_level = next_level;
                self.cultivation.progress = 0;
                self.cultivation.completed_tasks.clear();
                self.lifespan = next_level.base_lifespan();
                return true;
            }
        }

        false
    }

    /// 尝试突破
    pub fn breakthrough(&mut self) -> bool {
        if self.cultivation.is_perfect() {
            // 如果需要渡劫，则进行渡劫
            if self.cultivation.current_level.requires_tribulation() {
                return self.attempt_tribulation();
            } else {
                // 不需要渡劫的直接突破
                if let Some(next_level) = self.cultivation.current_level.next() {
                    self.cultivation.current_level = next_level;
                    self.cultivation.progress = 0;
                    self.cultivation.completed_tasks.clear();
                    self.lifespan = next_level.base_lifespan();
                    return true;
                }
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

        self.cultivation.progress = (self.cultivation.progress + actual_progress).min(100);
        self.cultivation.completed_tasks.push(task.id);

        actual_progress
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
