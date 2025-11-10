/// 小境界
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubLevel {
    Early,          // 初期
    Middle,         // 中期
    Perfect,        // 大圆满
}

impl SubLevel {
    pub fn next(&self) -> Option<SubLevel> {
        match self {
            SubLevel::Early => Some(SubLevel::Middle),
            SubLevel::Middle => Some(SubLevel::Perfect),
            SubLevel::Perfect => None,
        }
    }
}

impl std::fmt::Display for SubLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SubLevel::Early => "初期",
            SubLevel::Middle => "中期",
            SubLevel::Perfect => "大圆满",
        };
        write!(f, "{}", name)
    }
}

/// 修为等级系统
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CultivationLevel {
    QiRefining,      // 练气
    Foundation,      // 筑基
    GoldenCore,      // 结丹
    NascentSoul,     // 凝婴
    SpiritSevering,  // 化神
    VoidRefinement,  // 练虚
    Ascension,       // 飞升
}

impl CultivationLevel {
    /// 获取基础寿元
    pub fn base_lifespan(&self) -> u32 {
        match self {
            CultivationLevel::QiRefining => 150,
            CultivationLevel::Foundation => 300,
            CultivationLevel::GoldenCore => 500,
            CultivationLevel::NascentSoul => 1000,
            CultivationLevel::SpiritSevering => 2000,
            CultivationLevel::VoidRefinement => 5000,
            CultivationLevel::Ascension => u32::MAX,
        }
    }

    /// 是否需要渡劫
    pub fn requires_tribulation(&self) -> bool {
        matches!(
            self,
            CultivationLevel::Foundation
                | CultivationLevel::GoldenCore
                | CultivationLevel::NascentSoul
                | CultivationLevel::SpiritSevering
                | CultivationLevel::VoidRefinement
                | CultivationLevel::Ascension
        )
    }

    /// 下一个等级
    pub fn next(&self) -> Option<CultivationLevel> {
        match self {
            CultivationLevel::QiRefining => Some(CultivationLevel::Foundation),
            CultivationLevel::Foundation => Some(CultivationLevel::GoldenCore),
            CultivationLevel::GoldenCore => Some(CultivationLevel::NascentSoul),
            CultivationLevel::NascentSoul => Some(CultivationLevel::SpiritSevering),
            CultivationLevel::SpiritSevering => Some(CultivationLevel::VoidRefinement),
            CultivationLevel::VoidRefinement => Some(CultivationLevel::Ascension),
            CultivationLevel::Ascension => None,
        }
    }

    /// 获取数值等级（用于计算）
    /// 练气=0, 筑基=1, 结丹=2, 凝婴=3, 化神=4, 练虚=5, 飞升=6
    pub fn to_numeric(&self) -> u32 {
        *self as u32
    }
}

impl std::fmt::Display for CultivationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            CultivationLevel::QiRefining => "练气",
            CultivationLevel::Foundation => "筑基",
            CultivationLevel::GoldenCore => "结丹",
            CultivationLevel::NascentSoul => "凝婴",
            CultivationLevel::SpiritSevering => "化神",
            CultivationLevel::VoidRefinement => "练虚",
            CultivationLevel::Ascension => "飞升",
        };
        write!(f, "{}", name)
    }
}

/// 修炼路径 - 需要完成的任务类型和数量
#[derive(Debug, Clone)]
pub struct CultivationPath {
    pub required: std::collections::HashMap<String, u32>,  // 需要完成的任务类型和数量
    pub completed: std::collections::HashMap<String, u32>, // 每种类型已完成的数量
}

impl CultivationPath {
    /// 创建一个新的空修炼路径
    pub fn new() -> Self {
        Self {
            required: std::collections::HashMap::new(),
            completed: std::collections::HashMap::new(),
        }
    }

    /// 创建带有要求的修炼路径
    pub fn with_requirements(requirements: std::collections::HashMap<String, u32>) -> Self {
        let mut completed = std::collections::HashMap::new();
        for task_type in requirements.keys() {
            completed.insert(task_type.clone(), 0);
        }
        Self {
            required: requirements,
            completed,
        }
    }

    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        for (task_type, required_count) in &self.required {
            let completed_count = self.completed.get(task_type).unwrap_or(&0);
            if completed_count < required_count {
                return false;
            }
        }
        true
    }

    /// 完成一个指定类型的任务
    pub fn complete_task_by_type(&mut self, task_type: &str) -> bool {
        if let Some(&required_count) = self.required.get(task_type) {
            let completed_count = self.completed.entry(task_type.to_string()).or_insert(0);
            if *completed_count < required_count {
                *completed_count += 1;
                return true;
            }
        }
        false
    }

    /// 获取总进度
    pub fn progress(&self) -> (u32, u32) {
        let total_required: u32 = self.required.values().sum();
        let total_completed: u32 = self.completed.values().sum();
        (total_completed, total_required)
    }

    /// 获取每种类型的进度
    pub fn progress_by_type(&self, task_type: &str) -> (u32, u32) {
        let required = self.required.get(task_type).copied().unwrap_or(0);
        let completed = self.completed.get(task_type).copied().unwrap_or(0);
        (completed, required)
    }
}
