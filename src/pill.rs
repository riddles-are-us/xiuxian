/// 丹药类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PillType {
    QiRecovery,         // 回气丹 - 恢复精力
    BodyStrength,       // 健体丹 - 恢复体魄
    VitalityElixir,     // 元气丹 - 同时恢复精力和体魄
    CultivationBoost,   // 修炼丹 - 增加修为进度（未来扩展）
}

impl PillType {
    /// 从字符串解析丹药类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "QiRecovery" => Some(PillType::QiRecovery),
            "BodyStrength" => Some(PillType::BodyStrength),
            "VitalityElixir" => Some(PillType::VitalityElixir),
            "CultivationBoost" => Some(PillType::CultivationBoost),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> &str {
        match self {
            PillType::QiRecovery => "QiRecovery",
            PillType::BodyStrength => "BodyStrength",
            PillType::VitalityElixir => "VitalityElixir",
            PillType::CultivationBoost => "CultivationBoost",
        }
    }

    /// 获取丹药名称
    pub fn name(&self) -> &str {
        match self {
            PillType::QiRecovery => "回气丹",
            PillType::BodyStrength => "健体丹",
            PillType::VitalityElixir => "元气丹",
            PillType::CultivationBoost => "修炼丹",
        }
    }

    /// 获取丹药描述
    pub fn description(&self) -> &str {
        match self {
            PillType::QiRecovery => "恢复30点精力",
            PillType::BodyStrength => "恢复30点体魄",
            PillType::VitalityElixir => "恢复20点精力和20点体魄",
            PillType::CultivationBoost => "增加修炼进度（未实现）",
        }
    }

    /// 获取丹药效果
    pub fn effects(&self) -> PillEffect {
        match self {
            PillType::QiRecovery => PillEffect {
                energy_restore: 30,
                constitution_restore: 0,
                cultivation_boost: 0,
            },
            PillType::BodyStrength => PillEffect {
                energy_restore: 0,
                constitution_restore: 30,
                cultivation_boost: 0,
            },
            PillType::VitalityElixir => PillEffect {
                energy_restore: 20,
                constitution_restore: 20,
                cultivation_boost: 0,
            },
            PillType::CultivationBoost => PillEffect {
                energy_restore: 0,
                constitution_restore: 0,
                cultivation_boost: 10,
            },
        }
    }

    /// 获取丹药炼制成本（资源）
    pub fn crafting_cost(&self) -> u32 {
        match self {
            PillType::QiRecovery => 50,
            PillType::BodyStrength => 50,
            PillType::VitalityElixir => 100,
            PillType::CultivationBoost => 200,
        }
    }
}

/// 丹药效果
#[derive(Debug, Clone, Copy)]
pub struct PillEffect {
    pub energy_restore: u32,        // 恢复精力
    pub constitution_restore: u32,  // 恢复体魄
    pub cultivation_boost: u32,     // 增加修为进度
}

/// 丹药库存
#[derive(Debug, Clone)]
pub struct PillInventory {
    pub pills: std::collections::HashMap<PillType, u32>,
}

impl PillInventory {
    pub fn new() -> Self {
        let mut pills = std::collections::HashMap::new();
        // 初始库存
        pills.insert(PillType::QiRecovery, 10);
        pills.insert(PillType::BodyStrength, 10);
        pills.insert(PillType::VitalityElixir, 5);
        pills.insert(PillType::CultivationBoost, 0);

        Self { pills }
    }

    /// 获取某种丹药的数量
    pub fn get_count(&self, pill_type: PillType) -> u32 {
        *self.pills.get(&pill_type).unwrap_or(&0)
    }

    /// 添加丹药
    pub fn add(&mut self, pill_type: PillType, count: u32) {
        *self.pills.entry(pill_type).or_insert(0) += count;
    }

    /// 使用丹药（返回是否成功）
    pub fn consume(&mut self, pill_type: PillType) -> bool {
        if let Some(count) = self.pills.get_mut(&pill_type) {
            if *count > 0 {
                *count -= 1;
                return true;
            }
        }
        false
    }

    /// 炼制丹药（消耗资源）
    pub fn craft(&mut self, pill_type: PillType, resources: &mut u32) -> bool {
        let cost = pill_type.crafting_cost();
        if *resources >= cost {
            *resources -= cost;
            self.add(pill_type, 1);
            true
        } else {
            false
        }
    }
}
