use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::modifier::ConditionalModifier;

/// 建筑定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: u32,  // 基础建造成本
    pub parent_id: Option<String>,  // 父建筑ID，None表示根节点
    pub conditional_modifiers: Vec<ConditionalModifier>,  // 建筑提供的条件modifier
    pub is_built: bool,  // 是否已建造
}

impl Building {
    /// 创建一个新的建筑
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        base_cost: u32,
        parent_id: Option<String>,
        conditional_modifiers: Vec<ConditionalModifier>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            base_cost,
            parent_id,
            conditional_modifiers,
            is_built: false,
        }
    }

    /// 创建根建筑
    pub fn new_root(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        base_cost: u32,
        conditional_modifiers: Vec<ConditionalModifier>,
    ) -> Self {
        Self::new(id, name, description, base_cost, None, conditional_modifiers)
    }

    /// 创建子建筑
    pub fn new_child(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        base_cost: u32,
        parent_id: impl Into<String>,
        conditional_modifiers: Vec<ConditionalModifier>,
    ) -> Self {
        Self::new(id, name, description, base_cost, Some(parent_id.into()), conditional_modifiers)
    }
}

/// 建筑树 - 管理宗门的所有建筑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingTree {
    pub buildings: HashMap<String, Building>,
    pub root_id: String,
    pub buildings_built_count: u32,  // 已建造的建筑数量（用于计算成本倍增）
}

impl BuildingTree {
    /// 创建一个新的建筑树
    pub fn new(root_building: Building) -> Self {
        let root_id = root_building.id.clone();
        let mut buildings = HashMap::new();
        buildings.insert(root_id.clone(), root_building);

        Self {
            buildings,
            root_id,
            buildings_built_count: 0,
        }
    }

    /// 添加建筑到树中
    pub fn add_building(&mut self, building: Building) -> Result<(), String> {
        // 检查ID是否已存在
        if self.buildings.contains_key(&building.id) {
            return Err(format!("建筑ID '{}'已存在", building.id));
        }

        // 如果有父节点，检查父节点是否存在
        if let Some(ref parent_id) = building.parent_id {
            if !self.buildings.contains_key(parent_id) {
                return Err(format!("父建筑'{}'不存在", parent_id));
            }
        } else if building.id != self.root_id {
            // 非根节点必须有父节点
            return Err("非根建筑必须指定父建筑".to_string());
        }

        self.buildings.insert(building.id.clone(), building);
        Ok(())
    }

    /// 检查建筑是否可以建造
    pub fn can_build(&self, building_id: &str) -> Result<(), String> {
        // 1. 检查建筑是否存在
        let building = self.buildings.get(building_id)
            .ok_or_else(|| format!("建筑'{}'不存在", building_id))?;

        // 2. 检查是否已建造
        if building.is_built {
            return Err(format!("建筑'{}'已经建造", building.name));
        }

        // 3. 检查父节点是否已建造
        if let Some(ref parent_id) = building.parent_id {
            let parent = self.buildings.get(parent_id)
                .ok_or_else(|| format!("父建筑'{}'不存在", parent_id))?;

            if !parent.is_built {
                return Err(format!("需要先建造'{}'", parent.name));
            }
        }

        Ok(())
    }

    /// 计算建造指定建筑的实际成本
    /// 成本公式：base_cost * 2^buildings_built_count
    pub fn calculate_build_cost(&self, building_id: &str) -> Result<u32, String> {
        let building = self.buildings.get(building_id)
            .ok_or_else(|| format!("建筑'{}'不存在", building_id))?;

        // 计算倍增系数：2^buildings_built_count
        let multiplier = 2_u32.pow(self.buildings_built_count);

        // 防止溢出
        building.base_cost.checked_mul(multiplier)
            .ok_or_else(|| "建造成本溢出".to_string())
    }

    /// 建造建筑
    pub fn build(&mut self, building_id: &str) -> Result<Vec<ConditionalModifier>, String> {
        // 1. 检查是否可以建造
        self.can_build(building_id)?;

        // 2. 标记为已建造
        let building = self.buildings.get_mut(building_id)
            .ok_or_else(|| format!("建筑'{}'不存在", building_id))?;

        building.is_built = true;

        // 3. 增加已建造计数
        self.buildings_built_count += 1;

        // 4. 返回建筑提供的modifiers
        Ok(building.conditional_modifiers.clone())
    }

    /// 获取所有已建造建筑提供的modifiers
    pub fn get_all_modifiers(&self) -> Vec<ConditionalModifier> {
        self.buildings
            .values()
            .filter(|b| b.is_built)
            .flat_map(|b| b.conditional_modifiers.clone())
            .collect()
    }

    /// 获取所有可建造的建筑（父节点已建造但自己未建造）
    pub fn get_buildable_buildings(&self) -> Vec<&Building> {
        self.buildings
            .values()
            .filter(|b| !b.is_built && self.can_build(&b.id).is_ok())
            .collect()
    }

    /// 获取建筑的子建筑列表
    pub fn get_children(&self, parent_id: &str) -> Vec<&Building> {
        self.buildings
            .values()
            .filter(|b| {
                b.parent_id.as_ref().map(|p| p == parent_id).unwrap_or(false)
            })
            .collect()
    }

