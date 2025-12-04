use crate::task::{Task, TaskType, GatheringTask, CombatTask, ExplorationTask, AuxiliaryTask};
use crate::disciple::TalentType;
use crate::config::{
    ConfigManager, TaskTemplateConfig, VillageTemplate, FactionTemplate,
    DangerousLocationTemplate, SecretRealmTemplate, MonsterTemplate,
};
use serde::Serialize;

/// 地图元素类型
#[derive(Debug, Clone)]
pub enum MapElement {
    Village(Village),
    Faction(Faction),
    DangerousLocation(DangerousLocation),
    SecretRealm(SecretRealm),
    Monster(Monster),
    Terrain(Terrain),  // 基础地形要素
}

/// 地形类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Mountain,  // 山
    Water,     // 水
    Forest,    // 林
    Plain,     // 平原
}

/// 地形要素（不产生任务）
#[derive(Debug, Clone)]
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub name: String,
}

/// 地图坐标
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// 带坐标的地图元素
#[derive(Debug, Clone)]
pub struct PositionedElement {
    pub element: MapElement,
    pub core_position: Position,        // 核心位置：用于交互（弟子需要到达此位置进行互动）
    pub positions: Vec<Position>,       // 占据的所有位置：用于碰撞检测（这些位置不可通行）
}

impl PositionedElement {
    /// 创建一个单格元素（核心位置和占据位置相同）
    pub fn new_single(element: MapElement, position: Position) -> Self {
        Self {
            element,
            core_position: position,
            positions: vec![position],
        }
    }

    /// 创建一个多格元素
    pub fn new_multi(element: MapElement, core_position: Position, positions: Vec<Position>) -> Self {
        Self {
            element,
            core_position,
            positions,
        }
    }

    /// 检查某个位置是否被此元素占据（用于碰撞检测）
    pub fn occupies_position(&self, pos: &Position) -> bool {
        self.positions.contains(pos)
    }

    /// 检查某个位置是否是核心位置（用于交互检测）
    pub fn is_core_position(&self, pos: &Position) -> bool {
        self.core_position == *pos
    }
}

impl MapElement {
    /// 获取地图元素的唯一标识符
    pub fn get_location_id(&self) -> String {
        match self {
            MapElement::Village(v) => format!("village_{}", v.name),
            MapElement::Faction(f) => format!("faction_{}", f.name),
            MapElement::DangerousLocation(d) => format!("danger_{}", d.name),
            MapElement::SecretRealm(s) => format!("realm_{}", s.name),
            MapElement::Monster(m) => format!("monster_{}", m.id),
            MapElement::Terrain(t) => format!("terrain_{}", t.name),
        }
    }

    /// 生成对应的任务
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        let location_id = self.get_location_id();
        let mut tasks = match self {
            MapElement::Village(v) => v.generate_tasks(task_id_start),
            MapElement::Faction(f) => f.generate_tasks(task_id_start),
            MapElement::DangerousLocation(d) => d.generate_tasks(task_id_start),
            MapElement::SecretRealm(s) => s.generate_tasks(task_id_start),
            MapElement::Monster(m) => m.generate_tasks(task_id_start),
            MapElement::Terrain(_) => Vec::new(),  // 地形不产生任务
        };

        // 为所有任务设置location_id
        for task in &mut tasks {
            task.location_id = Some(location_id.clone());
        }

        tasks
    }

    /// 获取资源供给
    pub fn get_resource_income(&self, reputation: i32) -> u32 {
        match self {
            MapElement::Village(v) => v.get_income(reputation),
            MapElement::Faction(f) => f.get_income(reputation),
            _ => 0,
        }
    }

    /// 检查是否可以被妖魔入侵（村庄、势力、秘境等）
    pub fn can_be_invaded(&self) -> bool {
        matches!(self,
            MapElement::Village(_) |
            MapElement::Faction(_) |
            MapElement::SecretRealm(_)
        )
    }

    /// 获取守卫任务名称
    pub fn get_defense_task_name(&self) -> Option<String> {
        match self {
            MapElement::Village(v) => Some(format!("守卫{}", v.name)),
            MapElement::Faction(f) => Some(format!("守卫{}", f.name)),
            MapElement::SecretRealm(s) => Some(format!("守卫秘境：{}", s.name)),
            _ => None,
        }
    }

    /// 检查是否是妖魔并且返回可变引用
    pub fn as_monster_mut(&mut self) -> Option<&mut Monster> {
        match self {
            MapElement::Monster(m) => Some(m),
            _ => None,
        }
    }

    /// 检查是否是妖魔并且返回不可变引用
    pub fn as_monster(&self) -> Option<&Monster> {
        match self {
            MapElement::Monster(m) => Some(m),
            _ => None,
        }
    }
}

/// 村庄
#[derive(Debug, Clone)]
pub struct Village {
    pub name: String,
    pub population: u32,
    pub prosperity: u32, // 繁荣度
    pub task_templates: Vec<TaskTemplateConfig>,
}

impl Village {
    pub fn from_template(template: &VillageTemplate) -> Self {
        Self {
            name: template.name.clone(),
            population: template.population,
            prosperity: template.prosperity,
            task_templates: template.task_templates.clone(),
        }
    }

    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        use rand::seq::SliceRandom;
        use std::collections::HashMap;

        // 按任务类型分组
        let mut templates_by_type: HashMap<String, Vec<&TaskTemplateConfig>> = HashMap::new();
        for template in &self.task_templates {
            templates_by_type
                .entry(template.task_type.clone())
                .or_insert_with(Vec::new)
                .push(template);
        }

        // 每种任务类型只随机选择一个模板
        let mut tasks = Vec::new();
        let mut rng = rand::thread_rng();
        let mut task_id = task_id_start;

