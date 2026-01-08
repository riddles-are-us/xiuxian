use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::modifier::{Modifier, ModifierTarget, ModifierApplication, ModifierSource, ModifierCondition, ConditionalModifier};
use crate::cultivation::CultivationLevel;
use crate::disciple::DiscipleType;

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

// ============ Modifier 配置结构体 ============

/// Modifier 目标配置
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ModifierTargetConfig {
    DaoHeart,
    Energy,
    Constitution,
    TalentBonus { talent_type: String },
    TribulationSuccessRate,
    TaskReward,
    TaskSuitability,
    TaskDifficulty,
    Income,
    EnergyConsumption,
    ConstitutionConsumption,
    CultivationSpeed,
}

impl ModifierTargetConfig {
    pub fn to_modifier_target(&self) -> ModifierTarget {
        match self {
            Self::DaoHeart => ModifierTarget::DaoHeart,
            Self::Energy => ModifierTarget::Energy,
            Self::Constitution => ModifierTarget::Constitution,
            Self::TalentBonus { talent_type } => ModifierTarget::TalentBonus(talent_type.clone()),
            Self::TribulationSuccessRate => ModifierTarget::TribulationSuccessRate,
            Self::TaskReward => ModifierTarget::TaskReward,
            Self::TaskSuitability => ModifierTarget::TaskSuitability,
            Self::TaskDifficulty => ModifierTarget::TaskDifficulty,
            Self::Income => ModifierTarget::Income,
            Self::EnergyConsumption => ModifierTarget::EnergyConsumption,
            Self::ConstitutionConsumption => ModifierTarget::ConstitutionConsumption,
            Self::CultivationSpeed => ModifierTarget::CultivationSpeed,
        }
    }
}

/// Modifier 应用方式配置
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ModifierApplicationConfig {
    Additive { value: f32 },
    Multiplicative { value: f32 },
    Override { value: f32 },
}

impl ModifierApplicationConfig {
    pub fn to_modifier_application(&self) -> ModifierApplication {
        match self {
            Self::Additive { value } => ModifierApplication::Additive(*value),
            Self::Multiplicative { value } => ModifierApplication::Multiplicative(*value),
            Self::Override { value } => ModifierApplication::Override(*value),
        }
    }
}

/// Modifier 条件配置
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ModifierConditionConfig {
    Always,
    // 修为条件
    CultivationLevelEquals { level: String },
    CultivationLevelGreaterThan { level: String },
    CultivationLevelLessThan { level: String },
    CultivationLevelGreaterOrEqual { level: String },
    CultivationLevelLessOrEqual { level: String },
    // 弟子类型
    DiscipleTypeEquals { disciple_type: String },
    // 属性条件
    DaoHeartGreaterThan { value: u32 },
    DaoHeartLessThan { value: u32 },
    EnergyGreaterThan { value: u32 },
    EnergyLessThan { value: u32 },
    ConstitutionGreaterThan { value: u32 },
    ConstitutionLessThan { value: u32 },
    // 年龄条件
    AgeGreaterThan { value: u32 },
    AgeLessThan { value: u32 },
    AgeEquals { value: u32 },
    // 组合条件
    And { conditions: Vec<ModifierConditionConfig> },
    Or { conditions: Vec<ModifierConditionConfig> },
    Not { condition: Box<ModifierConditionConfig> },
}

impl ModifierConditionConfig {
    fn parse_cultivation_level(level: &str) -> CultivationLevel {
        match level {
            "QiRefining" | "练气" => CultivationLevel::QiRefining,
            "Foundation" | "筑基" => CultivationLevel::Foundation,
            "GoldenCore" | "结丹" => CultivationLevel::GoldenCore,
            "NascentSoul" | "凝婴" => CultivationLevel::NascentSoul,
            "SpiritSevering" | "化神" => CultivationLevel::SpiritSevering,
            "VoidRefinement" | "练虚" => CultivationLevel::VoidRefinement,
            "Ascension" | "飞升" => CultivationLevel::Ascension,
            _ => CultivationLevel::QiRefining, // 默认
        }
    }

    fn parse_disciple_type(dtype: &str) -> DiscipleType {
        match dtype {
            "Outer" | "外门" => DiscipleType::Outer,
            "Inner" | "内门" => DiscipleType::Inner,
            "Personal" | "Core" | "亲传" => DiscipleType::Personal,
            _ => DiscipleType::Outer, // 默认
        }
    }