    /// 获取建筑树的深度（从根节点到最深叶子的距离）
    pub fn get_depth(&self) -> usize {
        fn calculate_depth(tree: &BuildingTree, node_id: &str) -> usize {
            let children = tree.get_children(node_id);
            if children.is_empty() {
                0
            } else {
                1 + children.iter()
                    .map(|child| calculate_depth(tree, &child.id))
                    .max()
                    .unwrap_or(0)
            }
        }

        calculate_depth(self, &self.root_id)
    }

    /// 获取已建造的建筑数量
    pub fn get_built_count(&self) -> usize {
        self.buildings.values().filter(|b| b.is_built).count()
    }

    /// 获取总建筑数量
    pub fn get_total_count(&self) -> usize {
        self.buildings.len()
    }

    /// 重置建筑树（取消所有建造，用于测试）
    #[cfg(test)]
    pub fn reset(&mut self) {
        for building in self.buildings.values_mut() {
            building.is_built = false;
        }
        self.buildings_built_count = 0;
    }
}

/// 建筑树构建器 - 方便创建预定义的建筑树
pub struct BuildingTreeBuilder {
    buildings: Vec<Building>,
}

impl BuildingTreeBuilder {
    pub fn new() -> Self {
        Self {
            buildings: Vec::new(),
        }
    }

    /// 添加根建筑
    pub fn root(mut self, building: Building) -> Self {
        assert!(building.parent_id.is_none(), "根建筑不应该有父节点");
        self.buildings.push(building);
        self
    }

    /// 添加子建筑
    pub fn child(mut self, building: Building) -> Self {
        assert!(building.parent_id.is_some(), "子建筑必须有父节点");
        self.buildings.push(building);
        self
    }

    /// 构建建筑树
    pub fn build(self) -> Result<BuildingTree, String> {
        if self.buildings.is_empty() {
            return Err("建筑树至少需要一个根节点".to_string());
        }

        // 找到根节点
        let root = self.buildings.iter()
            .find(|b| b.parent_id.is_none())
            .ok_or("未找到根节点")?;

        let mut tree = BuildingTree::new(root.clone());

        // 添加所有其他建筑
        for building in self.buildings.into_iter() {
            if building.parent_id.is_some() {
                tree.add_building(building)?;
            }
        }

        Ok(tree)
    }
}

