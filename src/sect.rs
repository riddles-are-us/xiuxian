use crate::disciple::{Disciple, DiscipleType, Heritage};
use crate::cultivation::CultivationLevel;

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
        }
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