    pub fn to_modifier_condition(&self) -> ModifierCondition {
        match self {
            Self::Always => ModifierCondition::Always,
            // 修为条件
            Self::CultivationLevelEquals { level } => {
                ModifierCondition::CultivationLevelEquals(Self::parse_cultivation_level(level))
            }
            Self::CultivationLevelGreaterThan { level } => {
                ModifierCondition::CultivationLevelGreaterThan(Self::parse_cultivation_level(level))
            }
            Self::CultivationLevelLessThan { level } => {
                ModifierCondition::CultivationLevelLessThan(Self::parse_cultivation_level(level))
            }
            Self::CultivationLevelGreaterOrEqual { level } => {
                ModifierCondition::CultivationLevelGreaterOrEqual(Self::parse_cultivation_level(level))
            }
            Self::CultivationLevelLessOrEqual { level } => {
                ModifierCondition::CultivationLevelLessOrEqual(Self::parse_cultivation_level(level))
            }
            // 弟子类型
            Self::DiscipleTypeEquals { disciple_type } => {
                ModifierCondition::DiscipleTypeEquals(Self::parse_disciple_type(disciple_type))
            }
            // 属性条件
            Self::DaoHeartGreaterThan { value } => ModifierCondition::DaoHeartGreaterThan(*value),
            Self::DaoHeartLessThan { value } => ModifierCondition::DaoHeartLessThan(*value),
            Self::EnergyGreaterThan { value } => ModifierCondition::EnergyGreaterThan(*value),
            Self::EnergyLessThan { value } => ModifierCondition::EnergyLessThan(*value),
            Self::ConstitutionGreaterThan { value } => ModifierCondition::ConstitutionGreaterThan(*value),
            Self::ConstitutionLessThan { value } => ModifierCondition::ConstitutionLessThan(*value),
            // 年龄条件
            Self::AgeGreaterThan { value } => ModifierCondition::AgeGreaterThan(*value),
            Self::AgeLessThan { value } => ModifierCondition::AgeLessThan(*value),
            Self::AgeEquals { value } => ModifierCondition::AgeEquals(*value),
            // 组合条件
            Self::And { conditions } => {
                ModifierCondition::And(conditions.iter().map(|c| c.to_modifier_condition()).collect())
            }
            Self::Or { conditions } => {
                ModifierCondition::Or(conditions.iter().map(|c| c.to_modifier_condition()).collect())
            }
            Self::Not { condition } => {
                ModifierCondition::Not(Box::new(condition.to_modifier_condition()))
            }
        }
    }
}

/// 单个 Modifier 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModifierConfig {
    pub name: String,
    pub target: ModifierTargetConfig,
    pub application: ModifierApplicationConfig,
    #[serde(default = "default_source")]
    pub source: String,
    pub condition: ModifierConditionConfig,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub duration: Option<u32>,
}

fn default_source() -> String {
    "System".to_string()
}

impl ModifierConfig {
    fn parse_source(source: &str) -> ModifierSource {
        match source {
            "Talent" => ModifierSource::Talent,
            "Equipment" => ModifierSource::Equipment,
            "Buff" => ModifierSource::Buff,
            "Debuff" => ModifierSource::Debuff,
            "Pill" => ModifierSource::Pill,
            "Heritage" => ModifierSource::Heritage,
            "Environment" => ModifierSource::Environment,
            "Relationship" => ModifierSource::Relationship,
            _ => ModifierSource::System,
        }
    }

    pub fn to_conditional_modifier(&self) -> ConditionalModifier {
        let mut modifier = Modifier::new(
            self.name.clone(),
            self.target.to_modifier_target(),
            self.application.to_modifier_application(),
            Self::parse_source(&self.source),
        );
        modifier.priority = self.priority;
        modifier.duration = self.duration;

        ConditionalModifier::new(
            self.condition.to_modifier_condition(),
            modifier,
        )
    }
}

/// 建筑配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildingConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: u32,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub modifiers: Vec<ModifierConfig>,
}

/// 建筑配置文件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildingsConfig {
    pub buildings: Vec<BuildingConfig>,
}

impl BuildingsConfig {
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

    /// 加载建筑配置（自动创建默认配置）
    pub fn load() -> Self {
        match Self::load_from_file("config/buildings.json") {
            Ok(config) => config,
            Err(_) => {
                println!("未找到建筑配置文件，使用默认配置");
                let config = Self::default_config();
                // 尝试保存默认配置
                if let Err(e) = std::fs::create_dir_all("config") {
                    println!("创建config目录失败: {}", e);
                } else if let Err(e) = config.save_to_file("config/buildings.json") {
                    println!("保存默认建筑配置失败: {}", e);
                }
                config
            }
        }
    }

