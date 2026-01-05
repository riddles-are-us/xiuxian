use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 地图元素配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapElementsConfig {
    pub villages: Vec<VillageTemplate>,
    pub factions: Vec<FactionTemplate>,
    pub dangerous_locations: Vec<DangerousLocationTemplate>,
    pub secret_realms: Vec<SecretRealmTemplate>,
}

/// 村庄模板
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VillageTemplate {
    pub name: String,
    pub population: u32,
    pub prosperity: u32,
    pub position: PositionConfig,
    #[serde(default)]
    pub size: Option<SizeConfig>,  // 建筑尺寸，None 表示 1x1
    pub task_templates: Vec<TaskTemplateConfig>,
}

/// 势力模板
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionTemplate {
    pub name: String,
    pub power_level: u32,
    pub relationship: i32,
    pub position: PositionConfig,
    #[serde(default)]
    pub size: Option<SizeConfig>,  // 建筑尺寸，None 表示 1x1
    pub friendly_task_templates: Vec<TaskTemplateConfig>,
    pub hostile_task_templates: Vec<TaskTemplateConfig>,
}

/// 险地模板
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DangerousLocationTemplate {
    pub name: String,
    pub danger_level: u32,
    pub position: PositionConfig,
    #[serde(default)]
    pub size: Option<SizeConfig>,  // 建筑尺寸，None 表示 1x1
    pub task_templates: Vec<TaskTemplateConfig>,
}

/// 秘境模板
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretRealmTemplate {
    pub name: String,
    pub realm_type: String, // "Fire", "Water", etc.
    pub difficulty: u32,
    pub position: PositionConfig,
    #[serde(default)]
    pub size: Option<SizeConfig>,  // 建筑尺寸，None 表示 1x1
    pub task_templates: Vec<TaskTemplateConfig>,
}

/// 位置配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PositionConfig {
    pub x: i32,
    pub y: i32,
}

/// 尺寸配置（用于大型建筑，如2x2）
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SizeConfig {
    pub width: u32,  // 宽度（格子数）
    pub height: u32, // 高度（格子数）
}

/// 任务模板配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskTemplateConfig {
    pub name_template: String, // 支持 {name} 等占位符
    pub task_type: String,      // "Gathering", "Combat", "Exploration", "Auxiliary"
    pub progress_reward: u32,
    pub resource_reward: u32,
    pub reputation_reward: i32,
    pub dao_heart_impact: i32,

    // 任务类型特定参数
    #[serde(default)]
    pub resource_type: Option<String>, // for Gathering
    #[serde(default)]
    pub difficulty: Option<u32>,
    #[serde(default)]
    pub danger_level: Option<u32>, // for Exploration
    #[serde(default)]
    pub skill_required: Option<String>, // for Auxiliary
}

/// 妖魔配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonstersConfig {
    pub monster_templates: Vec<MonsterTemplate>,
    pub spawn_rules: SpawnRules,
}

/// 妖魔模板
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonsterTemplate {
    pub name: String,
    pub level: u32,
    pub is_demon: bool,
    pub growth_rate: f64, // 成长速率（每回合的成长概率）
    pub position: Option<PositionConfig>, // 初始位置（如果有）
    pub task_templates: Vec<TaskTemplateConfig>,
}

/// 妖魔生成规则
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpawnRules {
    pub spawn_chance: f64,       // 每回合生成新妖魔的概率
    pub level_range: (u32, u32), // 生成妖魔的等级范围
    pub random_names: Vec<String>, // 随机妖魔名称池
}

