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

/// 从配置文件创建宗门建筑树
pub fn create_sect_building_tree() -> BuildingTree {
    use crate::config::BuildingsConfig;

    let config = BuildingsConfig::load();

    // 找到根节点（parent_id 为 None 的建筑）
    let root_config = config.buildings.iter()
        .find(|b| b.parent_id.is_none())
        .expect("建筑配置必须包含一个根节点");

    let modifiers: Vec<ConditionalModifier> = root_config.modifiers.iter()
        .map(|mc| mc.to_conditional_modifier())
        .collect();

    let root = Building::new_root(
        &root_config.id,
        &root_config.name,
        &root_config.description,
        root_config.base_cost,
        modifiers,
    );

    let mut tree = BuildingTree::new(root);

    // 添加所有子建筑
    for bc in &config.buildings {
        if bc.parent_id.is_some() {
            let modifiers: Vec<ConditionalModifier> = bc.modifiers.iter()
                .map(|mc| mc.to_conditional_modifier())
                .collect();

            let building = Building::new_child(
                &bc.id,
                &bc.name,
                &bc.description,
                bc.base_cost,
                bc.parent_id.as_ref().unwrap(),
                modifiers,
            );

            if let Err(e) = tree.add_building(building) {
                eprintln!("添加建筑 {} 失败: {}", bc.name, e);
            }
        }
    }

    tree
}

/// 创建默认的修仙宗门建筑树（兼容旧代码）
#[deprecated(note = "请使用 create_sect_building_tree() 代替")]
pub fn create_default_sect_building_tree() -> BuildingTree {
    create_sect_building_tree()
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
