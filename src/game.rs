use crate::event::{EventSystem, GameEvent, RecruitmentSystem, TaskResult};
use crate::map::GameMap;
use crate::sect::Sect;
use crate::task::Task;
use rand::Rng;

/// 游戏状态
#[derive(Debug, PartialEq)]
pub enum GameState {
    Running,
    Victory,  // 成为仙门
    Defeat,   // 灭门或怪物成魔
}

/// 游戏主循环
pub struct Game {
    pub sect: Sect,
    pub map: GameMap,
    pub event_system: EventSystem,
    pub recruitment_system: RecruitmentSystem,
    pub state: GameState,
}

impl Game {
    pub fn new(sect_name: String) -> Self {
        let mut map = GameMap::new();

        // 使用静态地图数据初始化（早期版本）
        let static_map_data = GameMap::create_default_static_map();
        map.initialize_from_static(static_map_data);

        // 旧的程序化生成地图初始化（已注释）
        // map.initialize();

        let mut game = Self {
            sect: Sect::new(sect_name),
            map,
            event_system: EventSystem::new(),
            recruitment_system: RecruitmentSystem::new(),
            state: GameState::Running,
        };

        // 初始招募几个弟子
        for _ in 0..3 {
            let disciple = game.recruitment_system.generate_random_disciple();
            let id = disciple.id;
            game.sect.recruit_disciple(disciple);
            game.event_system
                .add_event(GameEvent::DiscipleRecruited(id));
        }

        game
    }

    /// 游戏年度循环
    pub fn yearly_cycle(&mut self) {
        println!("\n========== 第{}年 ==========", self.sect.year + 1);

        // 1. 年度收入
        let income = self.map.calculate_income(self.sect.reputation);
        self.event_system.add_event(GameEvent::YearlyIncome(income));

        // 2. 尝试招募弟子
        if let Some(disciple) = self.recruitment_system.try_recruit(&self.sect) {
            let id = disciple.id;
            self.sect.recruit_disciple(disciple);
            self.event_system
                .add_event(GameEvent::DiscipleRecruited(id));
        }

        // 3. 生成任务
        let tasks = self.map.get_available_tasks();
        if !tasks.is_empty() {
            self.event_system
                .add_event(GameEvent::TaskAvailable(tasks.clone()));

            // 自动分配任务给弟子
            self.auto_assign_tasks(tasks);
        }

        // 4. 弟子年龄增长和寿元检查
        self.sect.yearly_update();

        // 5. 检查突破
        self.check_breakthroughs();

        // 6. 地图更新
        self.map.update();
        self.event_system.add_event(GameEvent::MapUpdate);

        // 7. 处理事件
        self.event_system.process_events(&mut self.sect);

        // 8. 检查游戏状态
        self.check_game_state();

        // 9. 显示统计
        println!("\n{}", self.sect.get_statistics());
    }

    /// 自动分配任务
    fn auto_assign_tasks(&mut self, tasks: Vec<Task>) {
        let mut rng = rand::thread_rng();

        for task in tasks {
            // 找到合适的弟子
            let suitable_disciples: Vec<usize> = self
                .sect
                .alive_disciples()
                .iter()
                .filter(|d| task.is_suitable_for_disciple(d))
                .map(|d| d.id)
                .collect();

            if !suitable_disciples.is_empty() {
                // 随机选择一个弟子执行任务
                let disciple_id = suitable_disciples[rng.gen_range(0..suitable_disciples.len())];

                // 执行任务
                self.execute_task(disciple_id, task);
            }
        }
    }

    /// 执行任务
    fn execute_task(&mut self, disciple_id: usize, task: Task) {
        let mut rng = rand::thread_rng();

        // 任务成功率基于弟子修为和任务难度
        let success = rng.gen_bool(0.8); // 简化版，80%成功率

        if success {
            if let Some(disciple) = self
                .sect
                .disciples
                .iter_mut()
                .find(|d| d.id == disciple_id)
            {
                let progress_gained = disciple.complete_task(&task);

                // 更新道心
                disciple.dao_heart =
                    ((disciple.dao_heart as i32 + task.dao_heart_impact).max(0) as u32).min(100);

                let result = TaskResult {
                    task_id: task.id,
                    disciple_id,
                    success: true,
                    resources_gained: task.resource_reward,
                    reputation_gained: task.reputation_reward,
                    progress_gained,
                };

                self.event_system
                    .add_event(GameEvent::TaskCompleted(result));
            }
        }
    }

