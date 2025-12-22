use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use dashmap::DashMap;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::api_types::*;
use crate::interactive::InteractiveGame;

/// 全局游戏状态
pub struct GameStore {
    games: DashMap<String, Arc<tokio::sync::Mutex<InteractiveGame>>>,
}

impl GameStore {
    pub fn new() -> Self {
        Self {
            games: DashMap::new(),
        }
    }

    pub fn create_game(&self, sect_name: String) -> String {
        let game_id = Uuid::new_v4().to_string();
        let game = InteractiveGame::new_with_mode(sect_name, true); // Web模式
        self.games.insert(game_id.clone(), Arc::new(tokio::sync::Mutex::new(game)));
        game_id
    }

    pub fn get_game(&self, game_id: &str) -> Option<Arc<tokio::sync::Mutex<InteractiveGame>>> {
        self.games.get(game_id).map(|entry| entry.value().clone())
    }

    pub fn remove_game(&self, game_id: &str) {
        self.games.remove(game_id);
    }
}

pub type AppState = Arc<GameStore>;

/// 创建路由
pub fn create_router() -> Router {
    let store = Arc::new(GameStore::new());

    Router::new()
        // 版本信息
        .route("/api/version", get(get_version))

        // 游戏管理
        .route("/api/game/new", post(create_game))
        .route("/api/game/:game_id", get(get_game_info))

        // 回合管理
        .route("/api/game/:game_id/turn/start", post(start_turn))
        .route("/api/game/:game_id/turn/end", post(end_turn))

        // 弟子管理
        .route("/api/game/:game_id/disciples", get(get_disciples))
        .route("/api/game/:game_id/disciples/:disciple_id", get(get_disciple))
        .route("/api/game/:game_id/recruit", post(recruit_disciple))
        .route("/api/game/:game_id/disciples/:disciple_id/move", post(move_disciple))

        // 任务管理
        .route("/api/game/:game_id/tasks", get(get_tasks))
        .route("/api/game/:game_id/tasks/:task_id/assign", post(assign_task))
        .route("/api/game/:game_id/tasks/:task_id/assign", delete(unassign_task))
        .route("/api/game/:game_id/tasks/auto-assign", post(auto_assign_tasks))

        // 统计信息
        .route("/api/game/:game_id/statistics", get(get_statistics))

        // 地图
        .route("/api/game/:game_id/map", get(get_map))

        // 渡劫
        .route("/api/game/:game_id/tribulation/candidates", get(get_tribulation_candidates))
        .route("/api/game/:game_id/tribulation", post(execute_tribulation))

        // 丹药
        .route("/api/game/:game_id/pills", get(get_pill_inventory))
        .route("/api/game/:game_id/pills/use", post(use_pill))

        // 草药和炼丹
        .route("/api/game/:game_id/herbs", get(get_herb_inventory))
        .route("/api/game/:game_id/recipes", get(get_all_recipes))
        .route("/api/game/:game_id/refine", post(refine_pill))

        // 建筑
        .route("/api/game/:game_id/buildings", get(get_building_tree))
        .route("/api/game/:game_id/buildings/build", post(build_building))

        // 关系系统
        .route("/api/game/:game_id/disciples/:disciple_id/relationships", get(get_disciple_relationships))
        .route("/api/game/:game_id/relationships", get(get_all_relationships))
        .route("/api/game/:game_id/relationships/mentorship", post(set_mentorship))
        .route("/api/game/:game_id/relationships/dao-companion", post(set_dao_companion))
        .route("/api/game/:game_id/relationships/update", post(update_relationship))

        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(store)
}

/// 启动服务器
pub async fn start_server() {
    let app = create_router();

    let addr = "0.0.0.0:3000".parse().unwrap();

    println!("🚀 Server running on http://localhost:3000");
    println!("📚 API documentation: /api");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// ==================== API 处理器 ====================

/// 获取版本信息
async fn get_version() -> impl IntoResponse {
    let response = VersionResponse {
        api_version: crate::version::API_VERSION.to_string(),
        app_name: crate::version::APP_NAME.to_string(),
    };
    (StatusCode::OK, Json(ApiResponse::ok(response)))
}

/// 创建新游戏
async fn create_game(
    State(store): State<AppState>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    let game_id = store.create_game(req.sect_name.clone());

    if let Some(game) = store.get_game(&game_id) {
        let game = game.lock().await;
        let response = GameInfoResponse {
            game_id: game_id.clone(),
            sect: (&game.sect).into(),
            state: format!("{:?}", game.state),
        };
        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<GameInfoResponse>::error(
                "INTERNAL_ERROR".to_string(),
                "创建游戏失败".to_string(),
            )),
        )
    }
}

