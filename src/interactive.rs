use crate::disciple::Disciple;
use crate::event::{EventSystem, RecruitmentSystem, TaskResult};
use crate::map::GameMap;
use crate::sect::Sect;
use crate::task::Task;
use crate::ui::UI;
use rand::Rng;

/// 游戏状态
#[derive(Debug, PartialEq)]
pub enum GameState {
    Running,
    Victory,
    Defeat,
}

/// 回合中的任务分配
#[derive(Debug)]
pub struct TaskAssignment {
    pub task_id: usize,
    pub disciple_id: Option<usize>,
}

/// 交互式游戏
pub struct InteractiveGame {
    pub sect: Sect,
    pub map: GameMap,
    pub event_system: EventSystem,
    pub recruitment_system: RecruitmentSystem,
    pub state: GameState,
    pub current_tasks: Vec<Task>,
    pub task_assignments: Vec<TaskAssignment>,
    pub is_web_mode: bool, // Web模式下不显示UI和等待输入
}

impl InteractiveGame {
    pub fn new(sect_name: String) -> Self {
        Self::new_with_mode(sect_name, false)
    }

    pub fn new_with_mode(sect_name: String, is_web_mode: bool) -> Self {
        let mut map = GameMap::new();
        map.initialize();

        let mut game = Self {
            sect: Sect::new(sect_name),
            map,
            event_system: EventSystem::new(),
            recruitment_system: RecruitmentSystem::new(),
            state: GameState::Running,
            current_tasks: Vec::new(),
            task_assignments: Vec::new(),
            is_web_mode,
        };

        // 初始招募3个弟子
        for _ in 0..3 {
            let disciple = game.recruitment_system.generate_random_disciple();
            game.sect.recruit_disciple(disciple);
        }

        game
    }

    /// 开始新的回合
    pub fn start_turn(&mut self) {
        self.sect.year += 1;

        if !self.is_web_mode {
            UI::clear_screen();
            UI::print_title(&format!("第 {} 年", self.sect.year));
        }

        // 1. 年度收入
        let income = self.map.calculate_income(self.sect.reputation);
        self.sect.add_resources(income);
        if !self.is_web_mode {
            UI::success(&format!("年度收入：{} 资源", income));
        }

        // 2. 尝试招募弟子
        if let Some(disciple) = self.recruitment_system.try_recruit(&self.sect) {
            if !self.is_web_mode {
                UI::success(&format!(
                    "新弟子加入：{} ({})",
                    disciple.name,
                    self.disciple_type_str(&disciple)
                ));
            }
            self.sect.recruit_disciple(disciple);
        }

        // 3. 弟子年龄增长和寿元检查
        self.sect.yearly_update();

        // 4. 检查突破
        self.check_breakthroughs();

        // 5. 生成任务
        self.current_tasks = self.map.get_available_tasks();
        self.task_assignments = self
            .current_tasks
            .iter()
            .map(|t| TaskAssignment {
                task_id: t.id,
                disciple_id: None,
            })
            .collect();

        // 6. 地图更新
        self.map.update();

        if !self.is_web_mode {
            UI::wait_for_enter("\n按回车键继续...");
        }
    }

    /// 显示回合信息
    pub fn show_turn_info(&self) {
        if self.is_web_mode {
            return; // Web模式下不显示UI
        }

        UI::clear_screen();
        UI::print_title(&format!("第 {} 年 - 回合信息", self.sect.year));

        // 显示宗门状态
        println!("{}", self.sect.get_statistics());

        UI::wait_for_enter("\n按回车键继续...");
    }