        for (_task_type, template_list) in templates_by_type {
            if let Some(template) = template_list.choose(&mut rng) {
                if let Some(task) = self.generate_task_from_template(task_id, template) {
                    tasks.push(task);
                    task_id += 1;
                }
            }
        }

        tasks
    }

    fn generate_task_from_template(&self, task_id: usize, template: &TaskTemplateConfig) -> Option<Task> {
        let name = template.name_template.replace("{name}", &self.name);
        let task_type = parse_task_type(template)?;

        let mut task = Task::new(
            task_id,
            name,
            task_type,
            template.progress_reward,
            template.resource_reward,
        );
        task.reputation_reward = template.reputation_reward;
        task.dao_heart_impact = template.dao_heart_impact;

        Some(task)
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
    pub friendly_task_templates: Vec<TaskTemplateConfig>,
    pub hostile_task_templates: Vec<TaskTemplateConfig>,
}

impl Faction {
    pub fn from_template(template: &FactionTemplate) -> Self {
        Self {
            name: template.name.clone(),
            power_level: template.power_level,
            relationship: template.relationship,
            friendly_task_templates: template.friendly_task_templates.clone(),
            hostile_task_templates: template.hostile_task_templates.clone(),
        }
    }

    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        let mut tasks = Vec::new();

        if self.relationship >= 0 {
            // 使用友好任务模板
            for (i, template) in self.friendly_task_templates.iter().enumerate() {
                if let Some(task) = self.generate_task_from_template(task_id_start + i, template) {
                    tasks.push(task);
                }
            }
        } else if self.relationship < -30 {
            // 使用敌对任务模板
            for (i, template) in self.hostile_task_templates.iter().enumerate() {
                let task_type = match template.task_type.as_str() {
                    "Combat" => TaskType::Combat(CombatTask {
                        enemy_name: self.name.clone(),
                        enemy_level: self.power_level,
                        difficulty: template.difficulty.unwrap_or(self.power_level),
                    }),
                    _ => continue,
                };

                let name = template.name_template.replace("{name}", &self.name);
                let mut task = Task::new(
                    task_id_start + i,
                    name,
                    task_type,
                    template.progress_reward,
                    template.resource_reward,
                );
                task.reputation_reward = template.reputation_reward;
                task.dao_heart_impact = template.dao_heart_impact;

                tasks.push(task);
            }
        }

        tasks
    }

    fn generate_task_from_template(&self, task_id: usize, template: &TaskTemplateConfig) -> Option<Task> {
        let name = template.name_template.replace("{name}", &self.name);
        let task_type = parse_task_type(template)?;

        let mut task = Task::new(
            task_id,
            name,
            task_type,
            template.progress_reward,
            template.resource_reward,
        );
        task.reputation_reward = template.reputation_reward;
        task.dao_heart_impact = template.dao_heart_impact;

        Some(task)
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
    pub task_templates: Vec<TaskTemplateConfig>,
}

impl DangerousLocation {
    pub fn from_template(template: &DangerousLocationTemplate) -> Self {
        Self {
            name: template.name.clone(),
            danger_level: template.danger_level,
            task_templates: template.task_templates.clone(),
        }
    }

    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        use rand::seq::SliceRandom;
        use std::collections::HashMap;

        // 按任务类型分组
        let mut templates_by_type: HashMap<String, Vec<&TaskTemplateConfig>> = HashMap::new();
        for template in &self.task_templates {
            templates_by_type
                .entry(template.task_type.clone())
                .or_insert_with(Vec::new)
                .push(template);
        }

        // 每种任务类型只随机选择一个模板
        let mut tasks = Vec::new();
        let mut rng = rand::thread_rng();
        let mut task_id = task_id_start;

        for (_task_type, template_list) in templates_by_type {
            if let Some(template) = template_list.choose(&mut rng) {
                if let Some(task) = self.generate_task_from_template(task_id, template) {
                    tasks.push(task);
                    task_id += 1;
                }
            }
        }

        tasks
    }

    fn generate_task_from_template(&self, task_id: usize, template: &TaskTemplateConfig) -> Option<Task> {
        let name = template.name_template.replace("{name}", &self.name);
        let task_type = parse_task_type(template)?;

        let mut task = Task::new(
            task_id,
            name,
            task_type,
            template.progress_reward,
            template.resource_reward,
        );
        task.reputation_reward = template.reputation_reward;
        task.dao_heart_impact = template.dao_heart_impact;

        Some(task)
    }
}

/// 秘境
#[derive(Debug, Clone)]
pub struct SecretRealm {
    pub name: String,
    pub realm_type: TalentType, // 秘境类型，对应某种资质
    pub difficulty: u32,
    pub task_templates: Vec<TaskTemplateConfig>,
}

impl SecretRealm {
    pub fn from_template(template: &SecretRealmTemplate) -> Self {
        let realm_type = parse_talent_type(&template.realm_type);
        Self {
            name: template.name.clone(),
            realm_type,
            difficulty: template.difficulty,
            task_templates: template.task_templates.clone(),
        }
    }

    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        use rand::seq::SliceRandom;
        use std::collections::HashMap;

        // 按任务类型分组
        let mut templates_by_type: HashMap<String, Vec<&TaskTemplateConfig>> = HashMap::new();
        for template in &self.task_templates {
            templates_by_type
                .entry(template.task_type.clone())
                .or_insert_with(Vec::new)
                .push(template);
        }

        // 每种任务类型只随机选择一个模板
        let mut tasks = Vec::new();
        let mut rng = rand::thread_rng();
        let mut task_id = task_id_start;

        for (_task_type, template_list) in templates_by_type {
            if let Some(template) = template_list.choose(&mut rng) {
                if let Some(task) = self.generate_task_from_template(task_id, template) {
                    tasks.push(task);
                    task_id += 1;
                }
            }
        }

