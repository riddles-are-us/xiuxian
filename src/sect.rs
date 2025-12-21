use crate::disciple::{Disciple, DiscipleType, Heritage};
use crate::cultivation::CultivationLevel;
use crate::pill::PillInventory;
use crate::modifier::ConditionalModifier;
use crate::building::BuildingTree;
use crate::relationship::{Relationship, RelationDimension, RelationLevel, RelationGrowth};
use crate::task::TaskType;

/// 宗门
#[derive(Debug)]
pub struct Sect {
    pub name: String,
    pub disciples: Vec<Disciple>,
    pub resources: u32,
    pub reputation: i32,
    pub is_immortal_sect: bool,
    pub heritages: Vec<Heritage>, // 传承库
    pub year: u32, // 当前年份
    pub pill_inventory: PillInventory, // 丹药库存
    pub sect_modifiers: Vec<ConditionalModifier>, // 宗门级别的条件modifier
    pub building_tree: Option<BuildingTree>, // 建筑树（可选）
}

impl Sect {
    pub fn new(name: String) -> Self {
        Self {
            name,
            disciples: Vec::new(),
            resources: 1000, // 初始资源
            reputation: 0,
            is_immortal_sect: false,
            heritages: Vec::new(),
            year: 0,
            pill_inventory: PillInventory::new(),
            sect_modifiers: Vec::new(),
            building_tree: None,
        }
    }

    /// 初始化建筑树
    pub fn init_building_tree(&mut self, building_tree: BuildingTree) {
        self.building_tree = Some(building_tree);
    }

    /// 添加宗门级别的条件modifier
    pub fn add_sect_modifier(&mut self, conditional_modifier: ConditionalModifier) {
        self.sect_modifiers.push(conditional_modifier);
    }

    /// 移除宗门级别的条件modifier
    pub fn remove_sect_modifier(&mut self, index: usize) -> Option<ConditionalModifier> {
        if index < self.sect_modifiers.len() {
            Some(self.sect_modifiers.remove(index))
        } else {
            None
        }
    }

    /// 清除所有宗门modifier
    pub fn clear_sect_modifiers(&mut self) {
        self.sect_modifiers.clear();
    }

    /// 获取对指定弟子生效的所有宗门modifier（包括建筑提供的）
    /// 注意：由于生命周期限制，这个方法返回拥有所有权的Modifier向量
    pub fn get_applicable_modifiers_owned(&self, disciple: &Disciple) -> Vec<crate::modifier::Modifier> {
        let mut modifiers: Vec<crate::modifier::Modifier> = Vec::new();

        // 添加宗门直接设置的modifiers
        for cm in &self.sect_modifiers {
            if let Some(m) = cm.get_modifier_if_applies(disciple) {
                modifiers.push(m.clone());
            }
        }

        // 添加建筑树提供的modifiers
        if let Some(ref tree) = self.building_tree {
            for cm in tree.get_all_modifiers() {
                if cm.applies_to(disciple) {
                    modifiers.push(cm.modifier.clone());
                }
            }
        }

        modifiers
    }

    /// 获取对指定弟子生效的所有宗门modifier（返回引用，仅包括直接设置的modifiers）
    pub fn get_applicable_modifiers(&self, disciple: &Disciple) -> Vec<&crate::modifier::Modifier> {
        self.sect_modifiers
            .iter()
            .filter_map(|cm| cm.get_modifier_if_applies(disciple))
            .collect()
    }

    // === 建筑系统方法 ===

    /// 建造建筑
    pub fn build_building(&mut self, building_id: &str) -> Result<String, String> {
        // 1. 检查是否有建筑树
        let tree = self.building_tree.as_mut()
            .ok_or("宗门尚未初始化建筑树")?;

        // 2. 检查是否可以建造
        tree.can_build(building_id)?;

        // 3. 计算建造成本
        let cost = tree.calculate_build_cost(building_id)?;

        // 4. 检查资源是否足够
        if self.resources < cost {
            return Err(format!("资源不足，需要{}，当前只有{}", cost, self.resources));
        }

        // 5. 扣除资源
        self.resources -= cost;

        // 6. 执行建造
        let modifiers = tree.build(building_id)?;

        // 7. 将建筑提供的modifiers添加到宗门modifiers（可选，因为get_applicable_modifiers已经会获取它们）
        // 这里选择不添加，让modifiers由建筑树统一管理

        // 8. 获取建筑名称用于返回消息
        let building_name = tree.buildings.get(building_id)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| building_id.to_string());

