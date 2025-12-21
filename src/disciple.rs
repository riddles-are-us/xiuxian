use crate::cultivation::{CultivationLevel, SubLevel, CultivationPath};
use crate::task::{Task, TaskType};
use crate::modifier::{ModifierStack, ModifierTarget, Modifier, ModifierSource};
use crate::map::Position;
use crate::relationship::Relationship;

/// 弟子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiscipleType {
    Outer,      // 外门
    Inner,      // 内门
    Personal,   // 亲传
}

/// 资质类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    pub dao_heart: u32,  // 道心 0-100 (native value)
    pub energy: u32,     // 精力 0-100 (native value)
    pub constitution: u32, // 体魄 0-100 (native value)
    pub heritage: Option<Heritage>,
    pub relationships: Vec<Relationship>, // 与其他弟子的关系
    pub children: Vec<usize>, // 子女ID列表
    pub modifiers: ModifierStack, // Modifier系统
    pub position: Position, // 弟子在地图上的位置
    pub moves_remaining: u32, // 本回合剩余移动距离
}

impl Disciple {
    pub fn new(id: usize, name: String, disciple_type: DiscipleType, talents: Vec<Talent>) -> Self {
        let lifespan = CultivationLevel::QiRefining.base_lifespan();
        let movement_range = CultivationLevel::QiRefining.movement_range();

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
            relationships: Vec::new(),
            children: Vec::new(),
            modifiers: ModifierStack::new(),
            position: Position { x: 10, y: 10 }, // 初始位置在宗门
            moves_remaining: movement_range, // 初始化为移动范围
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

    /// 移动到指定位置
    pub fn move_to(&mut self, new_position: Position) {
        self.position = new_position;
    }

    /// 检查是否在指定位置
    pub fn is_at_position(&self, pos: &Position) -> bool {
        self.position.x == pos.x && self.position.y == pos.y
    }

    // === Modifier系统辅助方法 ===

    /// 获取有效道心值（应用modifier后，包含宗门modifiers）
    pub fn get_effective_dao_heart(&self) -> f32 {
        self.get_effective_dao_heart_with_sect_modifiers(&[])
    }

    /// 获取有效道心值（应用modifier后，包含宗门modifiers）
    pub fn get_effective_dao_heart_with_sect_modifiers(&self, sect_modifiers: &[&crate::modifier::Modifier]) -> f32 {
        self.modifiers.calculate_effective_with_extras(&ModifierTarget::DaoHeart, self.dao_heart as f32, sect_modifiers)
    }

    /// 获取有效精力值（应用modifier后）
    pub fn get_effective_energy(&self) -> f32 {
        self.get_effective_energy_with_sect_modifiers(&[])
    }

    /// 获取有效精力值（应用modifier后，包含宗门modifiers）
    pub fn get_effective_energy_with_sect_modifiers(&self, sect_modifiers: &[&crate::modifier::Modifier]) -> f32 {
        self.modifiers.calculate_effective_with_extras(&ModifierTarget::Energy, self.energy as f32, sect_modifiers)
    }

    /// 获取有效体魄值（应用modifier后）
    pub fn get_effective_constitution(&self) -> f32 {
        self.get_effective_constitution_with_sect_modifiers(&[])
    }

    /// 获取有效体魄值（应用modifier后，包含宗门modifiers）
    pub fn get_effective_constitution_with_sect_modifiers(&self, sect_modifiers: &[&crate::modifier::Modifier]) -> f32 {
        self.modifiers.calculate_effective_with_extras(&ModifierTarget::Constitution, self.constitution as f32, sect_modifiers)
    }

    /// 每回合更新modifier（减少持续时间，移除过期的modifier）
    pub fn tick_modifiers(&mut self) -> usize {
        self.modifiers.tick()
    }

    /// 添加一个modifier
    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.add_modifier(modifier);
    }

    /// 移除指定来源的所有modifier
    pub fn remove_modifiers_by_source(&mut self, source: &ModifierSource) {
        self.modifiers.remove_modifiers_by_source(source);
    }

    /// 获取资质加成（应用modifier后的有效值）
    pub fn get_talent_bonus(&self, talent_type: &TalentType) -> f32 {
        self.get_talent_bonus_with_sect_modifiers(talent_type, &[])
    }

    /// 获取资质加成（应用modifier后的有效值，包含宗门modifiers）
    pub fn get_talent_bonus_with_sect_modifiers(&self, talent_type: &TalentType, sect_modifiers: &[&crate::modifier::Modifier]) -> f32 {
        // 1. 计算native值（原始天赋加成）
        let native_bonus = self.talents
            .iter()
            .find(|t| &t.talent_type == talent_type)
            .map(|t| t.level as f32 * 0.1)
            .unwrap_or(0.0);

        // 2. 应用modifier获取effective值（包含宗门modifiers）
        let talent_type_str = format!("{:?}", talent_type);
        let target = ModifierTarget::TalentBonus(talent_type_str);
        self.modifiers.calculate_effective_with_extras(&target, native_bonus, sect_modifiers)
    }

    /// 计算渡劫成功率（应用modifier后的有效值）
    pub fn tribulation_success_rate(&self) -> f32 {
        self.tribulation_success_rate_with_sect_modifiers(&[])
    }

