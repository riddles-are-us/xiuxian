use serde::{Deserialize, Serialize};
use crate::task::TaskType;

/// 关系维度类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationDimension {
    Romance,       // 男女情感
    Mentorship,    // 师徒关系
    Comrade,       // 战友关系
    Understanding, // 认知程度
    FatefulBond,   // 机缘关系
}

impl RelationDimension {
    /// 获取维度的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            RelationDimension::Romance => "情感",
            RelationDimension::Mentorship => "师徒",
            RelationDimension::Comrade => "战友",
            RelationDimension::Understanding => "认知",
            RelationDimension::FatefulBond => "机缘",
        }
    }

    /// 获取所有维度
    pub fn all() -> Vec<RelationDimension> {
        vec![
            RelationDimension::Romance,
            RelationDimension::Mentorship,
            RelationDimension::Comrade,
            RelationDimension::Understanding,
            RelationDimension::FatefulBond,
        ]
    }
}

/// 关系等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationLevel {
    Stranger,      // 陌生 (0-19)
    Acquaintance,  // 一面之缘 (20-39)
    Familiar,      // 熟识 (40-59)
    Close,         // 亲近 (60-79)
    Intimate,      // 亲密无间 (80-99)
    Destined,      // 命定之人 (100)
}

impl RelationLevel {
    /// 根据分数获取等级
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=19 => RelationLevel::Stranger,
            20..=39 => RelationLevel::Acquaintance,
            40..=59 => RelationLevel::Familiar,
            60..=79 => RelationLevel::Close,
            80..=99 => RelationLevel::Intimate,
            _ => RelationLevel::Destined,
        }
    }

    /// 获取等级的中文名称
    pub fn name(&self) -> &'static str {
        match self {
            RelationLevel::Stranger => "陌生",
            RelationLevel::Acquaintance => "一面之缘",
            RelationLevel::Familiar => "熟识",
            RelationLevel::Close => "亲近",
            RelationLevel::Intimate => "亲密无间",
            RelationLevel::Destined => "命定之人",
        }
    }

    /// 获取等级的最低分数
    pub fn min_score(&self) -> u32 {
        match self {
            RelationLevel::Stranger => 0,
            RelationLevel::Acquaintance => 20,
            RelationLevel::Familiar => 40,
            RelationLevel::Close => 60,
            RelationLevel::Intimate => 80,
            RelationLevel::Destined => 100,
        }
    }
}

/// 关系分数结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationScores {
    pub romance: u32,       // 男女情感 0-100
    pub mentorship: u32,    // 师徒关系 0-100
    pub comrade: u32,       // 战友关系 0-100
    pub understanding: u32, // 认知程度 0-100
    pub fateful_bond: u32,  // 机缘关系 0-100
}

impl RelationScores {
    /// 创建新的关系分数（全部为0）
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取指定维度的分数
    pub fn get(&self, dimension: RelationDimension) -> u32 {
        match dimension {
            RelationDimension::Romance => self.romance,
            RelationDimension::Mentorship => self.mentorship,
            RelationDimension::Comrade => self.comrade,
            RelationDimension::Understanding => self.understanding,
            RelationDimension::FatefulBond => self.fateful_bond,
        }
    }

    /// 设置指定维度的分数
    pub fn set(&mut self, dimension: RelationDimension, value: u32) {
        let value = value.min(100); // 确保不超过100
        match dimension {
            RelationDimension::Romance => self.romance = value,
            RelationDimension::Mentorship => self.mentorship = value,
            RelationDimension::Comrade => self.comrade = value,
            RelationDimension::Understanding => self.understanding = value,
            RelationDimension::FatefulBond => self.fateful_bond = value,
        }
    }

