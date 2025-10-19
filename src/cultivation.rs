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