impl MapElementsConfig {
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 创建默认配置
    pub fn default_config() -> Self {
        Self {
            villages: vec![
                VillageTemplate {
                    name: "清风镇".to_string(),
                    population: 1000,
                    prosperity: 50,
                    position: PositionConfig { x: 5, y: 5 },
                    size: None,
                    task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "在{name}采集灵药".to_string(),
                            task_type: "Gathering".to_string(),
                            progress_reward: 5,
                            resource_reward: 10,
                            reputation_reward: 5,
                            dao_heart_impact: 0,
                            resource_type: Some("灵药".to_string()),
                            difficulty: Some(1),
                            danger_level: None,
                            skill_required: None,
                        },
                        TaskTemplateConfig {
                            name_template: "在{name}行医".to_string(),
                            task_type: "Auxiliary".to_string(),
                            progress_reward: 5,
                            resource_reward: 8,
                            reputation_reward: 15,
                            dao_heart_impact: 5,
                            resource_type: None,
                            difficulty: None,
                            danger_level: None,
                            skill_required: Some("Medical".to_string()),
                        },
                    ],
                },
                VillageTemplate {
                    name: "灵泉村".to_string(),
                    population: 500,
                    prosperity: 30,
                    position: PositionConfig { x: 15, y: 8 },
                    size: None,
                    task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "在{name}采集灵泉".to_string(),
                            task_type: "Gathering".to_string(),
                            progress_reward: 8,
                            resource_reward: 15,
                            reputation_reward: 10,
                            dao_heart_impact: 0,
                            resource_type: Some("灵泉".to_string()),
                            difficulty: Some(2),
                            danger_level: None,
                            skill_required: None,
                        },
                    ],
                },
            ],
            factions: vec![
                FactionTemplate {
                    name: "青云派".to_string(),
                    power_level: 3,
                    relationship: 20,
                    position: PositionConfig { x: 10, y: 10 },
                    size: Some(SizeConfig { width: 2, height: 2 }),  // 大型势力建筑
                    friendly_task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "与{name}交流".to_string(),
                            task_type: "Auxiliary".to_string(),
                            progress_reward: 8,
                            resource_reward: 15,
                            reputation_reward: 20,
                            dao_heart_impact: 0,
                            resource_type: None,
                            difficulty: None,
                            danger_level: None,
                            skill_required: None,
                        },
                    ],
                    hostile_task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "镇压{name}".to_string(),
                            task_type: "Combat".to_string(),
                            progress_reward: 15,
                            resource_reward: 30,
                            reputation_reward: 30,
                            dao_heart_impact: -5,
                            resource_type: None,
                            difficulty: Some(3),
                            danger_level: None,
                            skill_required: None,
                        },
                    ],
                },
            ],
            dangerous_locations: vec![
                DangerousLocationTemplate {
                    name: "迷雾森林".to_string(),
                    danger_level: 20,
                    position: PositionConfig { x: 3, y: 15 },
                    size: None,
                    task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "游历{name}".to_string(),
                            task_type: "Exploration".to_string(),
                            progress_reward: 10,
                            resource_reward: 20,
                            reputation_reward: 0,
                            dao_heart_impact: 0,
                            resource_type: None,
                            difficulty: None,
                            danger_level: Some(20),
                            skill_required: None,
                        },
                    ],
                },
            ],
            secret_realms: vec![
                SecretRealmTemplate {
                    name: "火焰洞窟".to_string(),
                    realm_type: "Fire".to_string(),
                    difficulty: 30,
                    position: PositionConfig { x: 17, y: 3 },
                    size: Some(SizeConfig { width: 2, height: 2 }),  // 大型秘境
                    task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "探索秘境：{name}".to_string(),
                            task_type: "Exploration".to_string(),
                            progress_reward: 20,
                            resource_reward: 50,
                            reputation_reward: 0,
                            dao_heart_impact: 0,
                            resource_type: None,
                            difficulty: None,
                            danger_level: Some(30),
                            skill_required: None,
                        },
                    ],
                },
            ],
        }
    }
}

impl MonstersConfig {
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 创建默认配置
    pub fn default_config() -> Self {
        Self {
            monster_templates: vec![
                MonsterTemplate {
                    name: "噬魂虎".to_string(),
                    level: 2,
                    is_demon: false,
                    growth_rate: 0.1,
                    position: Some(PositionConfig { x: 8, y: 12 }),
                    task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "讨伐{name}".to_string(),
                            task_type: "Combat".to_string(),
                            progress_reward: 15,
                            resource_reward: 40,
                            reputation_reward: 25,
                            dao_heart_impact: 3,
                            resource_type: None,
                            difficulty: Some(2),
                            danger_level: None,
                            skill_required: None,
                        },
                    ],
                },
                MonsterTemplate {
                    name: "血影狼".to_string(),
                    level: 3,
                    is_demon: false,
                    growth_rate: 0.15,
                    position: Some(PositionConfig { x: 12, y: 16 }),
                    task_templates: vec![
                        TaskTemplateConfig {
                            name_template: "讨伐{name}".to_string(),
                            task_type: "Combat".to_string(),
                            progress_reward: 20,
                            resource_reward: 50,
                            reputation_reward: 30,
                            dao_heart_impact: 3,
                            resource_type: None,
                            difficulty: Some(3),
                            danger_level: None,
                            skill_required: None,
                        },
                    ],
                },
            ],
            spawn_rules: SpawnRules {
                spawn_chance: 0.2,
                level_range: (1, 5),
                random_names: vec![
                    "妖兽".to_string(),
                    "魔狼".to_string(),
                    "邪灵".to_string(),
                    "凶兽".to_string(),
                    "妖虎".to_string(),
                    "魔猿".to_string(),
                ],
            },
        }
    }
}

/// 配置管理器
#[derive(Debug)]
pub struct ConfigManager {
    pub map_elements: MapElementsConfig,
    pub monsters: MonstersConfig,
}

impl ConfigManager {
    /// 加载所有配置
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let map_elements = match MapElementsConfig::load_from_file("config/map_elements.json") {
            Ok(config) => config,
            Err(_) => {
                println!("未找到地图元素配置文件，使用默认配置");
                let config = MapElementsConfig::default_config();
                // 尝试保存默认配置
                if let Err(e) = std::fs::create_dir_all("config") {
                    println!("创建config目录失败: {}", e);
                } else if let Err(e) = config.save_to_file("config/map_elements.json") {
                    println!("保存默认地图元素配置失败: {}", e);
                }
                config
            }
        };

        let monsters = match MonstersConfig::load_from_file("config/monsters.json") {
            Ok(config) => config,
            Err(_) => {
                println!("未找到妖魔配置文件，使用默认配置");
                let config = MonstersConfig::default_config();
                // 尝试保存默认配置
                if let Err(e) = std::fs::create_dir_all("config") {
                    println!("创建config目录失败: {}", e);
                } else if let Err(e) = config.save_to_file("config/monsters.json") {
                    println!("保存默认妖魔配置失败: {}", e);
                }
                config
            }
        };

        Ok(Self {
            map_elements,
            monsters,
        })
    }

    /// 创建默认配置
    pub fn create_default() -> Self {
        Self {
            map_elements: MapElementsConfig::default_config(),
            monsters: MonstersConfig::default_config(),
        }
    }
}
