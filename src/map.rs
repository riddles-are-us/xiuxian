use crate::task::{Task, TaskType, GatheringTask, CombatTask, ExplorationTask, AuxiliaryTask};
use crate::disciple::TalentType;
use serde::Serialize;

/// 地图元素类型
#[derive(Debug, Clone)]
pub enum MapElement {
    Village(Village),
    Faction(Faction),
    DangerousLocation(DangerousLocation),
    SecretRealm(SecretRealm),
    Monster(Monster),
}

/// 地图坐标
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// 带坐标的地图元素
#[derive(Debug, Clone)]
pub struct PositionedElement {
    pub element: MapElement,
    pub position: Position,
}

impl MapElement {
    /// 生成对应的任务
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        match self {
            MapElement::Village(v) => v.generate_tasks(task_id_start),
            MapElement::Faction(f) => f.generate_tasks(task_id_start),
            MapElement::DangerousLocation(d) => d.generate_tasks(task_id_start),
            MapElement::SecretRealm(s) => s.generate_tasks(task_id_start),
            MapElement::Monster(m) => m.generate_tasks(task_id_start),
        }
    }

    /// 获取资源供给
    pub fn get_resource_income(&self, reputation: i32) -> u32 {
        match self {
            MapElement::Village(v) => v.get_income(reputation),
            MapElement::Faction(f) => f.get_income(reputation),
            _ => 0,
        }
    }
}

/// 村庄
#[derive(Debug, Clone)]
pub struct Village {
    pub name: String,
    pub population: u32,
    pub prosperity: u32, // 繁荣度
}

impl Village {
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        let mut tasks = Vec::new();

        // 采集任务
        let mut task = Task::new(
            task_id_start,
            format!("在{}采集灵药", self.name),
            TaskType::Gathering(GatheringTask {
                resource_type: "灵药".to_string(),
                difficulty: 1,
            }),
            5,
            10,
        );
        task.reputation_reward = 5;
        tasks.push(task);

        // 守卫任务
        let mut task = Task::new(
            task_id_start + 1,
            format!("守卫{}", self.name),
            TaskType::Auxiliary(AuxiliaryTask {
                task_name: "守卫村庄".to_string(),
                skill_required: None,
            }),
            3,
            5,
        );
        task.reputation_reward = 10;
        task.dao_heart_impact = 2;
        tasks.push(task);

        // 行医任务
        let mut task = Task::new(
            task_id_start + 2,
            format!("在{}行医", self.name),
            TaskType::Auxiliary(AuxiliaryTask {
                task_name: "行医救人".to_string(),
                skill_required: Some(TalentType::Medical),
            }),
            5,
            8,
        );
        task.reputation_reward = 15;
        task.dao_heart_impact = 5;
        tasks.push(task);

        tasks
    }

    pub fn get_income(&self, reputation: i32) -> u32 {
        let base = self.prosperity / 10;
        let bonus = if reputation > 100 {
            reputation as u32 / 20
        } else if reputation > 50 {
            reputation as u32 / 50
        } else {
            0
        };
        base + bonus
    }
}

/// 势力
#[derive(Debug, Clone)]
pub struct Faction {
    pub name: String,
    pub power_level: u32,
    pub relationship: i32, // 关系 -100 到 100
}

impl Faction {
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        let mut tasks = Vec::new();

        if self.relationship >= 0 {
            // 友好势力的交流任务
            let mut task = Task::new(
                task_id_start,
                format!("与{}交流", self.name),
                TaskType::Auxiliary(AuxiliaryTask {
                    task_name: "势力交流".to_string(),
                    skill_required: None,
                }),
                8,
                15,
            );
            task.reputation_reward = 20;
            tasks.push(task);
        } else if self.relationship < -30 {
            // 敌对势力的镇压任务
            let mut task = Task::new(
                task_id_start,
                format!("镇压{}", self.name),
                TaskType::Combat(CombatTask {
                    enemy_name: self.name.clone(),
                    enemy_level: self.power_level,
                    difficulty: self.power_level,
                }),
                15,
                30,
            );
            task.reputation_reward = 30;
            task.dao_heart_impact = -5;
            tasks.push(task);
        }

        tasks
    }

    pub fn get_income(&self, reputation: i32) -> u32 {
        if self.relationship > 50 && reputation > 80 {
            self.power_level * 5
        } else if self.relationship > 0 {
            self.power_level * 2
        } else {
            0
        }
    }
}

/// 险要之地
#[derive(Debug, Clone)]
pub struct DangerousLocation {
    pub name: String,
    pub danger_level: u32,
}

impl DangerousLocation {
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        vec![Task::new(
            task_id_start,
            format!("游历{}", self.name),
            TaskType::Exploration(ExplorationTask {
                location: self.name.clone(),
                danger_level: self.danger_level,
            }),
            10,
            20,
        )]
    }
}

