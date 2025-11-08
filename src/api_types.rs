use serde::{Deserialize, Serialize};
use crate::disciple::{Disciple, Talent, Heritage};
use crate::sect::Sect;

/// API响应包装
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message,
                details: None,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

/// 版本信息响应
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub api_version: String,
    pub app_name: String,
}

/// 创建游戏请求
#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub sect_name: String,
}

/// 游戏信息响应
#[derive(Debug, Serialize)]
pub struct GameInfoResponse {
    pub game_id: String,
    pub sect: SectDto,
    pub state: String,
}

/// 宗门DTO
#[derive(Debug, Serialize, Clone)]
pub struct SectDto {
    pub name: String,
    pub year: u32,
    pub resources: u32,
    pub reputation: i32,
    pub disciples_count: usize,
}

impl From<&Sect> for SectDto {
    fn from(sect: &Sect) -> Self {
        Self {
            name: sect.name.clone(),
            year: sect.year,
            resources: sect.resources,
            reputation: sect.reputation,
            disciples_count: sect.alive_disciples().len(),
        }
    }
}

/// 道侣DTO
#[derive(Debug, Serialize, Clone)]
pub struct DaoCompanionDto {
    pub companion_id: usize,
    pub affinity: u32,
}

/// 弟子DTO
#[derive(Debug, Serialize, Clone)]
pub struct DiscipleDto {
    pub id: usize,
    pub name: String,
    pub disciple_type: String,
    pub cultivation: CultivationDto,
    pub age: u32,
    pub lifespan: u32,
    pub dao_heart: u32,
    pub energy: u32,        // 精力 0-100
    pub constitution: u32,   // 体魄 0-100
    pub talents: Vec<TalentDto>,
    pub heritage: Option<HeritageDto>,
    pub dao_companion: Option<DaoCompanionDto>,
    pub children_count: usize,
    pub current_task: Option<String>,
    pub current_task_info: Option<CurrentTaskInfo>,
}

/// 当前任务详情
#[derive(Debug, Serialize, Clone)]
pub struct CurrentTaskInfo {
    pub task_id: usize,
    pub task_name: String,
    pub duration: u32,
    pub progress: u32,
}