    /// 显示所有弟子
    pub fn show_all_disciples(&self) {
        UI::clear_screen();
        UI::print_title("弟子列表");

        let disciples = self.sect.alive_disciples();
        if disciples.is_empty() {
            UI::warning("没有存活的弟子！");
            return;
        }

        for (i, disciple) in disciples.iter().enumerate() {
            println!("\n[{}] {}", i + 1, disciple.name);
            println!("    类型: {}", self.disciple_type_str(disciple));
            println!(
                "    修为: {} (进度: {}%)",
                disciple.cultivation.current_level, disciple.cultivation.progress
            );
            println!("    年龄: {}/{}", disciple.age, disciple.lifespan);
            println!("    道心: {}", disciple.dao_heart);

            if !disciple.talents.is_empty() {
                print!("    资质: ");
                for talent in &disciple.talents {
                    print!("{:?}(Lv{}) ", talent.talent_type, talent.level);
                }
                println!();
            }

            if let Some(ref heritage) = disciple.heritage {
                println!(
                    "    传承: {} (渡劫加成: {:.0}%)",
                    heritage.name,
                    heritage.tribulation_bonus * 100.0
                );
            }

            // 显示当前执行的任务
            let current_task = self.task_assignments
                .iter()
                .find(|a| a.disciple_id == Some(disciple.id))
                .and_then(|a| self.current_tasks.iter().find(|t| t.id == a.task_id));

            if let Some(task) = current_task {
                println!("    📋 当前任务: {}", task.name);
            } else {
                println!("    📋 当前任务: 空闲");
            }

            // 显示是否适合当前任务
            if !self.current_tasks.is_empty() {
                let suitable_tasks: Vec<&Task> = self
                    .current_tasks
                    .iter()
                    .filter(|t| t.is_suitable_for_disciple(disciple))
                    .collect();
                if !suitable_tasks.is_empty() {
                    println!("    可执行任务数: {}", suitable_tasks.len());
                }
            }
        }
    }

    /// 显示所有任务
    pub fn show_all_tasks(&self) {
        UI::clear_screen();
        UI::print_title("本回合可用任务");

        if self.current_tasks.is_empty() {
            UI::warning("本回合没有可用任务。");
            return;
        }

        for (i, task) in self.current_tasks.iter().enumerate() {
            let assignment = &self.task_assignments[i];

            print!("\n[{}] {} ", i + 1, task.name);

            if let Some(disciple_id) = assignment.disciple_id {
                if let Some(disciple) = self.sect.disciples.iter().find(|d| d.id == disciple_id) {
                    println!("✓ 已分配给: {}", disciple.name);
                }
            } else {
                println!("⭕ 未分配");
            }

            println!("    类型: {:?}", task.task_type);
            println!(
                "    奖励: 修为+{}, 资源+{}, 声望+{}",
                task.progress_reward, task.resource_reward, task.reputation_reward
            );

            if task.dao_heart_impact != 0 {
                println!("    道心影响: {:+}", task.dao_heart_impact);
            }

            // 显示适合的弟子，区分空闲和忙碌
            let mut suitable_free = Vec::new();
            let mut suitable_busy = Vec::new();

            for disciple in self.sect.alive_disciples() {
                if task.is_suitable_for_disciple(disciple) {
                    let is_busy = self.task_assignments
                        .iter()
                        .any(|a| a.disciple_id == Some(disciple.id));

                    if is_busy {
                        suitable_busy.push(disciple.name.clone());
                    } else {
                        suitable_free.push(disciple.name.clone());
                    }
                }
            }

            if !suitable_free.is_empty() {
                println!("    ✓ 空闲适合: {}", suitable_free.join(", "));
            }
            if !suitable_busy.is_empty() {
                println!("    ⚠️  忙碌中: {}", suitable_busy.join(", "));
            }
            if suitable_free.is_empty() && suitable_busy.is_empty() {
                println!("    ❌ 没有适合的弟子");
            }
        }
    }

    /// 分配任务
    pub fn assign_tasks(&mut self) {
        loop {
            self.show_all_tasks();

            UI::print_separator();
            println!("1. 分配任务");
            println!("2. 取消分配");
            println!("3. 自动分配所有未分配任务");
            println!("4. 完成分配，进入下一阶段");

            let choice = UI::get_number_input("\n请选择操作: ", 1, 4);

            match choice {
                Some(1) => self.assign_single_task(),
                Some(2) => self.unassign_task(),
                Some(3) => self.auto_assign_remaining(),
                Some(4) => break,
                _ => {}
            }
        }
    }