impl Default for BuildingTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建默认的修仙宗门建筑树
pub fn create_default_sect_building_tree() -> BuildingTree {
    use crate::modifier::{Modifier, ModifierTarget, ModifierApplication, ModifierSource, ModifierCondition};
    use crate::disciple::DiscipleType;
    use crate::cultivation::CultivationLevel;

    // 根节点：宗门大殿
    let root = Building::new_root(
        "main_hall",
        "宗门大殿",
        "宗门的核心建筑，象征着宗门的威严",
        100,
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "大殿威严",
                    ModifierTarget::Income,
                    ModifierApplication::Multiplicative(0.1),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    let mut tree = BuildingTree::new(root);

    // 第一层建筑

    // 1. 藏书楼 - 提升修炼速度
    let library = Building::new_child(
        "library",
        "藏书楼",
        "收藏功法典籍，提升弟子修炼速度",
        150,
        "main_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "功法加成",
                    ModifierTarget::CultivationSpeed,
                    ModifierApplication::Multiplicative(0.15),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 2. 炼丹房 - 减少精力消耗
    let alchemy_room = Building::new_child(
        "alchemy_room",
        "炼丹房",
        "炼制丹药，帮助弟子恢复精力",
        150,
        "main_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "丹药滋养",
                    ModifierTarget::EnergyConsumption,
                    ModifierApplication::Multiplicative(-0.2),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 3. 演武场 - 减少体魄消耗
    let training_ground = Building::new_child(
        "training_ground",
        "演武场",
        "弟子切磋武艺之处，强健体魄",
        150,
        "main_hall",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "体魄强化",
                    ModifierTarget::ConstitutionConsumption,
                    ModifierApplication::Multiplicative(-0.2),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 第二层建筑

    // 4. 天机阁 - 提升内门弟子任务奖励
    let heavenly_pavilion = Building::new_child(
        "heavenly_pavilion",
        "天机阁",
        "推演天机，内门弟子任务奖励提升",
        200,
        "library",
        vec![
            ConditionalModifier::new(
                ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
                Modifier::new(
                    "天机加持",
                    ModifierTarget::TaskReward,
                    ModifierApplication::Multiplicative(0.25),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 5. 灵药园 - 提升任务收益
    let spirit_garden = Building::new_child(
        "spirit_garden",
        "灵药园",
        "种植灵药，增加宗门收入",
        200,
        "alchemy_room",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "灵药收益",
                    ModifierTarget::Income,
                    ModifierApplication::Multiplicative(0.2),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 6. 炼器坊 - 提升战斗任务适配度
    let weapon_forge = Building::new_child(
        "weapon_forge",
        "炼器坊",
        "炼制法宝，提升战斗能力",
        200,
        "training_ground",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "法宝加成",
                    ModifierTarget::TaskSuitability,
                    ModifierApplication::Additive(5.0),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 第三层建筑

    // 7. 传承殿 - 大幅提升亲传弟子修炼速度
    let heritage_hall = Building::new_child(
        "heritage_hall",
        "传承殿",
        "存放宗门至高传承，亲传弟子修炼速度大幅提升",
        300,
        "heavenly_pavilion",
        vec![
            ConditionalModifier::new(
                ModifierCondition::DiscipleTypeEquals(DiscipleType::Personal),
                Modifier::new(
                    "传承之力",
                    ModifierTarget::CultivationSpeed,
                    ModifierApplication::Multiplicative(0.5),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 8. 聚灵阵 - 筑基期以上弟子修炼速度提升
    let spirit_array = Building::new_child(
        "spirit_array",
        "聚灵阵",
        "汇聚天地灵气，筑基期以上弟子修炼速度提升",
        300,
        "spirit_garden",
        vec![
            ConditionalModifier::new(
                ModifierCondition::CultivationLevelGreaterThan(CultivationLevel::QiRefining),
                Modifier::new(
                    "灵气滋养",
                    ModifierTarget::CultivationSpeed,
                    ModifierApplication::Multiplicative(0.3),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 9. 护宗大阵 - 提升所有弟子道心
    let protection_array = Building::new_child(
        "protection_array",
        "护宗大阵",
        "守护宗门，提升弟子道心",
        300,
        "weapon_forge",
        vec![
            ConditionalModifier::new(
                ModifierCondition::Always,
                Modifier::new(
                    "大阵庇护",
                    ModifierTarget::DaoHeart,
                    ModifierApplication::Additive(10.0),
                    ModifierSource::System,
                ),
            ),
        ],
    );

    // 添加所有建筑到树中
    tree.add_building(library).unwrap();
    tree.add_building(alchemy_room).unwrap();
    tree.add_building(training_ground).unwrap();
    tree.add_building(heavenly_pavilion).unwrap();
    tree.add_building(spirit_garden).unwrap();
    tree.add_building(weapon_forge).unwrap();
    tree.add_building(heritage_hall).unwrap();
    tree.add_building(spirit_array).unwrap();
    tree.add_building(protection_array).unwrap();

    tree
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifier::{Modifier, ModifierTarget, ModifierApplication, ModifierSource, ModifierCondition};
    use crate::disciple::DiscipleType;

    #[test]
    fn test_building_tree_creation() {
        let root = Building::new_root(
            "root",
            "宗门大殿",
            "宗门的核心建筑",
            100,
            vec![],
        );

        let tree = BuildingTree::new(root);
        assert_eq!(tree.buildings.len(), 1);
        assert_eq!(tree.buildings_built_count, 0);
    }

    #[test]
    fn test_add_child_building() {
        let root = Building::new_root(
            "root",
            "宗门大殿",
            "宗门的核心建筑",
            100,
            vec![],
        );

        let mut tree = BuildingTree::new(root);

        let child = Building::new_child(
            "library",
            "藏书楼",
            "存放功法的地方",
            200,
            "root",
            vec![],
        );

        assert!(tree.add_building(child).is_ok());
        assert_eq!(tree.buildings.len(), 2);
    }

    #[test]
    fn test_build_cost_calculation() {
        let root = Building::new_root("root", "大殿", "核心", 100, vec![]);
        let mut tree = BuildingTree::new(root);

        // 第一个建筑：100 * 2^0 = 100
        assert_eq!(tree.calculate_build_cost("root").unwrap(), 100);

        // 建造后
        tree.build("root").unwrap();

        // 添加第二个建筑
        let child = Building::new_child("lib", "藏书楼", "书", 200, "root", vec![]);
        tree.add_building(child).unwrap();

        // 第二个建筑：200 * 2^1 = 400
        assert_eq!(tree.calculate_build_cost("lib").unwrap(), 400);
    }

    #[test]
    fn test_build_dependency() {
        let root = Building::new_root("root", "大殿", "核心", 100, vec![]);
        let mut tree = BuildingTree::new(root);

        let child = Building::new_child("lib", "藏书楼", "书", 200, "root", vec![]);
        tree.add_building(child).unwrap();

        // 父节点未建造，不能建造子节点
        assert!(tree.can_build("lib").is_err());

        // 建造父节点
        tree.build("root").unwrap();

        // 现在可以建造子节点
        assert!(tree.can_build("lib").is_ok());
    }

    #[test]
    fn test_building_with_modifiers() {
        let modifier = ConditionalModifier::new(
            ModifierCondition::DiscipleTypeEquals(DiscipleType::Inner),
            Modifier::new(
                "内门加成",
                ModifierTarget::TaskReward,
                ModifierApplication::Multiplicative(0.2),
                ModifierSource::System,
            ),
        );

        let root = Building::new_root(
            "root",
            "大殿",
            "核心",
            100,
            vec![modifier],
        );

        let mut tree = BuildingTree::new(root);
        tree.build("root").unwrap();

        let modifiers = tree.get_all_modifiers();
        assert_eq!(modifiers.len(), 1);
    }
}