/// 获取游戏信息
async fn get_game_info(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game) = store.get_game(&game_id) {
        let game = game.lock().await;
        let response = GameInfoResponse {
            game_id: game_id.clone(),
            sect: (&game.sect).into(),
            state: format!("{:?}", game.state),
        };
        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<GameInfoResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 开始新回合
async fn start_turn(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 开始回合
        game.start_turn();

        // 收集事件（简化版）
        let events = vec![
            GameEventDto {
                event_type: "Income".to_string(),
                message: format!("年度收入"),
            },
        ];

        // 获取任务和弟子
        let current_turn = game.sect.year;
        let tasks: Vec<TaskDto> = game.current_tasks
            .iter()
            .map(|task| {
                let assignment = game.task_assignments.iter().find(|a| a.task_id == task.id);
                let progress = assignment.map(|a| a.progress).unwrap_or(0);
                let assigned_to = assignment.map(|a| a.disciple_ids.clone()).unwrap_or_default();
                let remaining_turns = if task.created_turn + task.expiry_turns > current_turn {
                    task.created_turn + task.expiry_turns - current_turn
                } else {
                    0
                };

                // 找出适合该任务的弟子
                let mut free_disciples = Vec::new();
                let mut busy_disciples = Vec::new();

                for disciple in &game.sect.disciples {
                    // 检查弟子是否适合该任务（技能和修为检查）
                    if task.is_suitable_for_disciple(disciple) {
                        // 检查弟子是否在任务位置（如果任务有位置要求）
                        let is_at_location = if let Some(task_pos) = &task.position {
                            disciple.position.x == task_pos.x && disciple.position.y == task_pos.y
                        } else {
                            true // 没有位置要求的任务，所有弟子都可以
                        };

                        if !is_at_location {
                            continue; // 弟子不在任务位置，跳过
                        }

                        // 检查弟子是否正在执行其他任务
                        let is_busy = game.task_assignments.iter().any(|a|
                            a.disciple_ids.contains(&disciple.id) && a.task_id != task.id
                        );

                        if is_busy {
                            busy_disciples.push(disciple.id);
                        } else {
                            free_disciples.push(disciple.id);
                        }
                    }
                }

                // 提取敌人信息（如果是战斗任务）
                let enemy_info = if let crate::task::TaskType::Combat(combat_task) = &task.task_type {
                    // 从enemy_name中提取ID和名称（格式："{名称}#{ID}"）
                    let enemy_name_full = &combat_task.enemy_name;
                    if let Some(pos) = enemy_name_full.rfind('#') {
                        let enemy_name = enemy_name_full[..pos].to_string();
                        let enemy_id = format!("monster_{}", &enemy_name_full[pos+1..]);
                        Some(EnemyInfo {
                            enemy_id,
                            enemy_name,
                            enemy_level: combat_task.enemy_level,
                        })
                    } else {
                        // 如果没有ID，只返回名称
                        Some(EnemyInfo {
                            enemy_id: "unknown".to_string(),
                            enemy_name: enemy_name_full.clone(),
                            enemy_level: combat_task.enemy_level,
                        })
                    }
                } else {
                    None
                };

                TaskDto {
                    id: task.id,
                    name: task.name.clone(),
                    task_type: format!("{:?}", task.task_type),
                    rewards: TaskRewards {
                        progress: task.progress_reward,
                        resources: task.resource_reward,
                        reputation: task.reputation_reward,
                    },
                    dao_heart_impact: task.dao_heart_impact,
                    assigned_to,
                    max_participants: task.max_participants,
                    duration: task.duration,
                    progress,
                    expiry_turns: task.expiry_turns,
                    created_turn: task.created_turn,
                    remaining_turns,
                    energy_cost: task.energy_cost,
                    constitution_cost: task.constitution_cost,
                    skill_required: task.get_skill_required(),
                    suitable_disciples: SuitableDisciples {
                        free: free_disciples,
                        busy: busy_disciples,
                    },
                    enemy_info,
                    position: task.position.as_ref().map(|p| PositionDto { x: p.x, y: p.y }),
                }
            })
            .collect();

        let disciples: Vec<DiscipleDto> = game.sect
            .alive_disciples()
            .iter()
            .map(|d| (*d).into())
            .collect();

        // 获取待招募弟子信息
        let pending_recruitment = game.pending_recruitment.as_ref().map(|d| d.into());

        let response = TurnStartResponse {
            year: game.sect.year,
            events,
            tasks,
            disciples,
            pending_recruitment,
        };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<TurnStartResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 结束回合
async fn end_turn(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(_req): Json<TurnEndRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 执行任务
        game.execute_turn();

        // 检查游戏状态
        let _is_running = game.check_game_state();

        let response = TurnEndResponse {
            results: vec![],
            game_state: format!("{:?}", game.state),
        };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<TurnEndResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取所有弟子
async fn get_disciples(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        let mut disciples: Vec<DiscipleDto> = game.sect
            .alive_disciples()
            .iter()
            .map(|d| (*d).into())
            .collect();

        // 填充当前任务信息
        for disciple_dto in &mut disciples {
            // 查找弟子的任务分配
            if let Some(assignment) = game.task_assignments.iter().find(|a| a.contains_disciple(disciple_dto.id)) {
                if let Some(task) = game.current_tasks.iter().find(|t| t.id == assignment.task_id) {
                    disciple_dto.current_task_info = Some(CurrentTaskInfo {
                        task_id: task.id,
                        task_name: task.name.clone(),
                        duration: task.duration,
                        progress: assignment.progress,
                    });
                }
            }
        }

        (StatusCode::OK, Json(ApiResponse::ok(disciples)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<DiscipleDto>>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取单个弟子
async fn get_disciple(
    State(store): State<AppState>,
    Path((game_id, disciple_id)): Path<(String, usize)>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        if let Some(disciple) = game.sect.disciples.iter().find(|d| d.id == disciple_id) {
            let dto: DiscipleDto = disciple.into();
            (StatusCode::OK, Json(ApiResponse::ok(dto)))
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<DiscipleDto>::error(
                    "DISCIPLE_NOT_FOUND".to_string(),
                    "弟子不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<DiscipleDto>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 招募弟子（接受或拒绝）
async fn recruit_disciple(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<RecruitDiscipleRequest>,
) -> impl IntoResponse {
    const RECRUITMENT_COST: u32 = 1000;

    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 检查是否有待招募的弟子
        if let Some(disciple) = game.pending_recruitment.take() {
            if req.accept {
                // 检查资源是否足够
                let resources_before = game.sect.resources;
                if resources_before < RECRUITMENT_COST {
                    // 资源不足，放回pending
                    game.pending_recruitment = Some(disciple);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<RecruitDiscipleResponse>::error(
                            "INSUFFICIENT_RESOURCES".to_string(),
                            format!("资源不足，需要{}资源", RECRUITMENT_COST),
                        )),
                    );
                }

                // 扣除资源
                game.sect.resources -= RECRUITMENT_COST;
                let resources_after = game.sect.resources;

                // 添加弟子
                let disciple_dto: DiscipleDto = (&disciple).into();
                game.sect.recruit_disciple(disciple);

                let response = RecruitDiscipleResponse {
                    success: true,
                    message: format!("成功招募弟子「{}」", disciple_dto.name),
                    disciple: Some(disciple_dto),
                    resources_before,
                    resources_after,
                    cost: RECRUITMENT_COST,
                };

                (StatusCode::OK, Json(ApiResponse::ok(response)))
            } else {
                // 用户拒绝招募
                let response = RecruitDiscipleResponse {
                    success: true,
                    message: "已拒绝招募".to_string(),
                    disciple: None,
                    resources_before: game.sect.resources,
                    resources_after: game.sect.resources,
                    cost: 0,
                };

                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
        } else {
            // 没有待招募的弟子
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<RecruitDiscipleResponse>::error(
                    "NO_PENDING_RECRUITMENT".to_string(),
                    "当前没有待招募的弟子".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<RecruitDiscipleResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 移动弟子
async fn move_disciple(
    State(store): State<AppState>,
    Path((game_id, disciple_id)): Path<(String, usize)>,
    Json(req): Json<MoveDiscipleRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 查找弟子并获取所需信息
        let disciple_info = game.sect.disciples.iter().find(|d| d.id == disciple_id).map(|d| {
            (
                d.position.x,
                d.position.y,
                d.name.clone(),
                d.cultivation.current_level.movement_range(),
                d.moves_remaining,
            )
        });

        if let Some((old_x, old_y, disciple_name, max_range, moves_remaining)) = disciple_info {
            let old_position = PositionDto { x: old_x, y: old_y };

            // 计算距离（曼哈顿距离）
            let distance = ((req.x as i32 - old_x as i32).abs()
                + (req.y as i32 - old_y as i32).abs()) as u32;

            // 检查移动距离是否在范围内
            if distance > max_range {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<MoveDiscipleResponse>::error(
                        "MOVEMENT_OUT_OF_RANGE".to_string(),
                        format!(
                            "移动距离({})超出范围！{}的最大移动距离为{}格",
                            distance, disciple_name, max_range
                        ),
                    )),
                );
            }

            // 检查本回合剩余移动距离
            if distance > moves_remaining {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<MoveDiscipleResponse>::error(
                        "INSUFFICIENT_MOVES".to_string(),
                        format!(
                            "本回合移动距离不足！需要{}格，剩余{}格",
                            distance, moves_remaining
                        ),
                    )),
                );
            }

            // 更新弟子位置和移动距离
            if let Some(disciple) = game.sect.disciples.iter_mut().find(|d| d.id == disciple_id) {
                disciple.moves_remaining -= distance;
                let new_position = crate::map::Position { x: req.x, y: req.y };
                disciple.move_to(new_position);
            }

            // 检查并采集草药
            let mut collected_herb: Option<CollectedHerbInfo> = None;
            let mut herb_to_collect: Option<(String, crate::map::HerbQuality)> = None;
            let mut herb_index_to_remove: Option<usize> = None;

            for (idx, positioned) in game.map.elements.iter().enumerate() {
                if positioned.position.x == req.x && positioned.position.y == req.y {
                    if let crate::map::MapElement::Herb(herb) = &positioned.element {
                        herb_to_collect = Some((herb.name.clone(), herb.quality));
                        herb_index_to_remove = Some(idx);
                        break;
                    }
                }
            }

            // 移除草药并添加到仓库
            if let (Some(idx), Some((herb_name, herb_quality))) = (herb_index_to_remove, herb_to_collect) {
                game.map.elements.remove(idx);
                game.sect.add_herb(&herb_name, herb_quality);
                collected_herb = Some(CollectedHerbInfo {
                    name: herb_name,
                    quality: herb_quality.name().to_string(),
                });
            }

            let new_position_dto = PositionDto { x: req.x, y: req.y };

            let message = if let Some(ref herb) = collected_herb {
                format!("{}已移动至({}, {})，采集了{}({})", disciple_name, req.x, req.y, herb.name, herb.quality)
            } else {
                format!("{}已移动至({}, {})", disciple_name, req.x, req.y)
            };

            let response = MoveDiscipleResponse {
                success: true,
                message,
                disciple_id,
                disciple_name,
                old_position,
                new_position: new_position_dto,
                collected_herb,
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<MoveDiscipleResponse>::error(
                    "DISCIPLE_NOT_FOUND".to_string(),
                    "弟子不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<MoveDiscipleResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取任务列表
async fn get_tasks(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;
        let current_turn = game.sect.year;

        let tasks: Vec<TaskDto> = game.current_tasks
            .iter()
            .map(|task| {
                let assignment = game.task_assignments.iter().find(|a| a.task_id == task.id);
                let progress = assignment.map(|a| a.progress).unwrap_or(0);
                let assigned_to = assignment.map(|a| a.disciple_ids.clone()).unwrap_or_default();
                let remaining_turns = if task.created_turn + task.expiry_turns > current_turn {
                    task.created_turn + task.expiry_turns - current_turn
                } else {
                    0
                };

                // 找出适合该任务的弟子
                let mut free_disciples = Vec::new();
                let mut busy_disciples = Vec::new();

                for disciple in &game.sect.disciples {
                    // 检查弟子是否适合该任务（技能和修为检查）
                    if task.is_suitable_for_disciple(disciple) {
                        // 检查弟子是否在任务位置（如果任务有位置要求）
                        let is_at_location = if let Some(task_pos) = &task.position {
                            disciple.position.x == task_pos.x && disciple.position.y == task_pos.y
                        } else {
                            true // 没有位置要求的任务，所有弟子都可以
                        };

                        if !is_at_location {
                            continue; // 弟子不在任务位置，跳过
                        }

                        // 检查弟子是否正在执行其他任务
                        let is_busy = game.task_assignments.iter().any(|a|
                            a.disciple_ids.contains(&disciple.id) && a.task_id != task.id
                        );

                        if is_busy {
                            busy_disciples.push(disciple.id);
                        } else {
                            free_disciples.push(disciple.id);
                        }
                    }
                }

                // 提取敌人信息（如果是战斗任务）
                let enemy_info = if let crate::task::TaskType::Combat(combat_task) = &task.task_type {
                    // 从enemy_name中提取ID和名称（格式："{名称}#{ID}"）
                    let enemy_name_full = &combat_task.enemy_name;
                    if let Some(pos) = enemy_name_full.rfind('#') {
                        let enemy_name = enemy_name_full[..pos].to_string();
                        let enemy_id = format!("monster_{}", &enemy_name_full[pos+1..]);
                        Some(EnemyInfo {
                            enemy_id,
                            enemy_name,
                            enemy_level: combat_task.enemy_level,
                        })
                    } else {
                        // 如果没有ID，只返回名称
                        Some(EnemyInfo {
                            enemy_id: "unknown".to_string(),
                            enemy_name: enemy_name_full.clone(),
                            enemy_level: combat_task.enemy_level,
                        })
                    }
                } else {
                    None
                };

                TaskDto {
                    id: task.id,
                    name: task.name.clone(),
                    task_type: format!("{:?}", task.task_type),
                    rewards: TaskRewards {
                        progress: task.progress_reward,
                        resources: task.resource_reward,
                        reputation: task.reputation_reward,
                    },
                    dao_heart_impact: task.dao_heart_impact,
                    assigned_to,
                    max_participants: task.max_participants,
                    duration: task.duration,
                    progress,
                    expiry_turns: task.expiry_turns,
                    created_turn: task.created_turn,
                    remaining_turns,
                    energy_cost: task.energy_cost,
                    constitution_cost: task.constitution_cost,
                    skill_required: task.get_skill_required(),
                    suitable_disciples: SuitableDisciples {
                        free: free_disciples,
                        busy: busy_disciples,
                    },
                    enemy_info,
                    position: task.position.as_ref().map(|p| PositionDto { x: p.x, y: p.y }),
                }
            })
            .collect();

        (StatusCode::OK, Json(ApiResponse::ok(tasks)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<TaskDto>>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 分配任务
async fn assign_task(
    State(store): State<AppState>,
    Path((game_id, task_id)): Path<(String, usize)>,
    Json(req): Json<AssignTaskRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 检查任务是否存在
        if let Some(task) = game.current_tasks.iter().find(|t| t.id == task_id) {
            // 检查弟子是否存在
            if let Some(disciple) = game.sect.disciples.iter().find(|d| d.id == req.disciple_id) {
                // 检查弟子是否适合该任务
                if !task.is_suitable_for_disciple(disciple) {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<AssignTaskResponse>::error(
                            "DISCIPLE_NOT_SUITABLE".to_string(),
                            format!("弟子 {} 不适合该任务（可能缺少所需技能或修为不足）", disciple.name),
                        )),
                    );
                }

                // 检查弟子是否在任务位置
                if let Some(task_pos) = &task.position {
                    if disciple.position.x != task_pos.x || disciple.position.y != task_pos.y {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ApiResponse::<AssignTaskResponse>::error(
                                "DISCIPLE_NOT_AT_LOCATION".to_string(),
                                format!("弟子 {} 不在任务位置({}, {})，当前位置({}, {})",
                                    disciple.name, task_pos.x, task_pos.y,
                                    disciple.position.x, disciple.position.y),
                            )),
                        );
                    }
                }

                // 检查任务是否已满
                let max_participants = task.max_participants;
                let current_count = game.task_assignments.iter()
                    .find(|a| a.task_id == task_id)
                    .map(|a| a.disciple_ids.len())
                    .unwrap_or(0);

                if current_count >= max_participants as usize {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<AssignTaskResponse>::error(
                            "TASK_FULL".to_string(),
                            format!("任务已满，最多允许{}人参与", max_participants),
                        )),
                    );
                }

                // 检查弟子是否已经在其他任务中
                let already_in_other_task = game.task_assignments.iter()
                    .any(|a| a.task_id != task_id && a.disciple_ids.contains(&req.disciple_id));

                if already_in_other_task {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<AssignTaskResponse>::error(
                            "DISCIPLE_BUSY".to_string(),
                            format!("弟子 {} 已在执行其他任务", disciple.name),
                        )),
                    );
                }

                // 克隆守卫任务相关信息以避免借用冲突
                let enemy_name_opt = if task.name.contains("守卫") {
                    if let crate::task::TaskType::Combat(combat_task) = &task.task_type {
                        Some(combat_task.enemy_name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // 在 task_assignments 中找到对应的分配记录
                if let Some(assignment) = game.task_assignments.iter_mut().find(|a| a.task_id == task_id) {
                    assignment.add_disciple(req.disciple_id);
                    let current_count = assignment.disciple_ids.len();

                    // 如果是守卫任务，锁定妖魔的移动
                    if let Some(enemy_name) = enemy_name_opt {
                        game.map.lock_monster_for_defense_task(&enemy_name);
                    }

                    let response = AssignTaskResponse {
                        task_id,
                        disciple_id: req.disciple_id,
                        message: format!("任务分配成功 ({}/{}人)", current_count, max_participants),
                    };

                    (StatusCode::OK, Json(ApiResponse::ok(response)))
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignTaskResponse>::error(
                            "ASSIGNMENT_NOT_FOUND".to_string(),
                            "任务分配记录不存在".to_string(),
                        )),
                    )
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<AssignTaskResponse>::error(
                        "DISCIPLE_NOT_FOUND".to_string(),
                        "弟子不存在".to_string(),
                    )),
                )
            }
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<AssignTaskResponse>::error(
                    "TASK_NOT_FOUND".to_string(),
                    "任务不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignTaskResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 取消任务分配
async fn unassign_task(
    State(store): State<AppState>,
    Path((game_id, task_id)): Path<(String, usize)>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 检查任务是否存在
        if let Some(task) = game.current_tasks.iter().find(|t| t.id == task_id) {
            // 克隆守卫任务相关信息以避免借用冲突
            let enemy_name_opt = if task.name.contains("守卫") {
                if let crate::task::TaskType::Combat(combat_task) = &task.task_type {
                    Some(combat_task.enemy_name.clone())
                } else {
                    None
                }
            } else {
                None
            };

            // 在 task_assignments 中找到对应的分配记录
            if let Some(assignment) = game.task_assignments.iter_mut().find(|a| a.task_id == task_id) {
                let removed_count = assignment.disciple_ids.len();
                assignment.disciple_ids.clear();

                // 如果是守卫任务，解锁妖魔的移动
                if let Some(enemy_name) = enemy_name_opt {
                    game.map.unlock_monster_for_defense_task(&enemy_name);
                }

                (StatusCode::OK, Json(ApiResponse::ok(format!("取消成功，移除了{}名弟子", removed_count))))
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<String>::error(
                        "ASSIGNMENT_NOT_FOUND".to_string(),
                        "任务分配记录不存在".to_string(),
                    )),
                )
            }
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<String>::error(
                    "TASK_NOT_FOUND".to_string(),
                    "任务不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<String>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 自动分配任务
async fn auto_assign_tasks(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        game.auto_assign_remaining();

        (StatusCode::OK, Json(ApiResponse::ok("自动分配完成".to_string())))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<String>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取统计信息
async fn get_statistics(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;
        let stats = game.sect.get_statistics();

        let response = StatisticsResponse {
            year: stats.year,
            total_disciples: stats.total_disciples,
            disciples_by_type: DisciplesByType {
                outer: stats.outer_disciples,
                inner: stats.inner_disciples,
                personal: stats.personal_disciples,
            },
            resources: stats.resources,
            reputation: stats.reputation,
            cultivation_distribution: std::collections::HashMap::new(),
        };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<StatisticsResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取渡劫候选人
async fn get_tribulation_candidates(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        let candidates: Vec<TribulationCandidateDto> = game.sect
            .alive_disciples()
            .iter()
            .filter(|d| d.cultivation.can_tribulate())
            .map(|d| TribulationCandidateDto {
                disciple_id: d.id,
                name: d.name.clone(),
                current_level: format!("{:?}", d.cultivation.current_level),
                success_rate: d.tribulation_success_rate(),
                dao_heart: d.dao_heart,
                heritage_bonus: d.heritage.as_ref().map(|h| h.tribulation_bonus).unwrap_or(0.0),
            })
            .collect();

        let response = TribulationCandidatesResponse { candidates };
        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<TribulationCandidatesResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 执行渡劫
async fn execute_tribulation(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<TribulationRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        if let Some(disciple) = game.sect.disciples.iter_mut().find(|d| d.id == req.disciple_id) {
            let name = disciple.name.clone();
            let success = disciple.attempt_tribulation();

            let response = if success {
                TribulationResponse {
                    success: true,
                    disciple_id: req.disciple_id,
                    name: name.clone(),
                    new_level: Some(format!("{:?}", disciple.cultivation.current_level)),
                    message: format!("{}渡劫成功！", name),
                }
            } else {
                TribulationResponse {
                    success: false,
                    disciple_id: req.disciple_id,
                    name,
                    new_level: None,
                    message: "渡劫失败".to_string(),
                }
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<TribulationResponse>::error(
                    "DISCIPLE_NOT_FOUND".to_string(),
                    "弟子不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<TribulationResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取地图数据
async fn get_map(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        use crate::map::MapElement;
        use std::collections::HashMap;

        // 第一步：收集所有妖魔的入侵信息
        let mut attacks: HashMap<String, AttackInfo> = HashMap::new();
        for positioned in &game.map.elements {
            if let MapElement::Monster(monster) = &positioned.element {
                if let Some(ref invaded_location_id) = monster.invaded_location_id {
                    attacks.insert(
                        invaded_location_id.clone(),
                        AttackInfo {
                            attacker_name: format!("{}#{}", monster.name, monster.id),
                            attacker_level: monster.level,
                            is_demon: monster.is_demon,
                        },
                    );
                }
            }
        }

        // 第二步：遍历所有元素，为被攻击的元素添加攻击信息
        let elements: Vec<MapElementDto> = game.map.elements
            .iter()
            .map(|positioned| {
                let location_id = positioned.element.get_location_id();
                let under_attack = attacks.get(&location_id).cloned();

                let (element_type, name, details) = match &positioned.element {
                    MapElement::Village(v) => (
                        "Village".to_string(),
                        v.name.clone(),
                        MapElementDetails::Village {
                            population: v.population,
                            prosperity: v.prosperity,
                            under_attack,
                        },
                    ),
                    MapElement::Faction(f) => (
                        "Faction".to_string(),
                        f.name.clone(),
                        MapElementDetails::Faction {
                            power_level: f.power_level,
                            relationship: f.relationship,
                            under_attack,
                        },
                    ),
                    MapElement::DangerousLocation(d) => (
                        "DangerousLocation".to_string(),
                        d.name.clone(),
                        MapElementDetails::DangerousLocation {
                            danger_level: d.danger_level,
                        },
                    ),
                    MapElement::SecretRealm(s) => (
                        "SecretRealm".to_string(),
                        s.name.clone(),
                        MapElementDetails::SecretRealm {
                            realm_type: format!("{:?}", s.realm_type),
                            difficulty: s.difficulty,
                            under_attack,
                        },
                    ),
                    MapElement::Monster(m) => (
                        "Monster".to_string(),
                        m.name.clone(),
                        MapElementDetails::Monster {
                            monster_id: format!("monster_{}", m.id),
                            level: m.level,
                            is_demon: m.is_demon,
                            growth_rate: m.growth_rate,
                            invading_location: m.invaded_location_id.clone(),
                        },
                    ),
                    MapElement::Terrain(t) => (
                        "Terrain".to_string(),
                        t.name.clone(),
                        MapElementDetails::Terrain {
                            terrain_type: format!("{:?}", t.terrain_type),
                        },
                    ),
                    MapElement::Herb(h) => (
                        "Herb".to_string(),
                        h.name.clone(),
                        MapElementDetails::Herb {
                            herb_id: format!("herb_{}", h.id),
                            quality: h.quality.name().to_string(),
                            growth_stage: h.growth_stage,
                            max_growth: h.max_growth,
                            is_mature: h.is_mature(),
                        },
                    ),
                };

                MapElementDto {
                    element_type,
                    name,
                    position: PositionDto {
                        x: positioned.position.x,
                        y: positioned.position.y,
                    },
                    details,
                }
            })
            .collect();

        let response = MapDataResponse {
            width: game.map.width,
            height: game.map.height,
            elements,
        };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<MapDataResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取丹药库存
async fn get_pill_inventory(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        let mut pills = std::collections::HashMap::new();

        use crate::pill::PillType;

        for pill_type in [
            PillType::QiRecovery,
            PillType::BodyStrength,
            PillType::VitalityElixir,
            PillType::CultivationBoost,
        ] {
            let effects = pill_type.effects();
            pills.insert(
                pill_type.to_string().to_string(),
                PillInfo {
                    count: game.sect.pill_inventory.get_count(pill_type),
                    name: pill_type.name().to_string(),
                    description: pill_type.description().to_string(),
                    energy_restore: effects.energy_restore,
                    constitution_restore: effects.constitution_restore,
                    cultivation_boost: effects.cultivation_boost,
                },
            );
        }

        let response = PillInventoryResponse { pills };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<PillInventoryResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 给弟子服用丹药
async fn use_pill(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<UsePillRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        use crate::pill::PillType;

        // 解析丹药类型
        let pill_type = match PillType::from_str(&req.pill_type) {
            Some(pt) => pt,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<UsePillResponse>::error(
                        "INVALID_PILL_TYPE".to_string(),
                        "无效的丹药类型".to_string(),
                    )),
                );
            }
        };

        // 检查库存
        if game.sect.pill_inventory.get_count(pill_type) == 0 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UsePillResponse>::error(
                    "NO_PILLS".to_string(),
                    format!("{}库存不足", pill_type.name()),
                )),
            );
        }

        // 查找弟子
        let disciple_index = game.sect.disciples.iter().position(|d| d.id == req.disciple_id);

        if let Some(index) = disciple_index {
            // 消耗丹药
            if !game.sect.pill_inventory.consume(pill_type) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<UsePillResponse>::error(
                        "NO_PILLS".to_string(),
                        format!("{}库存不足", pill_type.name()),
                    )),
                );
            }

            let disciple = &mut game.sect.disciples[index];
            let name = disciple.name.clone();
            let energy_before = disciple.energy;
            let constitution_before = disciple.constitution;
            let progress_before = disciple.cultivation.progress;

            // 应用效果
            let effects = pill_type.effects();
            disciple.restore_energy(effects.energy_restore);
            disciple.restore_constitution(effects.constitution_restore);

            // 应用修为进度加成
            if effects.cultivation_boost > 0 {
                disciple.cultivation.add_progress(effects.cultivation_boost);
            }

            let response = UsePillResponse {
                success: true,
                message: format!("{}服用了{}", name, pill_type.name()),
                disciple_name: name,
                energy_before,
                energy_after: disciple.energy,
                constitution_before,
                constitution_after: disciple.constitution,
                progress_before,
                progress_after: disciple.cultivation.progress,
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<UsePillResponse>::error(
                    "DISCIPLE_NOT_FOUND".to_string(),
                    "弟子不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<UsePillResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取草药仓库
async fn get_herb_inventory(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        let herbs_list = game.sect.herb_inventory.get_all();
        let total_count = game.sect.herb_inventory.total_count();

        let herbs: Vec<HerbEntryDto> = herbs_list
            .iter()
            .map(|h| HerbEntryDto {
                name: h.name.clone(),
                quality: h.quality.name().to_string(),
                count: h.count,
            })
            .collect();

        let response = HerbInventoryResponse { total_count, herbs };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<HerbInventoryResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取所有炼丹配方
async fn get_all_recipes(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        use crate::pill::{PillRecipe, PillType};

        let all_recipes = PillRecipe::all_recipes();
        let mut recipes: Vec<PillRecipeDto> = Vec::new();

        for recipe in all_recipes {
            // 检查是否可以炼制
            let herb_count = game.sect.herb_inventory.count_by_quality(recipe.required_herb_quality);
            let has_enough_herbs = herb_count >= recipe.required_herb_count;
            let has_enough_resources = game.sect.resources >= recipe.resource_cost;

            let (can_craft, reason) = if !has_enough_herbs {
                (false, Some(format!("需要{}个{}品质草药，当前{}个",
                    recipe.required_herb_count,
                    recipe.required_herb_quality.name(),
                    herb_count)))
            } else if !has_enough_resources {
                (false, Some(format!("需要{}资源，当前{}资源",
                    recipe.resource_cost,
                    game.sect.resources)))
            } else {
                (true, None)
            };

            recipes.push(PillRecipeDto {
                pill_type: recipe.pill_type.to_string().to_string(),
                name: recipe.pill_type.name().to_string(),
                description: recipe.pill_type.description().to_string(),
                required_herb_quality: recipe.required_herb_quality.name().to_string(),
                required_herb_count: recipe.required_herb_count,
                resource_cost: recipe.resource_cost,
                success_rate: recipe.success_rate,
                output_count: recipe.output_count,
                can_craft,
                reason,
            });
        }

        let response = AllRecipesResponse { recipes };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AllRecipesResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 炼制丹药
async fn refine_pill(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<RefinePillRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        use crate::pill::PillType;

        // 解析丹药类型
        let pill_type = match PillType::from_str(&req.pill_type) {
            Some(pt) => pt,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<RefinePillResponse>::error(
                        "INVALID_PILL_TYPE".to_string(),
                        "无效的丹药类型".to_string(),
                    )),
                );
            }
        };

        // 尝试炼制
        match game.sect.refine_pill(pill_type) {
            Ok(count) => {
                let response = RefinePillResponse {
                    success: true,
                    message: format!("成功炼制{}个{}", count, pill_type.name()),
                    pill_name: Some(pill_type.name().to_string()),
                    output_count: Some(count),
                };
                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
            Err(msg) => {
                let response = RefinePillResponse {
                    success: false,
                    message: msg.clone(),
                    pill_name: None,
                    output_count: None,
                };
                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<RefinePillResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// GET /api/game/:game_id/buildings - 获取建筑树信息
async fn get_building_tree(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        if let Some(ref tree) = game.sect.building_tree {
            // 转换所有建筑为DTO
            let buildings: Vec<BuildingDto> = tree.buildings.values().map(|b| {
                let actual_cost = tree.calculate_build_cost(&b.id).unwrap_or(0);
                let can_build = tree.can_build(&b.id).is_ok();

                // 生成效果描述（包含具体数值）
                let effects: Vec<String> = b.conditional_modifiers.iter().map(|cm| {
                    use crate::modifier::{ModifierTarget, ModifierApplication};

                    let target_name = match &cm.modifier.target {
                        ModifierTarget::DaoHeart => "道心",
                        ModifierTarget::Energy => "精力",
                        ModifierTarget::Constitution => "体魄",
                        ModifierTarget::TalentBonus(_) => "天赋加成",
                        ModifierTarget::TribulationSuccessRate => "渡劫成功率",
                        ModifierTarget::TaskReward => "任务奖励",
                        ModifierTarget::TaskSuitability => "任务适配度",
                        ModifierTarget::TaskDifficulty => "任务难度",
                        ModifierTarget::Income => "收入",
                        ModifierTarget::EnergyConsumption => "精力消耗",
                        ModifierTarget::ConstitutionConsumption => "体魄消耗",
                        ModifierTarget::CultivationSpeed => "修炼速度",
                    };

                    let value_str = match &cm.modifier.application {
                        ModifierApplication::Additive(v) => {
                            if *v >= 0.0 {
                                format!("+{}", v)
                            } else {
                                format!("{}", v)
                            }
                        },
                        ModifierApplication::Multiplicative(v) => {
                            let percent = (v * 100.0) as i32;
                            if percent >= 0 {
                                format!("+{}%", percent)
                            } else {
                                format!("{}%", percent)
                            }
                        },
                        ModifierApplication::Override(v) => format!("={}", v),
                    };

                    format!("{} {}", target_name, value_str)
                }).collect();

                BuildingDto {
                    id: b.id.clone(),
                    name: b.name.clone(),
                    description: b.description.clone(),
                    base_cost: b.base_cost,
                    actual_cost,
                    parent_id: b.parent_id.clone(),
                    is_built: b.is_built,
                    can_build,
                    effects,
                }
            }).collect();

            let response = BuildingTreeResponse {
                total_buildings: tree.get_total_count(),
                built_count: tree.get_built_count(),
                buildings_built_count: tree.buildings_built_count,
                cost_multiplier: 2_u32.pow(tree.buildings_built_count),
                available_resources: game.sect.resources,
                buildings,
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<BuildingTreeResponse>::error(
                    "NO_BUILDING_TREE".to_string(),
                    "该宗门尚未初始化建筑树".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<BuildingTreeResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// POST /api/game/:game_id/buildings/build - 建造建筑
async fn build_building(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<BuildBuildingRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 检查建筑树是否存在
        if game.sect.building_tree.is_none() {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<BuildBuildingResponse>::error(
                    "NO_BUILDING_TREE".to_string(),
                    "该宗门尚未初始化建筑树".to_string(),
                )),
            );
        }

        // 获取建筑名称和成本（用于响应）
        let building_name = game.sect.building_tree.as_ref()
            .and_then(|tree| tree.buildings.get(&req.building_id))
            .map(|b| b.name.clone())
            .unwrap_or_else(|| req.building_id.clone());

        let cost = match game.sect.building_tree.as_ref()
            .and_then(|tree| tree.calculate_build_cost(&req.building_id).ok()) {
            Some(c) => c,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<BuildBuildingResponse>::error(
                        "INVALID_BUILDING".to_string(),
                        "无效的建筑ID".to_string(),
                    )),
                );
            }
        };

        let resources_before = game.sect.resources;

        // 尝试建造
        match game.sect.build_building(&req.building_id) {
            Ok(message) => {
                // 获取建筑提供的效果数量
                let effects_count = game.sect.building_tree.as_ref()
                    .and_then(|tree| tree.buildings.get(&req.building_id))
                    .map(|b| b.conditional_modifiers.len())
                    .unwrap_or(0);

                let response = BuildBuildingResponse {
                    success: true,
                    message,
                    building_name,
                    cost,
                    resources_before,
                    resources_after: game.sect.resources,
                    effects_count,
                };

                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
            Err(err) => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<BuildBuildingResponse>::error(
                        "BUILD_FAILED".to_string(),
                        err,
                    )),
                )
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<BuildBuildingResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

// ==================== 关系系统 API ====================

/// 获取弟子的所有关系
async fn get_disciple_relationships(
    State(store): State<AppState>,
    Path((game_id, disciple_id)): Path<(String, usize)>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        if let Some(disciple) = game.sect.disciples.iter().find(|d| d.id == disciple_id) {
            let relationships: Vec<RelationshipDto> = disciple.relationships.iter()
                .filter_map(|rel| {
                    let target = game.sect.disciples.iter().find(|d| d.id == rel.target_id)?;
                    Some(RelationshipDto {
                        target_id: rel.target_id,
                        target_name: target.name.clone(),
                        scores: (&rel.scores).into(),
                        established_year: rel.established_year,
                        is_dao_companion: rel.is_dao_companion,
                        is_master: rel.is_master,
                        is_disciple: rel.is_disciple,
                        primary_relation: rel.get_primary_relation().to_string(),
                        highest_level: rel.scores.highest_level().name().to_string(),
                    })
                })
                .collect();

            let response = DiscipleRelationshipsResponse {
                disciple_id,
                disciple_name: disciple.name.clone(),
                relationships,
            };

            (StatusCode::OK, Json(ApiResponse::ok(response)))
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<DiscipleRelationshipsResponse>::error(
                    "DISCIPLE_NOT_FOUND".to_string(),
                    "弟子不存在".to_string(),
                )),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<DiscipleRelationshipsResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 获取所有关系
async fn get_all_relationships(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let game = game_mutex.lock().await;

        let mut all_relationships = Vec::new();

        for disciple in &game.sect.disciples {
            for rel in &disciple.relationships {
                if let Some(target) = game.sect.disciples.iter().find(|d| d.id == rel.target_id) {
                    all_relationships.push(RelationshipPairDto {
                        from_id: disciple.id,
                        from_name: disciple.name.clone(),
                        to_id: rel.target_id,
                        to_name: target.name.clone(),
                        scores: (&rel.scores).into(),
                        primary_relation: rel.get_primary_relation().to_string(),
                    });
                }
            }
        }

        let response = AllRelationshipsResponse {
            total_relationships: all_relationships.len(),
            relationships: all_relationships,
        };

        (StatusCode::OK, Json(ApiResponse::ok(response)))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AllRelationshipsResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 设置师徒关系
async fn set_mentorship(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<SetMentorshipRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 获取名称用于响应
        let master_name = game.sect.disciples.iter()
            .find(|d| d.id == req.master_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "未知".to_string());
        let disciple_name = game.sect.disciples.iter()
            .find(|d| d.id == req.disciple_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "未知".to_string());

        match game.sect.set_mentorship(req.master_id, req.disciple_id) {
            Ok(()) => {
                let response = SetMentorshipResponse {
                    success: true,
                    message: format!("{} 正式拜 {} 为师", disciple_name, master_name),
                    master_name,
                    disciple_name,
                };
                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
            Err(err) => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<SetMentorshipResponse>::error(
                        "MENTORSHIP_FAILED".to_string(),
                        err,
                    )),
                )
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<SetMentorshipResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 设置道侣关系
async fn set_dao_companion(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<SetDaoCompanionRequest>,
) -> impl IntoResponse {
    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 获取名称用于响应
        let disciple1_name = game.sect.disciples.iter()
            .find(|d| d.id == req.disciple1_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "未知".to_string());
        let disciple2_name = game.sect.disciples.iter()
            .find(|d| d.id == req.disciple2_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "未知".to_string());

        match game.sect.set_dao_companion(req.disciple1_id, req.disciple2_id) {
            Ok(()) => {
                let response = SetDaoCompanionResponse {
                    success: true,
                    message: format!("{} 与 {} 结为道侣", disciple1_name, disciple2_name),
                    disciple1_name,
                    disciple2_name,
                };
                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
            Err(err) => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<SetDaoCompanionResponse>::error(
                        "DAO_COMPANION_FAILED".to_string(),
                        err,
                    )),
                )
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<SetDaoCompanionResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}

/// 更新关系分数
async fn update_relationship(
    State(store): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<UpdateRelationshipRequest>,
) -> impl IntoResponse {
    use crate::relationship::RelationDimension;

    if let Some(game_mutex) = store.get_game(&game_id) {
        let mut game = game_mutex.lock().await;

        // 解析维度
        let dimension = match req.dimension.as_str() {
            "Romance" | "romance" => RelationDimension::Romance,
            "Mentorship" | "mentorship" => RelationDimension::Mentorship,
            "Comrade" | "comrade" => RelationDimension::Comrade,
            "Understanding" | "understanding" => RelationDimension::Understanding,
            "FatefulBond" | "fateful_bond" => RelationDimension::FatefulBond,
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<UpdateRelationshipResponse>::error(
                        "INVALID_DIMENSION".to_string(),
                        format!("无效的关系维度: {}", req.dimension),
                    )),
                );
            }
        };

        // 获取名称用于响应
        let from_name = game.sect.disciples.iter()
            .find(|d| d.id == req.from_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "未知".to_string());
        let to_name = game.sect.disciples.iter()
            .find(|d| d.id == req.to_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "未知".to_string());

        // 获取旧分数
        let old_score = game.sect.disciples.iter()
            .find(|d| d.id == req.from_id)
            .and_then(|d| d.get_relationship(req.to_id))
            .map(|rel| rel.scores.get(dimension))
            .unwrap_or(0);

        match game.sect.update_relationship_score(req.from_id, req.to_id, dimension, req.delta) {
            Ok(level_up) => {
                // 获取新分数
                let new_score = game.sect.disciples.iter()
                    .find(|d| d.id == req.from_id)
                    .and_then(|d| d.get_relationship(req.to_id))
                    .map(|rel| rel.scores.get(dimension))
                    .unwrap_or(0);

                let response = UpdateRelationshipResponse {
                    success: true,
                    message: format!("{} 对 {} 的{}关系变化: {} -> {}",
                        from_name, to_name, dimension.name(), old_score, new_score),
                    from_name,
                    to_name,
                    dimension: req.dimension,
                    old_score,
                    new_score,
                    level_up: level_up.map(|l| l.name().to_string()),
                };
                (StatusCode::OK, Json(ApiResponse::ok(response)))
            }
            Err(err) => {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<UpdateRelationshipResponse>::error(
                        "UPDATE_FAILED".to_string(),
                        err,
                    )),
                )
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<UpdateRelationshipResponse>::error(
                "GAME_NOT_FOUND".to_string(),
                "游戏不存在".to_string(),
            )),
        )
    }
}
