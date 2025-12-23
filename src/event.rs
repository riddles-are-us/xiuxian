use crate::disciple::{Disciple, DiscipleType, Talent, TalentType};
use crate::sect::Sect;
use crate::task::Task;
use crate::relationship::{RelationDimension, RelationLevel};
use rand::Rng;

/// 游戏事件
#[derive(Debug, Clone)]
pub enum GameEvent {
    TaskAvailable(Vec<Task>),      // 任务可用
    TaskCompleted(TaskResult),     // 任务完成
    DiscipleRecruited(usize),      // 收徒
    DiscipleBreakthrough(usize),   // 弟子突破
    DiscipleTribulation(usize, bool), // 弟子渡劫 (弟子ID, 是否成功)
    DiscipleDeath(usize),          // 弟子死亡
    YearlyIncome(u32),             // 年度收入
    MapUpdate,                     // 地图更新
    ChildBorn(usize, usize),       // 子女出生 (父母ID)
    ChildComeOfAge(usize),         // 子女成年

    // 关系事件
    RelationshipLevelUp {          // 关系等级提升
        disciple_id: usize,
        target_id: usize,
        dimension: RelationDimension,
        new_level: RelationLevel,
    },
    BecameDaoCompanion(usize, usize),     // 成为道侣
    BecameMasterDisciple(usize, usize),   // 建立师徒关系 (师父ID, 徒弟ID)
}

/// 任务结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: usize,
    pub disciple_id: usize,
    pub disciple_name: String,
    pub success: bool,
    pub resources_gained: u32,
    pub reputation_gained: i32,
    pub progress_gained: u32,
    pub disciple_died: bool,  // 弟子是否死亡（战斗任务失败）
}

/// 事件系统
pub struct EventSystem {
    pub events: Vec<GameEvent>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// 添加事件
    pub fn add_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    /// 处理所有事件
    pub fn process_events(&mut self, sect: &mut Sect) {
        let events = std::mem::take(&mut self.events);

        for event in events {
            match event {
                GameEvent::TaskCompleted(result) => {
                    self.handle_task_completed(sect, result);
                }
                GameEvent::DiscipleBreakthrough(id) => {
                    self.handle_breakthrough(sect, id);
                }
                GameEvent::DiscipleTribulation(id, success) => {
                    self.handle_tribulation(sect, id, success);
                }
                GameEvent::DiscipleDeath(id) => {
                    sect.handle_disciple_death(id);
                }
                GameEvent::YearlyIncome(amount) => {
                    sect.add_resources(amount);
                    println!("年度收入：{} 资源", amount);
                }
                GameEvent::DiscipleRecruited(id) => {
                    if let Some(disciple) = sect.disciples.iter().find(|d| d.id == id) {
                        println!("收徒：{}（{}）", disciple.name, disciple.disciple_type_str());
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_task_completed(&self, sect: &mut Sect, result: TaskResult) {
        if result.success {
            sect.add_resources(result.resources_gained);
            sect.add_reputation(result.reputation_gained);

            if let Some(disciple) = sect.disciples.iter().find(|d| d.id == result.disciple_id) {
                println!(
                    "{}完成任务，获得 {} 资源，{} 声望，{} 修为进度",
                    disciple.name,
                    result.resources_gained,
                    result.reputation_gained,
                    result.progress_gained
                );
            }
        }
    }

    fn handle_breakthrough(&self, sect: &mut Sect, disciple_id: usize) {
        if let Some(disciple) = sect.disciples.iter().find(|d| d.id == disciple_id) {
            println!(
                "{}成功突破至{}期！",
                disciple.name, disciple.cultivation.current_level
            );
        }
    }

    fn handle_tribulation(&self, sect: &mut Sect, disciple_id: usize, success: bool) {
        if let Some(disciple) = sect.disciples.iter().find(|d| d.id == disciple_id) {
            if success {
                println!(
                    "{}成功渡劫，晋升至{}期！",
                    disciple.name, disciple.cultivation.current_level
                );
            } else {
                println!("{}渡劫失败，身死道消...", disciple.name);
                self.add_event_later(GameEvent::DiscipleDeath(disciple_id));
            }
        }
    }

    fn add_event_later(&self, _event: GameEvent) {
        // 这里可以添加延迟事件处理
    }
}

/// 招募系统
pub struct RecruitmentSystem {
    next_disciple_id: usize,
}

impl RecruitmentSystem {
    pub fn new() -> Self {
        Self {
            next_disciple_id: 0,
        }
    }

    /// 随机生成弟子
    pub fn generate_random_disciple(&mut self) -> Disciple {
        let mut rng = rand::thread_rng();

        let names = vec![
            "张三", "李四", "王五", "赵六", "陈七", "林八", "周九", "吴十",
            "云飞扬", "剑无心", "莫问天", "风清扬", "叶孤城", "独孤求败",
        ];

        let name = names[rng.gen_range(0..names.len())].to_string();

        // 随机生成资质
        let num_talents = rng.gen_range(1..4);
        let mut talents = Vec::new();

        let all_talents = vec![
            TalentType::Fire,
            TalentType::Water,
            TalentType::Wood,
            TalentType::Metal,
            TalentType::Earth,
            TalentType::Sword,
            TalentType::Alchemy,
            TalentType::Formation,
            TalentType::Medical,
        ];

        for _ in 0..num_talents {
            let talent_type = all_talents[rng.gen_range(0..all_talents.len())].clone();
            let level = rng.gen_range(1..8); // 1-7的资质等级

            talents.push(Talent {
                talent_type,
                level,
            });
        }

        // 随机决定弟子类型
        let disciple_type = match rng.gen_range(0..10) {
            0..=5 => DiscipleType::Outer,
            6..=8 => DiscipleType::Inner,
            _ => DiscipleType::Personal,
        };

        let id = self.next_disciple_id;
        self.next_disciple_id += 1;

        Disciple::new(id, name, disciple_type, talents)
    }

    /// 尝试招募弟子
    pub fn try_recruit(&mut self, sect: &Sect) -> Option<Disciple> {
        let mut rng = rand::thread_rng();

        // 根据声望决定招募概率（低概率，使招募成为稀有事件）
        let recruit_chance = if sect.reputation > 100 {
            0.15  // 15% - 约每7回合一次
        } else if sect.reputation > 50 {
            0.10  // 10% - 约每10回合一次
        } else if sect.reputation > 0 {
            0.05  // 5% - 约每20回合一次
        } else {
            0.02  // 2% - 约每50回合一次
        };

        if rng.gen_bool(recruit_chance) {
            Some(self.generate_random_disciple())
        } else {
            None
        }
    }
}

impl Disciple {
    pub fn disciple_type_str(&self) -> &str {
        match self.disciple_type {
            DiscipleType::Outer => "外门弟子",
            DiscipleType::Inner => "内门弟子",
            DiscipleType::Personal => "亲传弟子",
        }
    }
}