        Ok(format!("成功建造'{}'，花费{}资源，获得{}个效果",
                   building_name, cost, modifiers.len()))
    }

    /// 获取可建造的建筑列表（包含成本信息）
    pub fn get_buildable_buildings_with_cost(&self) -> Vec<(String, String, u32)> {
        if let Some(ref tree) = self.building_tree {
            tree.get_buildable_buildings()
                .iter()
                .filter_map(|b| {
                    tree.calculate_build_cost(&b.id)
                        .ok()
                        .map(|cost| (b.id.clone(), b.name.clone(), cost))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取建筑树信息摘要
    pub fn get_building_tree_summary(&self) -> Option<String> {
        self.building_tree.as_ref().map(|tree| {
            format!(
                "建筑树：{}/{}已建造，深度{}，下一个建筑成本倍增{}x",
                tree.get_built_count(),
                tree.get_total_count(),
                tree.get_depth(),
                2_u32.pow(tree.buildings_built_count)
            )
        })
    }

    /// 添加弟子
    pub fn recruit_disciple(&mut self, disciple: Disciple) {
        self.disciples.push(disciple);
    }

    /// 获取存活弟子
    pub fn alive_disciples(&self) -> Vec<&Disciple> {
        self.disciples.iter().filter(|d| d.is_alive()).collect()
    }

    /// 获取可变存活弟子
    pub fn alive_disciples_mut(&mut self) -> Vec<&mut Disciple> {
        self.disciples.iter_mut().filter(|d| d.is_alive()).collect()
    }

    /// 检查是否成为仙门
    pub fn check_immortal_sect(&mut self) -> bool {
        if self.disciples.iter().any(|d| d.is_immortal()) {
            self.is_immortal_sect = true;
            true
        } else {
            false
        }
    }

    /// 检查是否灭门
    pub fn is_destroyed(&self) -> bool {
        self.alive_disciples().is_empty()
    }

    /// 处理弟子死亡
    pub fn handle_disciple_death(&mut self, disciple_id: usize) {
        if let Some(disciple) = self.disciples.iter().find(|d| d.id == disciple_id) {
            // 生成传承
            if let Some(heritage) = disciple.generate_heritage() {
                println!("{}留下了传承：{}", disciple.name, heritage.name);
                self.heritages.push(heritage);
            }
        }
    }

    /// 增加资源
    pub fn add_resources(&mut self, amount: u32) {
        self.resources += amount;
    }

    /// 消耗资源
    pub fn consume_resources(&mut self, amount: u32) -> bool {
        if self.resources >= amount {
            self.resources -= amount;
            true
        } else {
            false
        }
    }

    /// 增加声望
    pub fn add_reputation(&mut self, amount: i32) {
        self.reputation += amount;
    }

    /// 年度更新
    pub fn yearly_update(&mut self) {
        self.year += 1;

        // 收集死亡弟子ID
        let mut dead_disciples = Vec::new();

        // 所有弟子增加年龄
        for disciple in &mut self.disciples {
            if disciple.is_alive() {
                disciple.age_one_year();

                // 检查是否寿元耗尽
                if !disciple.is_alive() {
                    println!(
                        "{}寿元耗尽，享年{}岁（{}期）",
                        disciple.name, disciple.age, disciple.cultivation.current_level
                    );
                    dead_disciples.push(disciple.id);
                }
            }
        }

        // 处理死亡弟子
        for id in dead_disciples {
            self.handle_disciple_death(id);
        }
    }

    /// 获取宗门统计信息
    pub fn get_statistics(&self) -> SectStatistics {
        let alive = self.alive_disciples();
        let outer = alive
            .iter()
            .filter(|d| d.disciple_type == DiscipleType::Outer)
            .count();
        let inner = alive
            .iter()
            .filter(|d| d.disciple_type == DiscipleType::Inner)
            .count();
        let personal = alive
            .iter()
            .filter(|d| d.disciple_type == DiscipleType::Personal)
            .count();

        let mut cultivation_distribution = vec![0; 7];
        for disciple in &alive {
            let level_index = disciple.cultivation.current_level as usize;
            cultivation_distribution[level_index] += 1;
        }

        SectStatistics {
            total_disciples: alive.len(),
            outer_disciples: outer,
            inner_disciples: inner,
            personal_disciples: personal,
            resources: self.resources,
            reputation: self.reputation,
            year: self.year,
            cultivation_distribution,
        }
    }

    // === 关系系统方法 ===

    /// 设置师徒关系
    pub fn set_mentorship(&mut self, master_id: usize, disciple_id: usize) -> Result<(), String> {
        // 验证两个弟子都存在
        if !self.disciples.iter().any(|d| d.id == master_id && d.is_alive()) {
            return Err("师父不存在或已死亡".to_string());
        }
        if !self.disciples.iter().any(|d| d.id == disciple_id && d.is_alive()) {
            return Err("徒弟不存在或已死亡".to_string());
        }
        if master_id == disciple_id {
            return Err("不能自己拜自己为师".to_string());
        }

        let year = self.year;

        // 为徒弟添加师父关系
        if let Some(disciple) = self.disciples.iter_mut().find(|d| d.id == disciple_id) {
            // 检查是否已有师父
            if disciple.get_master_id().is_some() {
                return Err("已有师父，不能再拜师".to_string());
            }
            let rel = Relationship::new_as_disciple_of(master_id, year);
            disciple.relationships.push(rel);
        }

        // 为师父添加徒弟关系
        if let Some(master) = self.disciples.iter_mut().find(|d| d.id == master_id) {
            let rel = Relationship::new_as_master_of(disciple_id, year);
            master.relationships.push(rel);
        }

        Ok(())
    }

    /// 设置道侣关系（需要双方情感 >= 80）
    pub fn set_dao_companion(&mut self, id1: usize, id2: usize) -> Result<(), String> {
        // 验证两个弟子都存在
        if !self.disciples.iter().any(|d| d.id == id1 && d.is_alive()) {
            return Err("第一位弟子不存在或已死亡".to_string());
        }
        if !self.disciples.iter().any(|d| d.id == id2 && d.is_alive()) {
            return Err("第二位弟子不存在或已死亡".to_string());
        }
        if id1 == id2 {
            return Err("不能与自己结为道侣".to_string());
        }

        // 检查双方情感分数
        let d1_romance = self.disciples.iter()
            .find(|d| d.id == id1)
            .and_then(|d| d.get_relationship(id2))
            .map(|r| r.scores.romance)
            .unwrap_or(0);

        let d2_romance = self.disciples.iter()
            .find(|d| d.id == id2)
            .and_then(|d| d.get_relationship(id1))
            .map(|r| r.scores.romance)
            .unwrap_or(0);

        if d1_romance < 80 || d2_romance < 80 {
            return Err(format!(
                "双方情感分数不足（需要>=80），当前: {} -> {} = {}, {} -> {} = {}",
                id1, id2, d1_romance, id2, id1, d2_romance
            ));
        }

        // 检查双方是否已有道侣
        if self.disciples.iter().find(|d| d.id == id1).map(|d| d.has_dao_companion()).unwrap_or(false) {
            return Err("第一位弟子已有道侣".to_string());
        }
        if self.disciples.iter().find(|d| d.id == id2).map(|d| d.has_dao_companion()).unwrap_or(false) {
            return Err("第二位弟子已有道侣".to_string());
        }

        let year = self.year;

        // 设置双方的道侣标记
        if let Some(d1) = self.disciples.iter_mut().find(|d| d.id == id1) {
            let rel = d1.get_or_create_relationship(id2, year);
            rel.is_dao_companion = true;
        }
        if let Some(d2) = self.disciples.iter_mut().find(|d| d.id == id2) {
            let rel = d2.get_or_create_relationship(id1, year);
            rel.is_dao_companion = true;
        }

        Ok(())
    }

    /// 更新关系分数
    pub fn update_relationship_score(
        &mut self,
        from_id: usize,
        to_id: usize,
        dimension: RelationDimension,
        delta: i32,
    ) -> Result<Option<RelationLevel>, String> {
        if !self.disciples.iter().any(|d| d.id == from_id && d.is_alive()) {
            return Err("来源弟子不存在或已死亡".to_string());
        }
        if !self.disciples.iter().any(|d| d.id == to_id && d.is_alive()) {
            return Err("目标弟子不存在或已死亡".to_string());
        }

        let year = self.year;

        if let Some(disciple) = self.disciples.iter_mut().find(|d| d.id == from_id) {
            let rel = disciple.get_or_create_relationship(to_id, year);
            let (_, level_up) = rel.scores.add(dimension, delta);
            Ok(level_up)
        } else {
            Err("弟子不存在".to_string())
        }
    }

    /// 一起完成任务时更新关系
    pub fn update_relationship_from_task(
        &mut self,
        disciple_ids: &[usize],
        task_type: &TaskType,
    ) -> Vec<(usize, usize, RelationDimension, RelationLevel)> {
        let year = self.year;
        let growth = RelationGrowth::from_task_type(task_type);
        let mut level_ups = Vec::new();

        // 为所有参与任务的弟子之间更新关系
        for i in 0..disciple_ids.len() {
            for j in 0..disciple_ids.len() {
                if i == j {
                    continue;
                }

                let from_id = disciple_ids[i];
                let to_id = disciple_ids[j];

                if let Some(disciple) = self.disciples.iter_mut().find(|d| d.id == from_id) {
                    let rel = disciple.get_or_create_relationship(to_id, year);
                    let ups = growth.apply_to(&mut rel.scores);
                    for (dim, level) in ups {
                        level_ups.push((from_id, to_id, dim, level));
                    }
                }
            }
        }

        level_ups
    }

    /// 获取两个弟子之间的关系描述
    pub fn get_relationship_description(&self, from_id: usize, to_id: usize) -> Option<String> {
        let from = self.disciples.iter().find(|d| d.id == from_id)?;
        let to = self.disciples.iter().find(|d| d.id == to_id)?;
        let rel = from.get_relationship(to_id)?;

        Some(format!(
            "{} 与 {} 的关系: {} (情感:{}, 师徒:{}, 战友:{}, 认知:{}, 机缘:{})",
            from.name,
            to.name,
            rel.get_primary_relation(),
            rel.scores.romance,
            rel.scores.mentorship,
            rel.scores.comrade,
            rel.scores.understanding,
            rel.scores.fateful_bond
        ))
    }

    /// 获取弟子的所有关系
    pub fn get_disciple_relationships(&self, disciple_id: usize) -> Vec<(usize, String, &Relationship)> {
        let disciple = match self.disciples.iter().find(|d| d.id == disciple_id) {
            Some(d) => d,
            None => return Vec::new(),
        };

        disciple.relationships.iter()
            .filter_map(|rel| {
                let target = self.disciples.iter().find(|d| d.id == rel.target_id)?;
                Some((rel.target_id, target.name.clone(), rel))
            })
            .collect()
    }
}

/// 宗门统计信息
#[derive(Debug)]
pub struct SectStatistics {
    pub total_disciples: usize,
    pub outer_disciples: usize,
    pub inner_disciples: usize,
    pub personal_disciples: usize,
    pub resources: u32,
    pub reputation: i32,
    pub year: u32,
    pub cultivation_distribution: Vec<usize>, // 各修为等级的弟子数量
}

impl std::fmt::Display for SectStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== 宗门统计 ===")?;
        writeln!(f, "年份: {}", self.year)?;
        writeln!(f, "弟子总数: {}", self.total_disciples)?;
        writeln!(f, "  外门: {}", self.outer_disciples)?;
        writeln!(f, "  内门: {}", self.inner_disciples)?;
        writeln!(f, "  亲传: {}", self.personal_disciples)?;
        writeln!(f, "资源: {}", self.resources)?;
        writeln!(f, "声望: {}", self.reputation)?;
        writeln!(f, "\n修为分布:")?;

        let levels = [
            CultivationLevel::QiRefining,
            CultivationLevel::Foundation,
            CultivationLevel::GoldenCore,
            CultivationLevel::NascentSoul,
            CultivationLevel::SpiritSevering,
            CultivationLevel::VoidRefinement,
            CultivationLevel::Ascension,
        ];

        for (i, level) in levels.iter().enumerate() {
            if self.cultivation_distribution[i] > 0 {
                writeln!(f, "  {}: {}", level, self.cultivation_distribution[i])?;
            }
        }

        Ok(())
    }
}