    /// 检查弟子是否可以突破
    fn check_breakthroughs(&mut self) {
        let mut breakthrough_disciples = Vec::new();
        let mut tribulation_results = Vec::new();

        for disciple in self.sect.alive_disciples_mut() {
            if disciple.cultivation.can_tribulate() {
                if disciple.cultivation.current_level.requires_tribulation() {
                    // 需要渡劫
                    let success = disciple.attempt_tribulation();
                    tribulation_results.push((disciple.id, success));

                    if !success {
                        // 渡劫失败，弟子身死
                        self.event_system
                            .add_event(GameEvent::DiscipleDeath(disciple.id));
                    }
                } else {
                    // 直接突破
                    if disciple.breakthrough() {
                        breakthrough_disciples.push(disciple.id);
                    }
                }
            }
        }

        for id in breakthrough_disciples {
            self.event_system
                .add_event(GameEvent::DiscipleBreakthrough(id));
        }

        for (id, success) in tribulation_results {
            self.event_system
                .add_event(GameEvent::DiscipleTribulation(id, success));
        }
    }

    /// 检查游戏状态
    fn check_game_state(&mut self) {
        // 检查是否成为仙门
        if self.sect.check_immortal_sect() {
            println!("\n🎉 恭喜！宗门有弟子飞升成仙，成为仙门！");
            self.state = GameState::Victory;
            return;
        }

        // 检查是否灭门
        if self.sect.is_destroyed() {
            println!("\n💀 宗门所有弟子寿元耗尽，宗门覆灭...");
            self.state = GameState::Defeat;
            return;
        }

        // 检查是否有怪物成魔
        if self.map.has_demon() {
            println!("\n👹 地图上出现了成魔的怪物，天下大乱，游戏结束！");
            self.state = GameState::Defeat;
            return;
        }
    }

    /// 运行游戏
    pub fn run(&mut self, max_years: u32) {
        println!("欢迎来到修仙模拟器！");
        println!("宗门名称：{}", self.sect.name);
        println!("\n游戏开始！\n");

        for _ in 0..max_years {
            if self.state != GameState::Running {
                break;
            }

            self.yearly_cycle();

            // 简单延迟，让玩家看清楚
            // std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // 游戏结束统计
        self.print_final_statistics();
    }

    /// 打印最终统计
    fn print_final_statistics(&self) {
        println!("\n========== 游戏结束 ==========");
        println!("最终状态: {:?}", self.state);
        println!("\n{}", self.sect.get_statistics());

        println!("\n存活弟子名单：");
        for disciple in self.sect.alive_disciples() {
            println!(
                "  {} - {} ({}) - 年龄: {}/{}",
                disciple.name,
                disciple.cultivation.current_level,
                disciple.disciple_type_str(),
                disciple.age,
                disciple.lifespan
            );
        }

        if !self.sect.heritages.is_empty() {
            println!("\n传承列表：");
            for heritage in &self.sect.heritages {
                println!("  {} ({}期)", heritage.name, heritage.level);
            }
        }
    }

    /// 显示当前弟子详情
    pub fn show_disciples(&self) {
        println!("\n========== 弟子列表 ==========");
        for disciple in self.sect.alive_disciples() {
            println!("\n名字: {}", disciple.name);
            println!("类型: {}", disciple.disciple_type_str());
            println!("修为: {} ({}%)", disciple.cultivation.current_level, disciple.cultivation.progress);
            println!("年龄: {}/{}", disciple.age, disciple.lifespan);
            println!("道心: {}", disciple.dao_heart);

            if !disciple.talents.is_empty() {
                print!("资质: ");
                for talent in &disciple.talents {
                    print!("{:?}({}) ", talent.talent_type, talent.level);
                }
                println!();
            }

            if let Some(ref heritage) = disciple.heritage {
                println!("传承: {} (渡劫加成: {:.1}%)", heritage.name, heritage.tribulation_bonus * 100.0);
            }
        }
    }
}
