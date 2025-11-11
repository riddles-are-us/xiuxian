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
#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub task_id: usize,
    pub disciple_id: Option<usize>,
    pub started_turn: Option<u32>,  // 任务开始的回合数
    pub progress: u32,               // 已执行的回合数
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
        // 弟子年龄增长和寿元检查（这会增加年份）
        self.sect.yearly_update();

        // 弟子自然恢复精力和体魄
        for disciple in self.sect.alive_disciples_mut() {
            disciple.natural_recovery();
        }

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

        // 3. 清理过期任务
        self.remove_expired_tasks();

        // 4. 生成新任务
        let mut new_tasks = self.map.get_available_tasks();
        for task in &mut new_tasks {
            task.created_turn = self.sect.year;
        }

        // 过滤掉同一地点已有相同类型任务的新任务
        // 收集当前所有任务的 (location_id, task_type) 组合
        let existing_location_task_types: std::collections::HashSet<(String, &str)> = self
            .current_tasks
            .iter()
            .filter_map(|task| {
                task.location_id
                    .as_ref()
                    .map(|loc_id| (loc_id.clone(), task.get_task_type_str()))
            })
            .collect();

        // 过滤新任务：如果相同的 (location_id, task_type) 组合已存在，则排除
        let filtered_tasks: Vec<_> = new_tasks
            .into_iter()
            .filter(|task| {
                if let Some(ref loc_id) = task.location_id {
                    let key = (loc_id.clone(), task.get_task_type_str());
                    !existing_location_task_types.contains(&key)
                } else {
                    // 没有location_id的任务（如果有的话）不过滤
                    true
                }
            })
            .collect();

        self.current_tasks.extend(filtered_tasks);

        // 初始化新任务的分配记录
        let existing_task_ids: Vec<usize> = self.task_assignments.iter().map(|a| a.task_id).collect();
        for task in &self.current_tasks {
            if !existing_task_ids.contains(&task.id) {
                self.task_assignments.push(TaskAssignment {
                    task_id: task.id,
                    disciple_id: None,
                    started_turn: None,
                    progress: 0,
                });
            }
        }

        // 5. 检查突破和分配修炼路径（在任务生成之后）
        self.check_breakthroughs();

        // 6. 地图更新
        self.map.update();

        // 7. 检查守卫任务有效性（妖魔是否已离开）
        self.check_and_remove_invalid_defense_tasks();

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
            let assignment = self.task_assignments.iter().find(|a| a.task_id == task.id);

            print!("\n[{}] {} ", i + 1, task.name);

            if let Some(assignment) = assignment {
                if let Some(disciple_id) = assignment.disciple_id {
                    if let Some(disciple) = self.sect.disciples.iter().find(|d| d.id == disciple_id) {
                        println!("✓ 已分配给: {}", disciple.name);
                    }
                } else {
                    println!("⭕ 未分配");
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
            let assignment = self.task_assignments.iter().find(|a| a.task_id == task.id);
            let status = if assignment.and_then(|a| a.disciple_id).is_some() {
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
            let is_busy = self.task_assignments.iter().any(|a| a.disciple_id == Some(disciple.id));
            let status = if is_busy {
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

        // 查找任务的分配记录并更新
        if let Some(assignment) = self.task_assignments.iter_mut().find(|a| a.task_id == task.id) {
            assignment.disciple_id = Some(selected_disciple.id);
        }

        UI::success(&format!(
            "已将任务 [{}] 分配给 {}",
            task.name, selected_disciple.name
        ));
        UI::wait_for_enter("\n按回车继续...");
    }

    /// 取消任务分配
    fn unassign_task(&mut self) {
        UI::print_subtitle("取消任务分配");

        let assigned: Vec<&Task> = self
            .current_tasks
            .iter()
            .filter(|task| {
                self.task_assignments
                    .iter()
                    .find(|a| a.task_id == task.id)
                    .and_then(|a| a.disciple_id)
                    .is_some()
            })
            .collect();

        if assigned.is_empty() {
            UI::error("没有已分配的任务");
            UI::wait_for_enter("\n按回车继续...");
            return;
        }

        println!("\n选择要取消的任务:");
        for (i, task) in assigned.iter().enumerate() {
            let assignment = self.task_assignments.iter().find(|a| a.task_id == task.id);
            if let Some(assignment) = assignment {
                if let Some(disciple_id) = assignment.disciple_id {
                    if let Some(d) = self.sect.disciples.iter().find(|d| d.id == disciple_id) {
                        println!("  [{}] {} (执行者: {})", i + 1, task.name, d.name);
                    }
                }
            }
        }

        let choice = UI::get_number_input("\n任务序号 (0=取消): ", 0, assigned.len());
        if choice.is_none() || choice == Some(0) {
            return;
        }

        let selected_task = assigned[choice.unwrap() - 1];
        if let Some(assignment) = self.task_assignments.iter_mut().find(|a| a.task_id == selected_task.id) {
            assignment.disciple_id = None;
        }

        UI::success("已取消任务分配");
        UI::wait_for_enter("\n按回车继续...");
    }

    /// 自动分配剩余任务
    pub fn auto_assign_remaining(&mut self) {
        let mut assigned_count = 0;

        // 收集需要分配的任务ID和弟子ID对
        let mut assignments_to_make = Vec::new();

        for task in &self.current_tasks {
            // 查找该任务的分配记录
            let assignment = self.task_assignments.iter().find(|a| a.task_id == task.id);

            if let Some(assignment) = assignment {
                if assignment.disciple_id.is_some() {
                    continue; // 已分配，跳过
                }

                // 找到适合的且未被分配任务的弟子
                let suitable: Vec<&Disciple> = self
                    .sect
                    .alive_disciples()
                    .into_iter()
                    .filter(|d| {
                        task.is_suitable_for_disciple(d) &&
                        // 确保该弟子还没有被分配任务
                        !self.task_assignments.iter().any(|a| a.disciple_id == Some(d.id)) &&
                        // 也不在待分配列表中
                        !assignments_to_make.iter().any(|(_, did)| *did == d.id)
                    })
                    .collect();

                if let Some(disciple) = suitable.first() {
                    assignments_to_make.push((task.id, disciple.id));
                }
            }
        }

        // 执行分配
        for (task_id, disciple_id) in assignments_to_make {
            if let Some(assignment) = self.task_assignments.iter_mut().find(|a| a.task_id == task_id) {
                assignment.disciple_id = Some(disciple_id);
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

        // 更新任务进度并收集完成的任务
        let mut completed_tasks = Vec::new();

        for assignment in &mut self.task_assignments {
            if let Some(disciple_id) = assignment.disciple_id {
                // 如果任务刚开始，设置开始回合
                if assignment.started_turn.is_none() {
                    assignment.started_turn = Some(self.sect.year);
                }

                // 增加进度
                assignment.progress += 1;

                // 消耗精力和体魄（每回合）
                if let Some(task) = self.current_tasks.iter().find(|t| t.id == assignment.task_id) {
                    if let Some(disciple) = self.sect.disciples.iter_mut().find(|d| d.id == disciple_id) {
                        disciple.consume_energy(task.energy_cost);
                        disciple.consume_constitution(task.constitution_cost);
                    }

                    // 检查任务是否完成
                    if assignment.progress >= task.duration {
                        completed_tasks.push((disciple_id, task.clone()));
                    }
                }
            }
        }

        // 执行完成的任务
        let mut results = Vec::new();
        for (disciple_id, task) in completed_tasks {
            let result = self.execute_single_task(disciple_id, task.clone());
            results.push(result);

            // 从当前任务中移除已完成的任务
            self.current_tasks.retain(|t| t.id != task.id);
            self.task_assignments.retain(|a| a.task_id != task.id);

            // 清除妖魔的任务关联和解锁移动
            self.map.clear_monster_task(task.id);
            if task.name.contains("守卫") {
                if let crate::task::TaskType::Combat(combat_task) = &task.task_type {
                    self.map.unlock_monster_for_defense_task(&combat_task.enemy_name);
                }
            }
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

                // 获取任务类型字符串
                use crate::task::TaskType;
                let task_type_str = match &task.task_type {
                    TaskType::Combat(_) => "Combat",
                    TaskType::Exploration(_) => "Exploration",
                    TaskType::Gathering(_) => "Gathering",
                    TaskType::Auxiliary(_) => "Auxiliary",
                    TaskType::Investment(_) => "Investment",
                };

                // 检查并标记修炼路径任务
                let path_task_completed = disciple.cultivation.try_complete_path_task_by_type(task_type_str);

                println!(
                    "✅ {} 完成任务 [{}]",
                    disciple_name, task.name
                );
                println!(
                    "   获得: 修为+{}, 资源+{}, 声望+{}",
                    progress_gained, task.resource_reward, task.reputation_reward
                );

                if path_task_completed {
                    let (completed, total) = disciple.cultivation.cultivation_path
                        .as_ref()
                        .map(|p| p.progress())
                        .unwrap_or((0, 0));
                    println!("   🔮 修炼路径进度: {}/{}", completed, total);
                }

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
        let mut disciples_need_path = Vec::new();

        for disciple in self.sect.alive_disciples_mut() {
            // 检查修炼路径是否为空（刚进入新境界）
            if let Some(ref path) = disciple.cultivation.cultivation_path {
                if path.required.is_empty() {
                    disciples_need_path.push(disciple.id);
                }
            }

            if disciple.cultivation.can_tribulate() {
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

        // 为需要的弟子生成修炼路径
        for disciple_id in disciples_need_path {
            self.generate_cultivation_path_tasks(disciple_id);
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

    /// 检查并移除无效的守卫任务（妖魔已离开）
    fn check_and_remove_invalid_defense_tasks(&mut self) {
        let invalid_task_ids = self.map.check_defense_tasks_validity(&self.current_tasks);

        if !invalid_task_ids.is_empty() {
            // 收集需要解锁的任务信息（task_id, task_name, enemy_name）
            let invalid_tasks: Vec<(usize, String, Option<String>)> = self
                .current_tasks
                .iter()
                .filter(|t| invalid_task_ids.contains(&t.id))
                .map(|t| {
                    let enemy_name = if let crate::task::TaskType::Combat(combat_task) = &t.task_type {
                        Some(combat_task.enemy_name.clone())
                    } else {
                        None
                    };
                    (t.id, t.name.clone(), enemy_name)
                })
                .collect();

            // 移除无效任务
            self.current_tasks.retain(|t| !invalid_task_ids.contains(&t.id));
            self.task_assignments.retain(|a| !invalid_task_ids.contains(&a.task_id));

            // 清除弟子的current_task和解锁妖魔
            for (task_id, task_name, enemy_name_opt) in invalid_tasks {
                // 清除妖魔的任务关联和解锁移动
                self.map.clear_monster_task(task_id);
                if let Some(enemy_name) = enemy_name_opt {
                    self.map.unlock_monster_for_defense_task(&enemy_name);
                }
            }
        }
    }

    /// 检查游戏状态
    /// 移除过期任务
    fn remove_expired_tasks(&mut self) {
        let current_turn = self.sect.year;
        let expired_tasks: Vec<(usize, String, Option<String>)> = self
            .current_tasks
            .iter()
            .filter(|t| t.is_expired(current_turn))
            .map(|t| {
                let enemy_name = if let crate::task::TaskType::Combat(combat_task) = &t.task_type {
                    Some(combat_task.enemy_name.clone())
                } else {
                    None
                };
                (t.id, t.name.clone(), enemy_name)
            })
            .collect();

        if !expired_tasks.is_empty() {
            if !self.is_web_mode {
                UI::warning(&format!("⏰ {} 个任务已过期", expired_tasks.len()));
            }

            let expired_task_ids: Vec<usize> = expired_tasks.iter().map(|(id, _, _)| *id).collect();

            // 移除过期任务
            self.current_tasks
                .retain(|t| !expired_task_ids.contains(&t.id));
            self.task_assignments
                .retain(|a| !expired_task_ids.contains(&a.task_id));

            // 清除正在执行过期任务的弟子和解锁妖魔
            for (task_id, task_name, enemy_name_opt) in expired_tasks {
                // 清除妖魔的任务关联和解锁移动
                self.map.clear_monster_task(task_id);
                if task_name.contains("守卫") {
                    if let Some(enemy_name) = enemy_name_opt {
                        self.map.unlock_monster_for_defense_task(&enemy_name);
                    }
                }
            }
        }
    }

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

    /// 为弟子生成修炼路径任务
    /// 为弟子生成修炼路径（设置需要完成的任务类型和数量）
    pub fn generate_cultivation_path_tasks(&mut self, disciple_id: usize) {
        use crate::cultivation::CultivationLevel;

        // 找到弟子
        let disciple = if let Some(d) = self.sect.disciples.iter_mut().find(|d| d.id == disciple_id) {
            d
        } else {
            return;
        };

        let level = disciple.cultivation.current_level;

        // 根据境界决定任务配比（总共12个）
        let (combat, exploration, gathering, auxiliary) = match level {
            CultivationLevel::QiRefining => (2, 3, 4, 3),      // 练气：多采集
            CultivationLevel::Foundation => (4, 3, 2, 3),      // 筑基：多战斗
            CultivationLevel::GoldenCore => (5, 4, 1, 2),      // 结丹：战斗+探索
            CultivationLevel::NascentSoul => (6, 4, 0, 2),     // 凝婴：更多战斗
            CultivationLevel::SpiritSevering => (7, 4, 0, 1),  // 化神：主要战斗
            CultivationLevel::VoidRefinement => (8, 3, 0, 1),  // 练虚：几乎全战斗
            CultivationLevel::Ascension => (10, 2, 0, 0),      // 飞升：纯战斗
        };

        // 创建修炼路径要求
        let mut requirements = std::collections::HashMap::new();
        if combat > 0 {
            requirements.insert("Combat".to_string(), combat);
        }
        if exploration > 0 {
            requirements.insert("Exploration".to_string(), exploration);
        }
        if gathering > 0 {
            requirements.insert("Gathering".to_string(), gathering);
        }
        if auxiliary > 0 {
            requirements.insert("Auxiliary".to_string(), auxiliary);
        }

        // 设置修炼路径
        disciple.cultivation.cultivation_path =
            Some(crate::cultivation::CultivationPath::with_requirements(requirements));

        if !self.is_web_mode {
            UI::success(&format!(
                "✨ {} 获得了新的修炼路径（需完成{}个战斗、{}个探索、{}个采集、{}个辅助任务）！",
                disciple.name, combat, exploration, gathering, auxiliary
            ));
        }
    }
}
