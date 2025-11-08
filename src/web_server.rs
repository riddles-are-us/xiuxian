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
                let assigned_to = assignment.and_then(|a| a.disciple_id);
                let remaining_turns = if task.created_turn + task.expiry_turns > current_turn {
                    task.created_turn + task.expiry_turns - current_turn
                } else {
                    0
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
                    duration: task.duration,
                    progress,
                    expiry_turns: task.expiry_turns,
                    created_turn: task.created_turn,
                    remaining_turns,
                    energy_cost: task.energy_cost,
                    constitution_cost: task.constitution_cost,
                }
            })
            .collect();

        let disciples: Vec<DiscipleDto> = game.sect
            .alive_disciples()
            .iter()
            .map(|d| (*d).into())
            .collect();

        let response = TurnStartResponse {
            year: game.sect.year,
            events,
            tasks,
            disciples,
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
            if let Some(assignment) = game.task_assignments.iter().find(|a| a.disciple_id == Some(disciple_dto.id)) {
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
                let assigned_to = assignment.and_then(|a| a.disciple_id);
                let remaining_turns = if task.created_turn + task.expiry_turns > current_turn {
                    task.created_turn + task.expiry_turns - current_turn
                } else {
                    0
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
                    duration: task.duration,
                    progress,
                    expiry_turns: task.expiry_turns,
                    created_turn: task.created_turn,
                    remaining_turns,
                    energy_cost: task.energy_cost,
                    constitution_cost: task.constitution_cost,
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
        if game.current_tasks.iter().any(|t| t.id == task_id) {
            // 检查弟子是否存在
            if !game.sect.alive_disciples().iter().any(|d| d.id == req.disciple_id) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<AssignTaskResponse>::error(
                        "DISCIPLE_NOT_FOUND".to_string(),
                        "弟子不存在".to_string(),
                    )),
                );
            }

            // 检查弟子是否已经被分配到其他任务
            let is_busy = game.task_assignments.iter().any(|a| {
                a.disciple_id == Some(req.disciple_id) && a.task_id != task_id
            });

            if is_busy {
                return (
                    StatusCode::CONFLICT,
                    Json(ApiResponse::<AssignTaskResponse>::error(
                        "DISCIPLE_BUSY".to_string(),
                        "该弟子已被分配到其他任务".to_string(),
                    )),
                );
            }

            // 在 task_assignments 中找到对应的分配记录
            if let Some(assignment) = game.task_assignments.iter_mut().find(|a| a.task_id == task_id) {
                assignment.disciple_id = Some(req.disciple_id);

                let response = AssignTaskResponse {
                    task_id,
                    disciple_id: req.disciple_id,
                    message: "任务分配成功".to_string(),
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
        if game.current_tasks.iter().any(|t| t.id == task_id) {
            // 在 task_assignments 中找到对应的分配记录
            if let Some(assignment) = game.task_assignments.iter_mut().find(|a| a.task_id == task_id) {
                assignment.disciple_id = None;
                (StatusCode::OK, Json(ApiResponse::ok("取消成功".to_string())))
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

        let elements: Vec<MapElementDto> = game.map.elements
            .iter()
            .map(|positioned| {
                let (element_type, name, details) = match &positioned.element {
                    MapElement::Village(v) => (
                        "Village".to_string(),
                        v.name.clone(),
                        MapElementDetails::Village {
                            population: v.population,
                            prosperity: v.prosperity,
                        },
                    ),
                    MapElement::Faction(f) => (
                        "Faction".to_string(),
                        f.name.clone(),
                        MapElementDetails::Faction {
                            power_level: f.power_level,
                            relationship: f.relationship,
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
                        },
                    ),
                    MapElement::Monster(m) => (
                        "Monster".to_string(),
                        m.name.clone(),
                        MapElementDetails::Monster {
                            level: m.level,
                            is_demon: m.is_demon,
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

            // 应用效果
            let effects = pill_type.effects();
            disciple.restore_energy(effects.energy_restore);
            disciple.restore_constitution(effects.constitution_restore);

            let response = UsePillResponse {
                success: true,
                message: format!("{}服用了{}", name, pill_type.name()),
                disciple_name: name,
                energy_before,
                energy_after: disciple.energy,
                constitution_before,
                constitution_after: disciple.constitution,
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