    /// 增加指定维度的分数
    pub fn add(&mut self, dimension: RelationDimension, delta: i32) -> (u32, Option<RelationLevel>) {
        let old_score = self.get(dimension);
        let old_level = RelationLevel::from_score(old_score);

        let new_score = if delta >= 0 {
            (old_score as i32 + delta).min(100) as u32
        } else {
            old_score.saturating_sub((-delta) as u32)
        };

        self.set(dimension, new_score);
        let new_level = RelationLevel::from_score(new_score);

        // 如果等级提升了，返回新等级
        if new_level > old_level {
            (new_score, Some(new_level))
        } else {
            (new_score, None)
        }
    }

    /// 获取指定维度的等级
    pub fn get_level(&self, dimension: RelationDimension) -> RelationLevel {
        RelationLevel::from_score(self.get(dimension))
    }

    /// 获取所有维度的等级
    pub fn get_all_levels(&self) -> Vec<(RelationDimension, RelationLevel)> {
        RelationDimension::all()
            .into_iter()
            .map(|dim| (dim, self.get_level(dim)))
            .collect()
    }

    /// 获取最高的关系等级
    pub fn highest_level(&self) -> RelationLevel {
        RelationDimension::all()
            .into_iter()
            .map(|dim| self.get_level(dim))
            .max()
            .unwrap_or(RelationLevel::Stranger)
    }

    /// 检查是否可以成为道侣（情感 >= 80）
    pub fn can_be_dao_companion(&self) -> bool {
        self.romance >= 80
    }

    /// 检查是否有师徒关系（师徒 >= 40）
    pub fn has_mentorship(&self) -> bool {
        self.mentorship >= 40
    }
}

/// 关系增长值
#[derive(Debug, Clone, Default)]
pub struct RelationGrowth {
    pub romance: i32,
    pub mentorship: i32,
    pub comrade: i32,
    pub understanding: i32,
    pub fateful_bond: i32,
}

impl RelationGrowth {
    /// 根据任务类型计算关系增长
    pub fn from_task_type(task_type: &TaskType) -> Self {
        match task_type {
            TaskType::Combat(_) => {
                // 战斗任务: 战友+3, 认知+1
                RelationGrowth {
                    comrade: 3,
                    understanding: 1,
                    ..Default::default()
                }
            }
            TaskType::Exploration(_) => {
                // 探索任务: 机缘+2, 认知+2
                RelationGrowth {
                    fateful_bond: 2,
                    understanding: 2,
                    ..Default::default()
                }
            }
            TaskType::Gathering(_) => {
                // 采集任务: 认知+2
                RelationGrowth {
                    understanding: 2,
                    ..Default::default()
                }
            }
            TaskType::Auxiliary(_) => {
                // 辅助任务: 认知+1, 情感+1
                RelationGrowth {
                    understanding: 1,
                    romance: 1,
                    ..Default::default()
                }
            }
            TaskType::Investment(_) => {
                // 投资任务: 无增长
                RelationGrowth::default()
            }
        }
    }

    /// 应用增长到分数
    pub fn apply_to(&self, scores: &mut RelationScores) -> Vec<(RelationDimension, RelationLevel)> {
        let mut level_ups = Vec::new();

        if self.romance != 0 {
            if let (_, Some(level)) = scores.add(RelationDimension::Romance, self.romance) {
                level_ups.push((RelationDimension::Romance, level));
            }
        }
        if self.mentorship != 0 {
            if let (_, Some(level)) = scores.add(RelationDimension::Mentorship, self.mentorship) {
                level_ups.push((RelationDimension::Mentorship, level));
            }
        }
        if self.comrade != 0 {
            if let (_, Some(level)) = scores.add(RelationDimension::Comrade, self.comrade) {
                level_ups.push((RelationDimension::Comrade, level));
            }
        }
        if self.understanding != 0 {
            if let (_, Some(level)) = scores.add(RelationDimension::Understanding, self.understanding) {
                level_ups.push((RelationDimension::Understanding, level));
            }
        }
        if self.fateful_bond != 0 {
            if let (_, Some(level)) = scores.add(RelationDimension::FatefulBond, self.fateful_bond) {
                level_ups.push((RelationDimension::FatefulBond, level));
            }
        }

        level_ups
    }
}