/// 秘境
#[derive(Debug, Clone)]
pub struct SecretRealm {
    pub name: String,
    pub realm_type: TalentType, // 秘境类型，对应某种资质
    pub difficulty: u32,
}

impl SecretRealm {
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        vec![Task::new(
            task_id_start,
            format!("探索秘境：{}", self.name),
            TaskType::Exploration(ExplorationTask {
                location: self.name.clone(),
                danger_level: self.difficulty,
            }),
            20,
            50,
        )]
    }
}

/// 怪物
#[derive(Debug, Clone)]
pub struct Monster {
    pub name: String,
    pub level: u32,
    pub is_demon: bool, // 是否成魔
}

impl Monster {
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        let mut task = Task::new(
            task_id_start,
            format!("讨伐{}", self.name),
            TaskType::Combat(CombatTask {
                enemy_name: self.name.clone(),
                enemy_level: self.level,
                difficulty: self.level,
            }),
            15,
            40,
        );
        task.reputation_reward = 25;
        task.dao_heart_impact = 3;
        vec![task]
    }

    /// 怪物成长
    pub fn grow(&mut self) {
        self.level += 1;
        if self.level >= 100 {
            self.is_demon = true;
        }
    }
}

/// 游戏地图
#[derive(Debug)]
pub struct GameMap {
    pub elements: Vec<PositionedElement>,
    pub width: i32,
    pub height: i32,
}

impl GameMap {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            width: 20,  // 地图宽度
            height: 20, // 地图高度
        }
    }

    /// 初始化地图
    pub fn initialize(&mut self) {
        // 添加初始村庄
        self.elements.push(PositionedElement {
            element: MapElement::Village(Village {
                name: "清风镇".to_string(),
                population: 1000,
                prosperity: 50,
            }),
            position: Position { x: 5, y: 5 },
        });

        self.elements.push(PositionedElement {
            element: MapElement::Village(Village {
                name: "灵泉村".to_string(),
                population: 500,
                prosperity: 30,
            }),
            position: Position { x: 15, y: 8 },
        });

        // 添加初始势力
        self.elements.push(PositionedElement {
            element: MapElement::Faction(Faction {
                name: "青云派".to_string(),
                power_level: 3,
                relationship: 20,
            }),
            position: Position { x: 10, y: 10 },
        });

        // 添加险要之地
        self.elements.push(PositionedElement {
            element: MapElement::DangerousLocation(DangerousLocation {
                name: "迷雾森林".to_string(),
                danger_level: 20,
            }),
            position: Position { x: 3, y: 15 },
        });

        // 添加秘境
        self.elements.push(PositionedElement {
            element: MapElement::SecretRealm(SecretRealm {
                name: "火焰洞窟".to_string(),
                realm_type: TalentType::Fire,
                difficulty: 30,
            }),
            position: Position { x: 17, y: 3 },
        });

        // 添加初始怪物
        self.elements.push(PositionedElement {
            element: MapElement::Monster(Monster {
                name: "噬魂虎".to_string(),
                level: 2,
                is_demon: false,
            }),
            position: Position { x: 8, y: 12 },
        });
    }

    /// 获取所有可用任务
    pub fn get_available_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        let mut task_id = 0;

        for positioned in &self.elements {
            let element_tasks = positioned.element.generate_tasks(task_id);
            task_id += element_tasks.len();
            tasks.extend(element_tasks);
        }

        tasks
    }

    /// 计算总资源收入
    pub fn calculate_income(&self, reputation: i32) -> u32 {
        self.elements
            .iter()
            .map(|positioned| positioned.element.get_resource_income(reputation))
            .sum()
    }

    /// 更新地图（新事件、怪物成长等）
    pub fn update(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // 怪物可能成长
        for positioned in &mut self.elements {
            if let MapElement::Monster(monster) = &mut positioned.element {
                if rng.gen_bool(0.1) {
                    // 10%概率成长
                    monster.grow();
                }
            }
        }

        // 可能出现新的怪物
        if rng.gen_bool(0.2) {
            // 20%概率
            // 随机位置
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            self.elements.push(PositionedElement {
                element: MapElement::Monster(Monster {
                    name: format!("妖兽{}", rng.gen_range(1..100)),
                    level: rng.gen_range(1..5),
                    is_demon: false,
                }),
                position: Position { x, y },
            });
        }
    }

    /// 检查是否有怪物成魔
    pub fn has_demon(&self) -> bool {
        self.elements.iter().any(|positioned| {
            if let MapElement::Monster(m) = &positioned.element {
                m.is_demon
            } else {
                false
            }
        })
    }
}