    /// 分配单个任务
    fn assign_single_task(&mut self) {
        UI::print_subtitle("分配任务");

        if self.current_tasks.is_empty() {
            UI::error("没有可用任务");
            UI::wait_for_enter("\n按回车继续...");
            return;
        }

        // 选择任务
        println!("\n选择要分配的任务:");
        for (i, task) in self.current_tasks.iter().enumerate() {
            let status = if self.task_assignments[i].disciple_id.is_some() {
                "✓"
            } else {
                "⭕"
            };
            println!("  [{}] {} {}", i + 1, status, task.name);
        }

        let task_choice = UI::get_number_input("\n任务序号 (0=取消): ", 0, self.current_tasks.len());
        if task_choice.is_none() || task_choice == Some(0) {
            return;
        }

        let task_idx = task_choice.unwrap() - 1;
        let task = &self.current_tasks[task_idx];

        // 显示适合的弟子（排除已分配任务的弟子）
        let disciples = self.sect.alive_disciples();
        let suitable: Vec<(usize, &Disciple)> = disciples
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                // 必须适合该任务
                task.is_suitable_for_disciple(*d) &&
                // 并且当前没有分配任务
                !self.task_assignments.iter().any(|a| a.disciple_id == Some(d.id))
            })
            .map(|(i, d)| (i, *d))
            .collect();

        if suitable.is_empty() {
            UI::error("没有适合该任务的空闲弟子（可能都已被分配任务）");
            UI::wait_for_enter("\n按回车继续...");
            return;
        }

        println!("\n选择执行弟子:");
        for (i, (_, disciple)) in suitable.iter().enumerate() {
            let status = if disciple.current_task.is_some() {
                "（忙碌）"
            } else {
                ""
            };
            println!(
                "  [{}] {} - {} ({}%) {}",
                i + 1,
                disciple.name,
                disciple.cultivation.current_level,
                disciple.cultivation.progress,
                status
            );
        }

        let disciple_choice = UI::get_number_input("\n弟子序号 (0=取消): ", 0, suitable.len());
        if disciple_choice.is_none() || disciple_choice == Some(0) {
            return;
        }

        let (_, selected_disciple) = suitable[disciple_choice.unwrap() - 1];
        self.task_assignments[task_idx].disciple_id = Some(selected_disciple.id);

        UI::success(&format!(
            "已将任务 [{}] 分配给 {}",
            task.name, selected_disciple.name
        ));
        UI::wait_for_enter("\n按回车继续...");
    }

    /// 取消任务分配
    fn unassign_task(&mut self) {
        UI::print_subtitle("取消任务分配");

        let assigned: Vec<(usize, &Task)> = self
            .current_tasks
            .iter()
            .enumerate()
            .filter(|(i, _)| self.task_assignments[*i].disciple_id.is_some())
            .collect();

        if assigned.is_empty() {
            UI::error("没有已分配的任务");
            UI::wait_for_enter("\n按回车继续...");
            return;
        }

        println!("\n选择要取消的任务:");
        for (i, (idx, task)) in assigned.iter().enumerate() {
            if let Some(disciple_id) = self.task_assignments[*idx].disciple_id {
                if let Some(d) = self.sect.disciples.iter().find(|d| d.id == disciple_id) {
                    println!("  [{}] {} (执行者: {})", i + 1, task.name, d.name);
                }
            }
        }

        let choice = UI::get_number_input("\n任务序号 (0=取消): ", 0, assigned.len());
        if choice.is_none() || choice == Some(0) {
            return;
        }

        let (task_idx, _) = assigned[choice.unwrap() - 1];
        self.task_assignments[task_idx].disciple_id = None;

        UI::success("已取消任务分配");
        UI::wait_for_enter("\n按回车继续...");
    }

    /// 自动分配剩余任务
    pub fn auto_assign_remaining(&mut self) {
        let mut assigned_count = 0;

        for i in 0..self.current_tasks.len() {
            if self.task_assignments[i].disciple_id.is_some() {
                continue;
            }

            let task = &self.current_tasks[i];

            // 找到适合的且未被分配任务的弟子
            let suitable: Vec<&Disciple> = self
                .sect
                .alive_disciples()
                .into_iter()
                .filter(|d| {
                    task.is_suitable_for_disciple(d) &&
                    // 确保该弟子还没有被分配任务
                    !self.task_assignments.iter().any(|a| a.disciple_id == Some(d.id))
                })
                .collect();

            if let Some(disciple) = suitable.first() {
                self.task_assignments[i].disciple_id = Some(disciple.id);
                assigned_count += 1;
            }
        }

        if !self.is_web_mode {
            UI::success(&format!("自动分配了 {} 个任务", assigned_count));
            UI::wait_for_enter("\n按回车继续...");
        }
    }

    /// 执行回合任务
    pub fn execute_turn(&mut self) {
        if !self.is_web_mode {
            UI::clear_screen();
            UI::print_title("任务执行结果");
        }

        // 先收集要执行的任务和弟子ID
        let mut tasks_to_execute = Vec::new();
        for assignment in &self.task_assignments {
            if let Some(disciple_id) = assignment.disciple_id {
                if let Some(task) = self.current_tasks.iter().find(|t| t.id == assignment.task_id)
                {
                    tasks_to_execute.push((disciple_id, task.clone()));
                }
            }
        }

        // 执行任务
        let mut results = Vec::new();
        for (disciple_id, task) in tasks_to_execute {
            let result = self.execute_single_task(disciple_id, task);
            results.push(result);
        }

        // 处理结果
        for result in results {
            if result.success {
                self.sect.add_resources(result.resources_gained);
                self.sect.add_reputation(result.reputation_gained);
            }
        }

        if !self.is_web_mode {
            UI::wait_for_enter("\n按回车键查看回合总结...");
        }
    }

    /// 执行单个任务
    fn execute_single_task(&mut self, disciple_id: usize, task: Task) -> TaskResult {
        let mut rng = rand::thread_rng();
        let success = rng.gen_bool(0.8);

        let disciple_name = self
            .sect
            .disciples
            .iter()
            .find(|d| d.id == disciple_id)
            .map(|d| d.name.clone())
            .unwrap_or_default();

        if success {
            if let Some(disciple) = self
                .sect
                .disciples
                .iter_mut()
                .find(|d| d.id == disciple_id)
            {
                let progress_gained = disciple.complete_task(&task);
                disciple.dao_heart =
                    ((disciple.dao_heart as i32 + task.dao_heart_impact).max(0) as u32).min(100);

                println!(
                    "✅ {} 完成任务 [{}]",
                    disciple_name, task.name
                );
                println!(
                    "   获得: 修为+{}, 资源+{}, 声望+{}",
                    progress_gained, task.resource_reward, task.reputation_reward
                );

                if task.dao_heart_impact != 0 {
                    println!("   道心变化: {:+}", task.dao_heart_impact);
                }

                TaskResult {
                    task_id: task.id,
                    disciple_id,
                    success: true,
                    resources_gained: task.resource_reward,
                    reputation_gained: task.reputation_reward,
                    progress_gained,
                }
            } else {
                TaskResult {
                    task_id: task.id,
                    disciple_id,
                    success: false,
                    resources_gained: 0,
                    reputation_gained: 0,
                    progress_gained: 0,
                }
            }
        } else {
            println!("❌ {} 执行任务 [{}] 失败", disciple_name, task.name);
            TaskResult {
                task_id: task.id,
                disciple_id,
                success: false,
                resources_gained: 0,
                reputation_gained: 0,
                progress_gained: 0,
            }
        }
    }

    /// 检查突破
    fn check_breakthroughs(&mut self) {
        let mut events = Vec::new();

        for disciple in self.sect.alive_disciples_mut() {
            if disciple.cultivation.is_perfect() {
                if disciple.cultivation.current_level.requires_tribulation() {
                    // 需要渡劫，询问用户
                    events.push((disciple.id, disciple.name.clone(), true));
                } else {
                    // 直接突破
                    if disciple.breakthrough() {
                        println!(
                            "✅ {} 成功突破至 {}！",
                            disciple.name, disciple.cultivation.current_level
                        );
                    }
                }
            }
        }

        // 处理渡劫
        for (id, name, _) in events {
            if let Some(disciple) = self.sect.disciples.iter().find(|d| d.id == id) {
                let success_rate = disciple.tribulation_success_rate();
                UI::warning(&format!(
                    "\n{} 已达到大圆满，可以尝试渡劫",
                    name
                ));
                println!("当前渡劫成功率: {:.1}%", success_rate * 100.0);
                println!("  道心: {}", disciple.dao_heart);
                if let Some(ref heritage) = disciple.heritage {
                    println!(
                        "  传承加成: {:.1}%",
                        heritage.tribulation_bonus * 100.0
                    );
                }

                if UI::confirm("\n是否尝试渡劫?") {
                    if let Some(disciple) = self.sect.disciples.iter_mut().find(|d| d.id == id) {
                        let success = disciple.attempt_tribulation();
                        if success {
                            UI::success(&format!(
                                "{} 渡劫成功！晋升至 {}",
                                name, disciple.cultivation.current_level
                            ));
                        } else {
                            UI::error(&format!("{} 渡劫失败，身死道消...", name));
                            // 弟子会在年度更新时处理
                        }
                    }
                } else {
                    UI::info(&format!("{} 选择继续修炼，等待时机", name));
                }
            }
        }
    }

    /// 检查游戏状态
    pub fn check_game_state(&mut self) -> bool {
        // 检查是否成为仙门
        if self.sect.check_immortal_sect() {
            if !self.is_web_mode {
                UI::clear_screen();
                UI::print_title("🎉 游戏胜利！");
                println!("\n恭喜！宗门有弟子飞升成仙，成为仙门！");
                println!("\n游戏用时: {} 年", self.sect.year);
            }
            self.state = GameState::Victory;
            return false;
        }

        // 检查是否灭门
        if self.sect.is_destroyed() {
            if !self.is_web_mode {
                UI::clear_screen();
                UI::print_title("💀 游戏失败");
                println!("\n宗门所有弟子寿元耗尽，宗门覆灭...");
                println!("\n游戏用时: {} 年", self.sect.year);
            }
            self.state = GameState::Defeat;
            return false;
        }

        // 检查是否有怪物成魔
        if self.map.has_demon() {
            if !self.is_web_mode {
                UI::clear_screen();
                UI::print_title("👹 游戏失败");
                println!("\n地图上出现了成魔的怪物，天下大乱！");
                println!("\n游戏用时: {} 年", self.sect.year);
            }
            self.state = GameState::Defeat;
            return false;
        }

        true
    }

    /// 显示主菜单并运行游戏
    pub fn run(&mut self) {
        UI::clear_screen();
        UI::print_title("修仙宗门模拟器");
        println!("\n宗门名称: {}", self.sect.name);
        println!("\n游戏目标:");
        println!("  胜利条件: 培养出飞升期弟子");
        println!("  失败条件: 所有弟子死亡 或 怪物成魔");

        UI::wait_for_enter("\n按回车开始游戏...");

        // 显示初始弟子
        self.show_all_disciples();
        UI::wait_for_enter("\n按回车开始第一回合...");

        loop {
            // 开始新回合
            self.start_turn();

            // 检查游戏状态
            if !self.check_game_state() {
                break;
            }

            // 回合循环
            loop {
                UI::clear_screen();
                UI::print_title(&format!("第 {} 年 - 主菜单", self.sect.year));

                let choice = UI::show_menu(
                    "请选择操作",
                    &[
                        "查看宗门状态",
                        "查看弟子列表",
                        "查看任务列表",
                        "分配任务",
                        "执行任务，结束回合",
                    ],
                );

                match choice {
                    0 => {
                        self.show_turn_info();
                    }
                    1 => {
                        self.show_all_disciples();
                        UI::wait_for_enter("\n按回车继续...");
                    }
                    2 => {
                        self.show_all_tasks();
                        UI::wait_for_enter("\n按回车继续...");
                    }
                    3 => {
                        self.assign_tasks();
                    }
                    4 => {
                        self.execute_turn();
                        break;
                    }
                    _ => {}
                }
            }

            // 检查游戏状态
            if !self.check_game_state() {
                break;
            }
        }

        // 显示最终统计
        self.show_final_statistics();
    }

    /// 显示最终统计
    fn show_final_statistics(&self) {
        UI::print_separator();
        println!("\n{}", self.sect.get_statistics());

        println!("\n存活弟子名单:");
        for disciple in self.sect.alive_disciples() {
            println!(
                "  {} - {} ({}) - 年龄: {}/{}",
                disciple.name,
                disciple.cultivation.current_level,
                self.disciple_type_str(disciple),
                disciple.age,
                disciple.lifespan
            );
        }

        if !self.sect.heritages.is_empty() {
            println!("\n传承列表:");
            for heritage in &self.sect.heritages {
                println!("  {} ({}期)", heritage.name, heritage.level);
            }
        }

        UI::print_separator();
    }

    /// 获取弟子类型字符串
    fn disciple_type_str(&self, disciple: &Disciple) -> &str {
        match disciple.disciple_type {
            crate::disciple::DiscipleType::Outer => "外门",
            crate::disciple::DiscipleType::Inner => "内门",
            crate::disciple::DiscipleType::Personal => "亲传",
        }
    }
}