        tasks
    }

    fn generate_task_from_template(&self, task_id: usize, template: &TaskTemplateConfig) -> Option<Task> {
        let name = template.name_template.replace("{name}", &self.name);
        let task_type = parse_task_type(template)?;

        let mut task = Task::new(
            task_id,
            name,
            task_type,
            template.progress_reward,
            template.resource_reward,
        );
        task.reputation_reward = template.reputation_reward;
        task.dao_heart_impact = template.dao_heart_impact;

        Some(task)
    }
}

/// 怪物/妖魔
#[derive(Debug, Clone)]
pub struct Monster {
    pub id: usize, // 唯一标识符
    pub name: String,
    pub level: u32,
    pub is_demon: bool, // 是否成魔
    pub growth_rate: f64, // 成长速率
    pub task_templates: Vec<TaskTemplateConfig>,
    pub current_task_id: Option<usize>, // 当前关联的任务ID（实现一对一关系）
    pub is_being_fought: bool, // 是否正在被战斗
    pub invaded_location_id: Option<String>, // 当前入侵的地点ID
    pub has_active_defense_task: bool, // 是否有正在执行的守卫任务（用于锁定移动）
}

// 全局妖魔ID计数器
static NEXT_MONSTER_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Monster {
    pub fn from_template(template: &MonsterTemplate) -> Self {
        let id = NEXT_MONSTER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            id,
            name: template.name.clone(),
            level: template.level,
            is_demon: template.is_demon,
            growth_rate: template.growth_rate,
            task_templates: template.task_templates.clone(),
            current_task_id: None,
            is_being_fought: false,
            invaded_location_id: None,
            has_active_defense_task: false,
        }
    }

    /// 创建新妖魔（用于随机生成）
    pub fn new(name: String, level: u32, task_templates: Vec<TaskTemplateConfig>) -> Self {
        let id = NEXT_MONSTER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            id,
            name,
            level,
            is_demon: false,
            growth_rate: 0.1,
            task_templates,
            current_task_id: None,
            is_being_fought: false,
            invaded_location_id: None,
            has_active_defense_task: false,
        }
    }

    /// 生成任务（只有在没有关联任务时才生成）
    pub fn generate_tasks(&self, task_id_start: usize) -> Vec<Task> {
        // 如果妖魔已经有关联的任务，则不生成新任务
        if self.current_task_id.is_some() {
            return Vec::new();
        }

        let mut tasks = Vec::new();
        for (i, template) in self.task_templates.iter().enumerate() {
            if let Some(task) = self.generate_task_from_template(task_id_start + i, template) {
                tasks.push(task);
            }
        }
        tasks
    }

    fn generate_task_from_template(&self, task_id: usize, template: &TaskTemplateConfig) -> Option<Task> {
        // 在任务名称中包含妖魔ID，确保唯一性
        let display_name = format!("{}#{}", self.name, self.id);
        let name = template.name_template.replace("{name}", &display_name);

        let task_type = match template.task_type.as_str() {
            "Combat" => TaskType::Combat(CombatTask {
                enemy_name: display_name,
                enemy_level: self.level,
                difficulty: template.difficulty.unwrap_or(self.level),
            }),
            _ => return None,
        };

        let mut task = Task::new(
            task_id,
            name,
            task_type,
            template.progress_reward,
            template.resource_reward,
        );
        task.reputation_reward = template.reputation_reward;
        task.dao_heart_impact = template.dao_heart_impact;

        Some(task)
    }

    /// 怪物成长
    pub fn grow(&mut self) {
        self.level += 1;
        if self.level >= 100 {
            self.is_demon = true;
        }
    }

    /// 设置关联任务
    pub fn set_task(&mut self, task_id: usize) {
        self.current_task_id = Some(task_id);
    }

    /// 清除关联任务
    pub fn clear_task(&mut self) {
        self.current_task_id = None;
    }

    /// 检查是否有关联任务
    pub fn has_task(&self) -> bool {
        self.current_task_id.is_some()
    }
}

/// 静态地图数据（用于早期版本）
#[derive(Debug, Clone)]
pub struct StaticMapData {
    pub width: i32,
    pub height: i32,
    pub villages: Vec<StaticVillage>,
    pub factions: Vec<StaticFaction>,
    pub dangerous_locations: Vec<StaticDangerousLocation>,
    pub secret_realms: Vec<StaticSecretRealm>,
    pub monsters: Vec<StaticMonster>,
    pub terrains: Vec<StaticTerrain>,
}

/// 静态村庄数据
#[derive(Debug, Clone)]
pub struct StaticVillage {
    pub name: String,
    pub core_position: Position,        // 交互位置（村庄入口）
    pub positions: Vec<Position>,       // 占据的所有位置（村庄范围）
    pub population: u32,
    pub prosperity: u32,
}

/// 静态势力数据
#[derive(Debug, Clone)]
pub struct StaticFaction {
    pub name: String,
    pub core_position: Position,        // 交互位置（宗门大门）
    pub positions: Vec<Position>,       // 占据的所有位置（宗门范围）
    pub power_level: u32,
    pub relationship: i32,
}

/// 静态险地数据
#[derive(Debug, Clone)]
pub struct StaticDangerousLocation {
    pub name: String,
    pub core_position: Position,        // 交互位置（入口）
    pub positions: Vec<Position>,       // 占据的所有位置（险地范围）
    pub danger_level: u32,
}

/// 静态秘境数据
#[derive(Debug, Clone)]
pub struct StaticSecretRealm {
    pub name: String,
    pub core_position: Position,        // 交互位置（秘境入口）
    pub positions: Vec<Position>,       // 占据的所有位置（秘境范围）
    pub realm_type: String, // "Fire", "Water", etc.
    pub difficulty: u32,
}