impl From<&Disciple> for DiscipleDto {
    fn from(disciple: &Disciple) -> Self {
        Self {
            id: disciple.id,
            name: disciple.name.clone(),
            disciple_type: format!("{:?}", disciple.disciple_type),
            cultivation: CultivationDto {
                level: format!("{:?}", disciple.cultivation.current_level),
                sub_level: format!("{}", disciple.cultivation.sub_level),  // 使用Display trait
                progress: disciple.cultivation.progress,
                cultivation_path: disciple.cultivation.cultivation_path.as_ref().map(|path| {
                    let (total_completed, total_required) = path.progress();
                    CultivationPathDto {
                        required: path.required.clone(),
                        completed: path.completed.clone(),
                        total_required,
                        total_completed,
                    }
                }),
            },
            age: disciple.age,
            lifespan: disciple.lifespan,
            dao_heart: disciple.dao_heart,
            energy: disciple.energy,
            constitution: disciple.constitution,
            talents: disciple.talents.iter().map(|t| t.into()).collect(),
            heritage: disciple.heritage.as_ref().map(|h| h.into()),
            dao_companion: disciple.dao_companion.as_ref().map(|dc| DaoCompanionDto {
                companion_id: dc.companion_id,
                affinity: dc.affinity,
            }),
            children_count: disciple.children.len(),
            current_task: disciple.current_task.clone(),
            current_task_info: None,  // 将在web_server中填充
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct CultivationDto {
    pub level: String,
    pub sub_level: String,           // 小境界（初期、中期、大圆满）
    pub progress: u32,                // 当前小境界进度 0-100
    pub cultivation_path: Option<CultivationPathDto>,  // 修炼路径（大圆满时）
}

#[derive(Debug, Serialize, Clone)]
pub struct CultivationPathDto {
    pub required: std::collections::HashMap<String, u32>,  // 需要完成的任务类型和数量
    pub completed: std::collections::HashMap<String, u32>, // 每种类型已完成的数量
    pub total_required: u32,                                // 总共需要完成的任务数
    pub total_completed: u32,                               // 总共已完成的任务数
}

#[derive(Debug, Serialize, Clone)]
pub struct TalentDto {
    pub talent_type: String,
    pub level: u32,
}

impl From<&Talent> for TalentDto {
    fn from(talent: &Talent) -> Self {
        Self {
            talent_type: format!("{:?}", talent.talent_type),
            level: talent.level,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct HeritageDto {
    pub name: String,
    pub level: String,
    pub tribulation_bonus: f32,
}

impl From<&Heritage> for HeritageDto {
    fn from(heritage: &Heritage) -> Self {
        Self {
            name: heritage.name.clone(),
            level: format!("{:?}", heritage.level),
            tribulation_bonus: heritage.tribulation_bonus,
        }
    }
}

/// 任务DTO
#[derive(Debug, Serialize, Clone)]
pub struct TaskDto {
    pub id: usize,
    pub name: String,
    pub task_type: String,
    pub rewards: TaskRewards,
    pub dao_heart_impact: i32,
    pub assigned_to: Option<usize>,
    pub duration: u32,           // 任务执行时间（回合数）
    pub progress: u32,            // 当前执行进度（回合数）
    pub expiry_turns: u32,        // 失效时间
    pub created_turn: u32,        // 创建回合
    pub remaining_turns: u32,     // 剩余回合数直到失效
    pub energy_cost: u32,        // 精力消耗（每回合）
    pub constitution_cost: u32,   // 体魄消耗（每回合）
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskRewards {
    pub progress: u32,
    pub resources: u32,
    pub reputation: i32,
}

/// 任务分配请求
#[derive(Debug, Deserialize)]
pub struct AssignTaskRequest {
    pub disciple_id: usize,
}

/// 任务分配响应
#[derive(Debug, Serialize)]
pub struct AssignTaskResponse {
    pub task_id: usize,
    pub disciple_id: usize,
    pub message: String,
}

/// 回合开始响应
#[derive(Debug, Serialize)]
pub struct TurnStartResponse {
    pub year: u32,
    pub events: Vec<GameEventDto>,
    pub tasks: Vec<TaskDto>,
    pub disciples: Vec<DiscipleDto>,
}

#[derive(Debug, Serialize)]
pub struct GameEventDto {
    pub event_type: String,
    pub message: String,
}

/// 回合结束请求
#[derive(Debug, Deserialize)]
pub struct TurnEndRequest {
    pub assignments: Vec<TaskAssignmentDto>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskAssignmentDto {
    pub task_id: usize,
    pub disciple_id: usize,
}

/// 回合结束响应
#[derive(Debug, Serialize)]
pub struct TurnEndResponse {
    pub results: Vec<TaskResultDto>,
    pub game_state: String,
}

#[derive(Debug, Serialize)]
pub struct TaskResultDto {
    pub task_id: usize,
    pub disciple_id: usize,
    pub success: bool,
    pub rewards: Option<TaskRewards>,
    pub message: String,
}

/// 统计信息响应
#[derive(Debug, Serialize)]
pub struct StatisticsResponse {
    pub year: u32,
    pub total_disciples: usize,
    pub disciples_by_type: DisciplesByType,
    pub resources: u32,
    pub reputation: i32,
    pub cultivation_distribution: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct DisciplesByType {
    pub outer: usize,
    pub inner: usize,
    pub personal: usize,
}

/// 渡劫候选人响应
#[derive(Debug, Serialize)]
pub struct TribulationCandidatesResponse {
    pub candidates: Vec<TribulationCandidateDto>,
}

#[derive(Debug, Serialize)]
pub struct TribulationCandidateDto {
    pub disciple_id: usize,
    pub name: String,
    pub current_level: String,
    pub success_rate: f32,
    pub dao_heart: u32,
    pub heritage_bonus: f32,
}

/// 渡劫请求
#[derive(Debug, Deserialize)]
pub struct TribulationRequest {
    pub disciple_id: usize,
}

/// 渡劫响应
#[derive(Debug, Serialize)]
pub struct TribulationResponse {
    pub success: bool,
    pub disciple_id: usize,
    pub name: String,
    pub new_level: Option<String>,
    pub message: String,
}

/// 地图元素DTO
#[derive(Debug, Serialize, Clone)]
pub struct MapElementDto {
    pub element_type: String,
    pub name: String,
    pub position: PositionDto,
    pub details: MapElementDetails,
}

#[derive(Debug, Serialize, Clone)]
pub struct PositionDto {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum MapElementDetails {
    Village { population: u32, prosperity: u32 },
    Faction { power_level: u32, relationship: i32 },
    DangerousLocation { danger_level: u32 },
    SecretRealm { realm_type: String, difficulty: u32 },
    Monster { level: u32, is_demon: bool },
}

/// 地图数据响应
#[derive(Debug, Serialize)]
pub struct MapDataResponse {
    pub width: i32,
    pub height: i32,
    pub elements: Vec<MapElementDto>,
}

/// 丹药库存响应
#[derive(Debug, Serialize)]
pub struct PillInventoryResponse {
    pub pills: std::collections::HashMap<String, PillInfo>,
}

#[derive(Debug, Serialize)]
pub struct PillInfo {
    pub count: u32,
    pub name: String,
    pub description: String,
    pub energy_restore: u32,
    pub constitution_restore: u32,
}

/// 服用丹药请求
#[derive(Debug, Deserialize)]
pub struct UsePillRequest {
    pub disciple_id: usize,
    pub pill_type: String,
}

/// 服用丹药响应
#[derive(Debug, Serialize)]
pub struct UsePillResponse {
    pub success: bool,
    pub message: String,
    pub disciple_name: String,
    pub energy_before: u32,
    pub energy_after: u32,
    pub constitution_before: u32,
    pub constitution_after: u32,
}