/// 单个关系（从一个弟子到另一个弟子的单向关系）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_id: usize,          // 目标弟子ID
    pub scores: RelationScores,    // 各维度分数
    pub established_year: u32,     // 建立关系的年份
    pub is_dao_companion: bool,    // 是否是道侣（需要双方确认）
    pub is_master: bool,           // 目标是否是自己的师父
    pub is_disciple: bool,         // 目标是否是自己的徒弟
}

impl Relationship {
    /// 创建新的关系
    pub fn new(target_id: usize, year: u32) -> Self {
        Self {
            target_id,
            scores: RelationScores::new(),
            established_year: year,
            is_dao_companion: false,
            is_master: false,
            is_disciple: false,
        }
    }

    /// 创建师徒关系（作为徒弟）
    pub fn new_as_disciple_of(master_id: usize, year: u32) -> Self {
        let mut rel = Self::new(master_id, year);
        rel.is_master = true;
        rel.scores.mentorship = 50; // 初始师徒分数
        rel
    }

    /// 创建师徒关系（作为师父）
    pub fn new_as_master_of(disciple_id: usize, year: u32) -> Self {
        let mut rel = Self::new(disciple_id, year);
        rel.is_disciple = true;
        rel.scores.mentorship = 50; // 初始师徒分数
        rel
    }

    /// 应用任务带来的关系增长
    pub fn apply_task_growth(&mut self, task_type: &TaskType) -> Vec<(RelationDimension, RelationLevel)> {
        let growth = RelationGrowth::from_task_type(task_type);
        growth.apply_to(&mut self.scores)
    }

    /// 获取主要关系类型描述
    pub fn get_primary_relation(&self) -> &'static str {
        if self.is_dao_companion {
            return "道侣";
        }
        if self.is_master {
            return "师父";
        }
        if self.is_disciple {
            return "徒弟";
        }

        // 根据最高分数判断
        let highest = self.scores.highest_level();
        if highest == RelationLevel::Stranger {
            return "陌生人";
        }

        // 找出分数最高的维度
        let dims = [
            (self.scores.romance, "知己"),
            (self.scores.comrade, "战友"),
            (self.scores.understanding, "旧识"),
            (self.scores.fateful_bond, "有缘人"),
        ];

        dims.iter()
            .max_by_key(|(score, _)| *score)
            .map(|(_, name)| *name)
            .unwrap_or("相识")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_level_from_score() {
        assert_eq!(RelationLevel::from_score(0), RelationLevel::Stranger);
        assert_eq!(RelationLevel::from_score(19), RelationLevel::Stranger);
        assert_eq!(RelationLevel::from_score(20), RelationLevel::Acquaintance);
        assert_eq!(RelationLevel::from_score(50), RelationLevel::Familiar);
        assert_eq!(RelationLevel::from_score(79), RelationLevel::Close);
        assert_eq!(RelationLevel::from_score(80), RelationLevel::Intimate);
        assert_eq!(RelationLevel::from_score(100), RelationLevel::Destined);
    }

    #[test]
    fn test_relation_scores_add() {
        let mut scores = RelationScores::new();

        // 添加分数
        let (new_score, level_up) = scores.add(RelationDimension::Romance, 25);
        assert_eq!(new_score, 25);
        assert_eq!(level_up, Some(RelationLevel::Acquaintance));

        // 继续添加
        let (new_score, level_up) = scores.add(RelationDimension::Romance, 20);
        assert_eq!(new_score, 45);
        assert_eq!(level_up, Some(RelationLevel::Familiar));

        // 减少分数
        let (new_score, level_up) = scores.add(RelationDimension::Romance, -10);
        assert_eq!(new_score, 35);
        assert_eq!(level_up, None); // 等级没有提升
    }

    #[test]
    fn test_relation_scores_cap() {
        let mut scores = RelationScores::new();
        scores.set(RelationDimension::Romance, 150); // 超过100
        assert_eq!(scores.romance, 100); // 应该被限制在100
    }
}