/// 静态妖魔数据
#[derive(Debug, Clone)]
pub struct StaticMonster {
    pub name: String,
    pub core_position: Position,        // 妖魔位置（单格，会移动）
    pub level: u32,
    pub is_demon: bool,
}

/// 静态地形数据（地形是不可通行的）
#[derive(Debug, Clone)]
pub struct StaticTerrain {
    pub name: String,
    pub core_position: Position,        // 地形中心位置
    pub positions: Vec<Position>,       // 占据的所有位置（不可通行）
    pub terrain_type: String, // "Mountain", "Water", "Forest", "Plain"
}

/// 游戏地图
#[derive(Debug)]
pub struct GameMap {
    pub elements: Vec<PositionedElement>,
    pub width: i32,
    pub height: i32,
    pub config: ConfigManager,
    pub static_data: Option<StaticMapData>, // 静态地图数据
}

impl GameMap {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            width: 20,  // 地图宽度
            height: 20, // 地图高度
            config: ConfigManager::create_default(),
            static_data: None,
        }
    }

    /// 创建默认的静态地图数据（供用户修改）
    pub fn create_default_static_map() -> StaticMapData {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // 山脉随机选择1x1或1x2 (横向)
        let mountain_positions = if rng.gen_bool(0.5) {
            // 1x1 山脉
            vec![Position { x: 2, y: 2 }]
        } else {
            // 1x2 山脉 (横向)
            vec![
                Position { x: 2, y: 2 },
                Position { x: 3, y: 2 },
            ]
        };

        let mountain2_positions = if rng.gen_bool(0.5) {
            // 1x1 山脉
            vec![Position { x: 17, y: 3 }]
        } else {
            // 1x2 山脉 (横向)
            vec![
                Position { x: 16, y: 3 },
                Position { x: 17, y: 3 },
            ]
        };

        StaticMapData {
            width: 20,
            height: 20,
            villages: vec![
                StaticVillage {
                    name: "青云村".to_string(),
                    core_position: Position { x: 5, y: 5 },
                    // 村庄占据1x1
                    positions: vec![Position { x: 5, y: 5 }],
                    population: 1000,
                    prosperity: 50,
                },
                StaticVillage {
                    name: "桃花村".to_string(),
                    core_position: Position { x: 15, y: 10 },
                    // 村庄占据1x1
                    positions: vec![Position { x: 15, y: 10 }],
                    population: 800,
                    prosperity: 40,
                },
            ],
            factions: vec![
                StaticFaction {
                    name: "天剑宗".to_string(),
                    core_position: Position { x: 10, y: 15 },
                    // 宗门占据2x2区域
                    positions: vec![
                        Position { x: 10, y: 15 },
                        Position { x: 11, y: 15 },
                        Position { x: 10, y: 16 },
                        Position { x: 11, y: 16 },
                    ],
                    power_level: 50,
                    relationship: 20,
                },
            ],
            dangerous_locations: vec![
                StaticDangerousLocation {
                    name: "黑风谷".to_string(),
                    core_position: Position { x: 3, y: 12 },
                    // 险地占据1x1
                    positions: vec![Position { x: 3, y: 12 }],
                    danger_level: 30,
                },
            ],
            secret_realms: vec![
                StaticSecretRealm {
                    name: "烈焰洞天".to_string(),
                    core_position: Position { x: 18, y: 18 },
                    // 秘境只占1x1（入口）
                    positions: vec![Position { x: 18, y: 18 }],
                    realm_type: "Fire".to_string(),
                    difficulty: 40,
                },
            ],
            monsters: vec![
                StaticMonster {
                    name: "山野妖兽".to_string(),
                    core_position: Position { x: 7, y: 8 },
                    // 妖魔只占1x1（会移动）
                    level: 10,
                    is_demon: false,
                },
            ],
            terrains: vec![
                StaticTerrain {
                    name: "太行山".to_string(),
                    core_position: mountain_positions[0],
                    // 山脉随机1x1或1x2（不可通行）
                    positions: mountain_positions,
                    terrain_type: "Mountain".to_string(),
                },
                StaticTerrain {
                    name: "昆仑山".to_string(),
                    core_position: mountain2_positions[0],
                    // 山脉随机1x1或1x2（不可通行）
                    positions: mountain2_positions,
                    terrain_type: "Mountain".to_string(),
                },
                StaticTerrain {
                    name: "玄水湖".to_string(),
                    core_position: Position { x: 12, y: 6 },
                    // 湖泊占据1x1（不可通行）
                    positions: vec![Position { x: 12, y: 6 }],
                    terrain_type: "Water".to_string(),
                },
                StaticTerrain {
                    name: "青松林".to_string(),
                    core_position: Position { x: 8, y: 1 },
                    // 森林占据1x1
                    positions: vec![Position { x: 8, y: 1 }],
                    terrain_type: "Forest".to_string(),
                },
            ],
        }
    }

    /// 检查静态地图数据中是否有位置冲突
    fn check_position_collisions(static_data: &StaticMapData) -> Result<(), String> {
        use std::collections::HashSet;
        let mut occupied_positions: HashSet<Position> = HashSet::new();

        // 检查村庄
        for village in &static_data.villages {
            for pos in &village.positions {
                if occupied_positions.contains(pos) {
                    return Err(format!("位置冲突: 村庄 '{}' 在位置 ({}, {}) 与其他元素重叠",
                        village.name, pos.x, pos.y));
                }
                occupied_positions.insert(*pos);
            }
        }

        // 检查势力
        for faction in &static_data.factions {
            for pos in &faction.positions {
                if occupied_positions.contains(pos) {
                    return Err(format!("位置冲突: 势力 '{}' 在位置 ({}, {}) 与其他元素重叠",
                        faction.name, pos.x, pos.y));
                }
                occupied_positions.insert(*pos);
            }
        }

        // 检查险地
        for danger in &static_data.dangerous_locations {
            for pos in &danger.positions {
                if occupied_positions.contains(pos) {
                    return Err(format!("位置冲突: 险地 '{}' 在位置 ({}, {}) 与其他元素重叠",
                        danger.name, pos.x, pos.y));
                }
                occupied_positions.insert(*pos);
            }
        }

        // 检查秘境
        for realm in &static_data.secret_realms {
            for pos in &realm.positions {
                if occupied_positions.contains(pos) {
                    return Err(format!("位置冲突: 秘境 '{}' 在位置 ({}, {}) 与其他元素重叠",
                        realm.name, pos.x, pos.y));
                }
                occupied_positions.insert(*pos);
            }
        }

        // 检查妖魔
        for monster in &static_data.monsters {
            let pos = &monster.core_position;
            if occupied_positions.contains(pos) {
                return Err(format!("位置冲突: 妖魔 '{}' 在位置 ({}, {}) 与其他元素重叠",
                    monster.name, pos.x, pos.y));
            }
            occupied_positions.insert(*pos);
        }

        // 检查地形
        for terrain in &static_data.terrains {
            for pos in &terrain.positions {
                if occupied_positions.contains(pos) {
                    return Err(format!("位置冲突: 地形 '{}' 在位置 ({}, {}) 与其他元素重叠",
                        terrain.name, pos.x, pos.y));
                }
                occupied_positions.insert(*pos);
            }
        }

        Ok(())
    }

    /// 从静态数据初始化地图
    pub fn initialize_from_static(&mut self, static_data: StaticMapData) {
        // 检查位置冲突
        if let Err(error_msg) = Self::check_position_collisions(&static_data) {
            println!("⚠ 地图初始化警告: {}", error_msg);
            println!("   建议: 请调整元素位置以避免重叠");
        }

        // 加载配置（用于任务模板）
        match ConfigManager::load() {
            Ok(config) => {
                println!("✓ 成功加载配置文件");
                self.config = config;
            }
            Err(e) => {
                println!("⚠ 加载配置失败: {}, 使用默认配置", e);
                self.config = ConfigManager::create_default();
            }
        }

        self.width = static_data.width;
        self.height = static_data.height;
        self.elements.clear();

        // 加载地形（不可通行）
        for terrain_data in &static_data.terrains {
            let terrain_type = match terrain_data.terrain_type.as_str() {
                "Mountain" => TerrainType::Mountain,
                "Water" => TerrainType::Water,
                "Forest" => TerrainType::Forest,
                "Plain" => TerrainType::Plain,
                _ => TerrainType::Plain,
            };

            self.elements.push(PositionedElement::new_multi(
                MapElement::Terrain(Terrain {
                    terrain_type,
                    name: terrain_data.name.clone(),
                }),
                terrain_data.core_position,
                terrain_data.positions.clone(),
            ));
        }

        // 加载村庄（使用配置中的任务模板）
        for village_data in &static_data.villages {
            let task_templates = self.config.map_elements.villages
                .first()
                .map(|v| v.task_templates.clone())
                .unwrap_or_default();

            self.elements.push(PositionedElement::new_multi(
                MapElement::Village(Village {
                    name: village_data.name.clone(),
                    population: village_data.population,
                    prosperity: village_data.prosperity,
                    task_templates,
                }),
                village_data.core_position,
                village_data.positions.clone(),
            ));
        }

        // 加载势力（使用配置中的任务模板）
        for faction_data in &static_data.factions {
            let (friendly_templates, hostile_templates) = self.config.map_elements.factions
                .first()
                .map(|f| (f.friendly_task_templates.clone(), f.hostile_task_templates.clone()))
                .unwrap_or_default();

            self.elements.push(PositionedElement::new_multi(
                MapElement::Faction(Faction {
                    name: faction_data.name.clone(),
                    power_level: faction_data.power_level,
                    relationship: faction_data.relationship,
                    friendly_task_templates: friendly_templates,
                    hostile_task_templates: hostile_templates,
                }),
                faction_data.core_position,
                faction_data.positions.clone(),
            ));
        }

        // 加载险地（使用配置中的任务模板）
        for danger_data in &static_data.dangerous_locations {
            let task_templates = self.config.map_elements.dangerous_locations
                .first()
                .map(|d| d.task_templates.clone())
                .unwrap_or_default();

            self.elements.push(PositionedElement::new_multi(
                MapElement::DangerousLocation(DangerousLocation {
                    name: danger_data.name.clone(),
                    danger_level: danger_data.danger_level,
                    task_templates,
                }),
                danger_data.core_position,
                danger_data.positions.clone(),
            ));
        }

        // 加载秘境（使用配置中的任务模板）
        for realm_data in &static_data.secret_realms {
            let task_templates = self.config.map_elements.secret_realms
                .first()
                .map(|r| r.task_templates.clone())
                .unwrap_or_default();

            self.elements.push(PositionedElement::new_multi(
                MapElement::SecretRealm(SecretRealm {
                    name: realm_data.name.clone(),
                    realm_type: parse_talent_type(&realm_data.realm_type),
                    difficulty: realm_data.difficulty,
                    task_templates,
                }),
                realm_data.core_position,
                realm_data.positions.clone(),
            ));
        }

        // 加载妖魔（使用配置中的任务模板）
        // 妖魔是单格，会移动
        for monster_data in &static_data.monsters {
            let task_templates = self.config.monsters.monster_templates
                .first()
                .map(|m| m.task_templates.clone())
                .unwrap_or_default();

            let mut monster = Monster::new(
                monster_data.name.clone(),
                monster_data.level,
                task_templates,
            );
            monster.is_demon = monster_data.is_demon;

            self.elements.push(PositionedElement::new_single(
                MapElement::Monster(monster),
                monster_data.core_position,
            ));
        }

        self.static_data = Some(static_data);
        println!("✓ 成功从静态数据初始化地图");
    }

    /// 检查某个位置是否可通行（用于寻路和移动）
    pub fn is_position_passable(&self, pos: &Position) -> bool {
        // 检查是否超出地图边界
        if pos.x < 0 || pos.x >= self.width || pos.y < 0 || pos.y >= self.height {
            return false;
        }

        // 检查是否被任何元素占据（地形、建筑等都是不可通行的）
        for positioned in &self.elements {
            if positioned.occupies_position(pos) {
                // 地形是完全不可通行的
                if matches!(positioned.element, MapElement::Terrain(_)) {
                    return false;
                }
                // 其他建筑也不可通行（村庄、势力、险地、秘境）
                // 但妖魔不算障碍物（可以和妖魔在同一位置战斗）
                if !matches!(positioned.element, MapElement::Monster(_)) {
                    return false;
                }
            }
        }

        true
    }

    /// 获取指定位置的所有元素（用于交互）
    pub fn get_elements_at_core_position(&self, pos: &Position) -> Vec<&MapElement> {
        self.elements
            .iter()
            .filter(|p| p.is_core_position(pos))
            .map(|p| &p.element)
            .collect()
    }

    /* ===== 旧的程序化生成地图逻辑（已注释，保留用于未来参考） =====

    /// 初始化地图（从配置加载 - 旧版程序化生成）
    pub fn initialize(&mut self) {
        // 加载配置
        match ConfigManager::load() {
            Ok(config) => {
                println!("✓ 成功加载配置文件");
                self.config = config;
            }
            Err(e) => {
                println!("⚠ 加载配置失败: {}, 使用默认配置", e);
                self.config = ConfigManager::create_default();
            }
        }

        // 生成基础地形元素（随机位置）
        self.generate_terrain();

        // 从配置加载村庄（使用配置文件中的位置）
        for village_template in &self.config.map_elements.villages {
            self.elements.push(PositionedElement {
                element: MapElement::Village(Village::from_template(village_template)),
                position: Position {
                    x: village_template.position.x,
                    y: village_template.position.y,
                },
            });
        }

        // 从配置加载势力（使用配置文件中的位置）
        for faction_template in &self.config.map_elements.factions {
            self.elements.push(PositionedElement {
                element: MapElement::Faction(Faction::from_template(faction_template)),
                position: Position {
                    x: faction_template.position.x,
                    y: faction_template.position.y,
                },
            });
        }

        // 从配置加载险地（使用配置文件中的位置）
        for dangerous_template in &self.config.map_elements.dangerous_locations {
            self.elements.push(PositionedElement {
                element: MapElement::DangerousLocation(
                    DangerousLocation::from_template(dangerous_template)
                ),
                position: Position {
                    x: dangerous_template.position.x,
                    y: dangerous_template.position.y,
                },
            });
        }

        // 从配置加载秘境（使用配置文件中的位置）
        for realm_template in &self.config.map_elements.secret_realms {
            self.elements.push(PositionedElement {
                element: MapElement::SecretRealm(SecretRealm::from_template(realm_template)),
                position: Position {
                    x: realm_template.position.x,
                    y: realm_template.position.y,
                },
            });
        }

        // 从配置加载初始妖魔（使用配置文件中的位置）
        for monster_template in &self.config.monsters.monster_templates {
            if let Some(pos) = &monster_template.position {
                self.elements.push(PositionedElement {
                    element: MapElement::Monster(Monster::from_template(monster_template)),
                    position: Position {
                        x: pos.x,
                        y: pos.y,
                    },
                });
            }
        }
    }

    /// 生成基础地形元素（旧版 - 随机生成地形位置）
    fn generate_terrain(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // 随机生成山、水、林等地形
        let terrain_types = [
            (TerrainType::Mountain, "太行山"),
            (TerrainType::Mountain, "昆仑山"),
            (TerrainType::Water, "玄水湖"),
            (TerrainType::Forest, "青松林"),
            (TerrainType::Forest, "竹海"),
            (TerrainType::Plain, "沃野平原"),
        ];

        for (terrain_type, name) in &terrain_types {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            self.elements.push(PositionedElement {
                element: MapElement::Terrain(Terrain {
                    terrain_type: *terrain_type,
                    name: name.to_string(),
                }),
                position: Position { x, y },
            });
        }
    }

    ===== 旧的程序化生成地图逻辑结束 ===== */

    /// 获取所有可用任务
    pub fn get_available_tasks(&mut self) -> Vec<Task> {
        let mut tasks = Vec::new();
        let mut task_id = 0;

        for positioned in &mut self.elements {
            let mut element_tasks = positioned.element.generate_tasks(task_id);

            // 为所有从此位置生成的任务设置核心位置（交互位置）
            for task in &mut element_tasks {
                task.position = Some(positioned.core_position);
            }

            // 如果是妖魔任务，需要记录任务ID
            if let MapElement::Monster(monster) = &mut positioned.element {
                if !element_tasks.is_empty() && !monster.has_task() {
                    // 记录这个妖魔现在有任务了
                    if let Some(first_task) = element_tasks.first() {
                        monster.set_task(first_task.id);
                    }
                }
            }

            task_id += element_tasks.len();
            tasks.extend(element_tasks);
        }

        // 添加守卫任务（妖魔入侵时）
        let defense_tasks = self.generate_defense_tasks(task_id);
        tasks.extend(defense_tasks);

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

        // 妖魔行动：移动或修行
        self.monster_actions();

        // 怪物可能成长
        for positioned in &mut self.elements {
            if let MapElement::Monster(monster) = &mut positioned.element {
                if rng.gen_bool(monster.growth_rate) {
                    monster.grow();
                }
            }
        }

        /* ===== 旧的随机妖魔生成逻辑（已注释，保留用于未来参考） =====
        // 可能出现新的怪物（从配置的随机名称池中选择）
        let spawn_chance = self.config.monsters.spawn_rules.spawn_chance;
        if rng.gen_bool(spawn_chance) {
            let (min_level, max_level) = self.config.monsters.spawn_rules.level_range;
            let random_names = &self.config.monsters.spawn_rules.random_names;

            if !random_names.is_empty() {
                let name = random_names[rng.gen_range(0..random_names.len())].clone();
                let level = rng.gen_range(min_level..=max_level);

                // 随机位置
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);

                // 使用默认的任务模板（从第一个妖魔模板复制，如果有的话）
                let task_templates = if let Some(first_template) = self.config.monsters.monster_templates.first() {
                    first_template.task_templates.clone()
                } else {
                    vec![]
                };

                self.elements.push(PositionedElement {
                    element: MapElement::Monster(Monster::new(name, level, task_templates)),
                    position: Position { x, y },
                });
            }
        }
        ===== 旧的随机妖魔生成逻辑结束 ===== */
    }

    /// 妖魔行动（移动或修行）
    fn monster_actions(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut move_actions = Vec::new(); // (monster_index, new_position)

        for (i, positioned) in self.elements.iter_mut().enumerate() {
            if let MapElement::Monster(monster) = &mut positioned.element {
                // 如果妖魔正在被战斗或有正在执行的守卫任务，则不能行动
                if monster.is_being_fought || monster.has_active_defense_task {
                    continue;
                }

                // 50% 概率选择移动，50% 概率选择修行
                if rng.gen_bool(0.5) {
                    // 移动：随机选择相邻位置
                    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                    let (dx, dy) = directions[rng.gen_range(0..directions.len())];
                    let new_x = (positioned.core_position.x + dx).max(0).min(self.width - 1);
                    let new_y = (positioned.core_position.y + dy).max(0).min(self.height - 1);
                    let new_position = Position { x: new_x, y: new_y };

                    // 只在位置可通行时才移动
                    move_actions.push((i, new_position));
                } else {
                    // 修行：提升等级
                    if rng.gen_bool(0.3) {  // 30% 概率成功修行
                        monster.grow();
                    }
                }
            }
        }

        // 执行移动
        for (monster_index, new_position) in move_actions {
            // 检查新位置是否可通行（需要临时移除自己来检查）
            let is_passable = {
                let current_monster_pos = if let Some(elem) = self.elements.get(monster_index) {
                    elem.core_position
                } else {
                    continue;
                };

                // 临时检查：排除自己的位置
                self.elements.iter().enumerate().all(|(i, positioned)| {
                    if i == monster_index {
                        return true; // 跳过自己
                    }
                    if positioned.occupies_position(&new_position) {
                        // 地形不可通行
                        if matches!(positioned.element, MapElement::Terrain(_)) {
                            return false;
                        }
                        // 建筑不可通行（但可以入侵）
                        if !matches!(positioned.element, MapElement::Monster(_)) {
                            // 这里允许移动到建筑位置（用于入侵）
                            return true;
                        }
                    }
                    true
                })
            };

            if is_passable {
                if let Some(positioned) = self.elements.get_mut(monster_index) {
                    positioned.core_position = new_position;
                    positioned.positions = vec![new_position]; // 妖魔只占一格

                    // 检查是否移动到了可入侵的地点
                    if matches!(positioned.element, MapElement::Monster(_)) {
                        self.check_monster_invasion(monster_index, new_position);
                    }
                }
            }
        }
    }

    /// 检查妖魔是否入侵了某个地点
    fn check_monster_invasion(&mut self, monster_index: usize, monster_pos: Position) {
        // 先查找同核心位置的可入侵元素
        let invaded_location_id = self.elements.iter().enumerate()
            .find(|(i, positioned)| {
                *i != monster_index &&
                positioned.core_position.x == monster_pos.x &&
                positioned.core_position.y == monster_pos.y &&
                positioned.element.can_be_invaded()
            })
            .map(|(_, positioned)| positioned.element.get_location_id());

        // 更新妖魔的入侵状态（无论是否找到入侵地点，都要更新）
        if let Some(monster_elem) = self.elements.get_mut(monster_index) {
            if let MapElement::Monster(monster) = &mut monster_elem.element {
                // 如果找到入侵地点就设置，否则清除
                monster.invaded_location_id = invaded_location_id;
            }
        }
    }

    /// 生成被入侵地点的守卫任务
    pub fn generate_defense_tasks(&self, task_id_start: usize) -> Vec<Task> {
        let mut tasks = Vec::new();
        let mut task_id = task_id_start;

        for positioned in &self.elements {
            if let MapElement::Monster(monster) = &positioned.element {
                // 如果妖魔入侵了某个地点
                if let Some(ref invaded_location_id) = monster.invaded_location_id {
                    // 找到被入侵的地点
                    if let Some(invaded_elem) = self.elements.iter().find(|p| {
                        p.element.get_location_id() == *invaded_location_id
                    }) {
                        if let Some(task_name) = invaded_elem.element.get_defense_task_name() {
                            // 创建守卫任务
                            let mut task = Task::new(
                                task_id,
                                task_name,
                                crate::task::TaskType::Combat(crate::task::CombatTask {
                                    enemy_name: format!("{}#{}", monster.name, monster.id),
                                    enemy_level: monster.level,
                                    difficulty: monster.level,
                                }),
                                monster.level * 10,  // 进度奖励
                                monster.level * 20,  // 资源奖励
                            );

                            // 设置任务位置为被入侵地点的核心位置
                            task.position = Some(invaded_elem.core_position);

                            tasks.push(task);
                            task_id += 1;
                        }
                    }
                }
            }
        }

        tasks
    }

    /// 清除妖魔的任务关联（当任务完成或失效时调用）
    pub fn clear_monster_task(&mut self, task_id: usize) {
        for positioned in &mut self.elements {
            if let MapElement::Monster(monster) = &mut positioned.element {
                if monster.current_task_id == Some(task_id) {
                    monster.clear_task();
                }
            }
        }
    }

    /// 锁定妖魔的移动（当守卫任务被分配时调用）
    /// 通过敌人名称（如"妖兽#10"）识别妖魔ID
    pub fn lock_monster_for_defense_task(&mut self, enemy_name: &str) {
        // 从敌人名称中提取妖魔ID（格式：{怪物名}#{ID}）
        if let Some(id_str) = enemy_name.split('#').nth(1) {
            if let Ok(monster_id) = id_str.parse::<usize>() {
                for positioned in &mut self.elements {
                    if let MapElement::Monster(monster) = &mut positioned.element {
                        if monster.id == monster_id {
                            monster.has_active_defense_task = true;
                            return;
                        }
                    }
                }
            }
        }
    }

    /// 解锁妖魔的移动（当守卫任务完成或失效时调用）
    /// 通过敌人名称（如"妖兽#10"）识别妖魔ID
    pub fn unlock_monster_for_defense_task(&mut self, enemy_name: &str) {
        // 从敌人名称中提取妖魔ID（格式：{怪物名}#{ID}）
        if let Some(id_str) = enemy_name.split('#').nth(1) {
            if let Ok(monster_id) = id_str.parse::<usize>() {
                for positioned in &mut self.elements {
                    if let MapElement::Monster(monster) = &mut positioned.element {
                        if monster.id == monster_id {
                            monster.has_active_defense_task = false;
                            return;
                        }
                    }
                }
            }
        }
    }

    /// 检查所有守卫任务，移除那些妖魔已离开的任务
    /// 返回需要移除的任务ID列表
    pub fn check_defense_tasks_validity(&self, current_tasks: &[crate::task::Task]) -> Vec<usize> {
        let mut invalid_task_ids = Vec::new();

        for task in current_tasks {
            // 只检查守卫任务（Combat类型且名称包含"守卫"）
            if let crate::task::TaskType::Combat(combat_task) = &task.task_type {
                // 守卫任务的敌人名称格式为 "{怪物名}#{ID}"
                if task.name.contains("守卫") && combat_task.enemy_name.contains('#') {
                    // 提取妖魔ID（从 "妖兽#10" 这样的格式中提取）
                    if let Some(id_str) = combat_task.enemy_name.split('#').nth(1) {
                        if let Ok(monster_id) = id_str.parse::<usize>() {
                            // 查找妖魔
                            if let Some(positioned) = self.elements.iter().find(|p| {
                                if let MapElement::Monster(m) = &p.element {
                                    m.id == monster_id
                                } else {
                                    false
                                }
                            }) {
                                if let MapElement::Monster(monster) = &positioned.element {
                                    // 检查妖魔是否还在入侵状态
                                    if monster.invaded_location_id.is_none() {
                                        // 妖魔已经离开，任务应该失效
                                        invalid_task_ids.push(task.id);
                                    }
                                }
                            } else {
                                // 妖魔不存在了（被消灭），任务应该失效
                                invalid_task_ids.push(task.id);
                            }
                        }
                    }
                }
            }
        }

        invalid_task_ids
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

/// 辅助函数：解析任务类型
fn parse_task_type(template: &TaskTemplateConfig) -> Option<TaskType> {
    match template.task_type.as_str() {
        "Gathering" => Some(TaskType::Gathering(GatheringTask {
            resource_type: template.resource_type.as_ref()?.clone(),
            difficulty: template.difficulty.unwrap_or(1),
        })),
        "Combat" => Some(TaskType::Combat(CombatTask {
            enemy_name: "未知敌人".to_string(), // 需要在调用处替换
            enemy_level: template.difficulty.unwrap_or(1),
            difficulty: template.difficulty.unwrap_or(1),
        })),
        "Exploration" => Some(TaskType::Exploration(ExplorationTask {
            location: "未知地点".to_string(), // 需要在调用处替换
            danger_level: template.danger_level.unwrap_or(10),
        })),
        "Auxiliary" => {
            let skill_required = template.skill_required.as_ref()
                .and_then(|s| parse_talent_type_option(s));
            Some(TaskType::Auxiliary(AuxiliaryTask {
                task_name: template.name_template.clone(),
                skill_required,
            }))
        },
        _ => None,
    }
}

/// 辅助函数：解析资质类型
fn parse_talent_type(s: &str) -> TalentType {
    match s {
        "Fire" => TalentType::Fire,
        "Water" => TalentType::Water,
        "Wood" => TalentType::Wood,
        "Metal" => TalentType::Metal,
        "Earth" => TalentType::Earth,
        "Sword" => TalentType::Sword,
        "Alchemy" => TalentType::Alchemy,
        "Formation" => TalentType::Formation,
        "Medical" => TalentType::Medical,
        _ => TalentType::Fire, // 默认
    }
}

fn parse_talent_type_option(s: &str) -> Option<TalentType> {
    match s {
        "Fire" => Some(TalentType::Fire),
        "Water" => Some(TalentType::Water),
        "Wood" => Some(TalentType::Wood),
        "Metal" => Some(TalentType::Metal),
        "Earth" => Some(TalentType::Earth),
        "Sword" => Some(TalentType::Sword),
        "Alchemy" => Some(TalentType::Alchemy),
        "Formation" => Some(TalentType::Formation),
        "Medical" => Some(TalentType::Medical),
        _ => None,
    }
}