    /// 计算渡劫成功率（应用modifier后的有效值，包含宗门modifiers）
    pub fn tribulation_success_rate_with_sect_modifiers(&self, sect_modifiers: &[&crate::modifier::Modifier]) -> f32 {
        // 1. 使用有效道心值（应用modifier后，包含宗门modifiers）
        let effective_dao_heart = self.get_effective_dao_heart_with_sect_modifiers(sect_modifiers);

        let base_rate = 0.3; // 基础成功率30%
        let dao_heart_bonus = effective_dao_heart * 0.005; // 道心加成
        let heritage_bonus = self.heritage
            .as_ref()
            .map(|h| h.tribulation_bonus)
            .unwrap_or(0.0);

        // 2. 计算native成功率
        let native_rate = (base_rate + dao_heart_bonus + heritage_bonus).min(0.95);

        // 3. 应用TribulationSuccessRate modifier（包含宗门modifiers）
        let effective_rate = self.modifiers.calculate_effective_with_extras(
            &ModifierTarget::TribulationSuccessRate,
            native_rate,
            sect_modifiers
        );

        // 4. 确保在合理范围内 (0.0 - 0.95)
        effective_rate.max(0.0).min(0.95)
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

    /// 完成任务（应用modifier后的有效奖励）
    pub fn complete_task(&mut self, task: &Task) -> u32 {
        // 1. 天赋加成（已经应用了modifier）
        let talent_bonus = match &task.task_type {
            TaskType::Gathering(_) => self.get_talent_bonus(&TalentType::Wood),
            TaskType::Combat(_) => self.get_talent_bonus(&TalentType::Sword),
            TaskType::Exploration(_) => 0.0,
            TaskType::Auxiliary(_) => self.get_talent_bonus(&TalentType::Formation),
            TaskType::Investment(_) => 0.0,
        };

        // 2. 动态修为奖励计算
        let base_progress = task.progress_reward as f32;

        // 3. 难度系数：任务越难，奖励越高
        //    难度值通常在 0-100，我们将其映射到 0.5-2.0 的乘数
        let task_difficulty = task.get_difficulty() as f32;
        let difficulty_multiplier = 0.5 + (task_difficulty / 50.0).min(1.5);

        // 4. 等级惩罚：弟子修为越高，获得的奖励越少（边际收益递减）
        //    练气(0)->100%, 筑基(1)->83%, 结丹(2)->71%, 凝婴(3)->63%, 化神(4)->56%, 练虚(5)->50%, 飞升(6)->45%
        let disciple_level = self.cultivation.current_level.to_numeric() as f32;
        let level_penalty = 1.0 / (1.0 + disciple_level / 6.0);

        // 5. 天赋乘数
        let talent_multiplier = 1.0 + talent_bonus;

        // 6. 计算native奖励
        let native_reward = base_progress * difficulty_multiplier * level_penalty * talent_multiplier;

        // 7. 应用TaskReward modifier获取effective奖励
        let effective_reward = self.modifiers.calculate_effective(
            &ModifierTarget::TaskReward,
            native_reward
        );

        // 8. 转换为整数，确保至少给予1点修为
        let actual_progress = (effective_reward as u32).max(1);

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

    /// 消耗精力（应用modifier后的有效消耗）
    pub fn consume_energy(&mut self, amount: u32) {
        // 1. 应用EnergyConsumption modifier
        let effective_consumption = self.modifiers.calculate_effective(
            &ModifierTarget::EnergyConsumption,
            amount as f32
        ) as u32;

        // 2. 执行消耗
        if self.energy >= effective_consumption {
            self.energy -= effective_consumption;
        } else {
            self.energy = 0;
        }

        // 如果精力降到0，减少1年寿命
        if self.energy == 0 && self.lifespan > 0 {
            self.lifespan = self.lifespan.saturating_sub(1);
            println!("   ⚠️ {}精力耗尽，寿命减少1年（剩余{}年）", self.name, self.lifespan - self.age);
        }
    }

    /// 消耗体魄（应用modifier后的有效消耗）
    pub fn consume_constitution(&mut self, amount: u32) {
        // 1. 应用ConstitutionConsumption modifier
        let effective_consumption = self.modifiers.calculate_effective(
            &ModifierTarget::ConstitutionConsumption,
            amount as f32
        ) as u32;

        // 2. 执行消耗
        if self.constitution >= effective_consumption {
            self.constitution -= effective_consumption;
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

    // === 关系系统方法 ===

    /// 获取与指定弟子的关系
    pub fn get_relationship(&self, target_id: usize) -> Option<&Relationship> {
        self.relationships.iter().find(|r| r.target_id == target_id)
    }

    /// 获取与指定弟子的关系（可变引用）
    pub fn get_relationship_mut(&mut self, target_id: usize) -> Option<&mut Relationship> {
        self.relationships.iter_mut().find(|r| r.target_id == target_id)
    }

    /// 添加或获取与指定弟子的关系
    pub fn get_or_create_relationship(&mut self, target_id: usize, year: u32) -> &mut Relationship {
        if !self.relationships.iter().any(|r| r.target_id == target_id) {
            self.relationships.push(Relationship::new(target_id, year));
        }
        self.get_relationship_mut(target_id).unwrap()
    }

    /// 移除与指定弟子的关系
    pub fn remove_relationship(&mut self, target_id: usize) {
        self.relationships.retain(|r| r.target_id != target_id);
    }

    /// 获取道侣ID（如果有）
    pub fn get_dao_companion_id(&self) -> Option<usize> {
        self.relationships
            .iter()
            .find(|r| r.is_dao_companion)
            .map(|r| r.target_id)
    }

    /// 获取师父ID（如果有）
    pub fn get_master_id(&self) -> Option<usize> {
        self.relationships
            .iter()
            .find(|r| r.is_master)
            .map(|r| r.target_id)
    }

    /// 获取所有徒弟ID
    pub fn get_disciple_ids(&self) -> Vec<usize> {
        self.relationships
            .iter()
            .filter(|r| r.is_disciple)
            .map(|r| r.target_id)
            .collect()
    }

    /// 是否有道侣
    pub fn has_dao_companion(&self) -> bool {
        self.relationships.iter().any(|r| r.is_dao_companion)
    }
}