    /// 创建默认建筑配置
    pub fn default_config() -> Self {
        Self {
            buildings: vec![
                // 根节点：宗门大殿
                BuildingConfig {
                    id: "main_hall".to_string(),
                    name: "宗门大殿".to_string(),
                    description: "宗门的核心建筑，象征着宗门的威严".to_string(),
                    base_cost: 100,
                    parent_id: None,
                    modifiers: vec![
                        ModifierConfig {
                            name: "大殿威严".to_string(),
                            target: ModifierTargetConfig::Income,
                            application: ModifierApplicationConfig::Multiplicative { value: 0.1 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第一层：藏书楼
                BuildingConfig {
                    id: "library".to_string(),
                    name: "藏书楼".to_string(),
                    description: "收藏功法典籍，提升弟子修炼速度".to_string(),
                    base_cost: 150,
                    parent_id: Some("main_hall".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "功法加成".to_string(),
                            target: ModifierTargetConfig::CultivationSpeed,
                            application: ModifierApplicationConfig::Multiplicative { value: 0.15 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第一层：炼丹房
                BuildingConfig {
                    id: "alchemy_room".to_string(),
                    name: "炼丹房".to_string(),
                    description: "炼制丹药，帮助弟子恢复精力".to_string(),
                    base_cost: 150,
                    parent_id: Some("main_hall".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "丹药滋养".to_string(),
                            target: ModifierTargetConfig::EnergyConsumption,
                            application: ModifierApplicationConfig::Multiplicative { value: -0.2 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第一层：演武场
                BuildingConfig {
                    id: "training_ground".to_string(),
                    name: "演武场".to_string(),
                    description: "弟子切磋武艺之处，强健体魄".to_string(),
                    base_cost: 150,
                    parent_id: Some("main_hall".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "体魄强化".to_string(),
                            target: ModifierTargetConfig::ConstitutionConsumption,
                            application: ModifierApplicationConfig::Multiplicative { value: -0.2 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第二层：天机阁
                BuildingConfig {
                    id: "heavenly_pavilion".to_string(),
                    name: "天机阁".to_string(),
                    description: "推演天机，内门弟子任务奖励提升".to_string(),
                    base_cost: 200,
                    parent_id: Some("library".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "天机加持".to_string(),
                            target: ModifierTargetConfig::TaskReward,
                            application: ModifierApplicationConfig::Multiplicative { value: 0.25 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::DiscipleTypeEquals { disciple_type: "Inner".to_string() },
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第二层：灵药园
                BuildingConfig {
                    id: "spirit_garden".to_string(),
                    name: "灵药园".to_string(),
                    description: "种植灵药，增加宗门收入".to_string(),
                    base_cost: 200,
                    parent_id: Some("alchemy_room".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "灵药收益".to_string(),
                            target: ModifierTargetConfig::Income,
                            application: ModifierApplicationConfig::Multiplicative { value: 0.2 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第二层：炼器坊
                BuildingConfig {
                    id: "weapon_forge".to_string(),
                    name: "炼器坊".to_string(),
                    description: "炼制法宝，提升战斗能力".to_string(),
                    base_cost: 200,
                    parent_id: Some("training_ground".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "法宝加成".to_string(),
                            target: ModifierTargetConfig::TaskSuitability,
                            application: ModifierApplicationConfig::Additive { value: 5.0 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第三层：传承殿
                BuildingConfig {
                    id: "heritage_hall".to_string(),
                    name: "传承殿".to_string(),
                    description: "存放宗门至高传承，亲传弟子修炼速度大幅提升".to_string(),
                    base_cost: 300,
                    parent_id: Some("heavenly_pavilion".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "传承之力".to_string(),
                            target: ModifierTargetConfig::CultivationSpeed,
                            application: ModifierApplicationConfig::Multiplicative { value: 0.5 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::DiscipleTypeEquals { disciple_type: "Personal".to_string() },
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第三层：聚灵阵
                BuildingConfig {
                    id: "spirit_array".to_string(),
                    name: "聚灵阵".to_string(),
                    description: "汇聚天地灵气，筑基期以上弟子修炼速度提升".to_string(),
                    base_cost: 300,
                    parent_id: Some("spirit_garden".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "灵气滋养".to_string(),
                            target: ModifierTargetConfig::CultivationSpeed,
                            application: ModifierApplicationConfig::Multiplicative { value: 0.3 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::CultivationLevelGreaterThan { level: "QiRefining".to_string() },
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
                // 第三层：护宗大阵
                BuildingConfig {
                    id: "protection_array".to_string(),
                    name: "护宗大阵".to_string(),
                    description: "守护宗门，提升弟子道心".to_string(),
                    base_cost: 300,
                    parent_id: Some("weapon_forge".to_string()),
                    modifiers: vec![
                        ModifierConfig {
                            name: "大阵庇护".to_string(),
                            target: ModifierTargetConfig::DaoHeart,
                            application: ModifierApplicationConfig::Additive { value: 10.0 },
                            source: "System".to_string(),
                            condition: ModifierConditionConfig::Always,
                            priority: 0,
                            duration: None,
                        },
                    ],
                },
            ],
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
