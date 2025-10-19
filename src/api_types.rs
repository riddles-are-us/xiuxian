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
    pub talents: Vec<TalentDto>,
    pub heritage: Option<HeritageDto>,
    pub dao_companion: Option<DaoCompanionDto>,
    pub children_count: usize,
    pub current_task: Option<String>,
}

impl From<&Disciple> for DiscipleDto {
    fn from(disciple: &Disciple) -> Self {
        Self {
            id: disciple.id,
            name: disciple.name.clone(),
            disciple_type: format!("{:?}", disciple.disciple_type),
            cultivation: CultivationDto {
                level: format!("{:?}", disciple.cultivation.current_level),
                progress: disciple.cultivation.progress,
            },
            age: disciple.age,
            lifespan: disciple.lifespan,
            dao_heart: disciple.dao_heart,
            talents: disciple.talents.iter().map(|t| t.into()).collect(),
            heritage: disciple.heritage.as_ref().map(|h| h.into()),
            dao_companion: disciple.dao_companion.as_ref().map(|dc| DaoCompanionDto {
                companion_id: dc.companion_id,
                affinity: dc.affinity,
            }),
            children_count: disciple.children.len(),
            current_task: disciple.current_task.clone(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct CultivationDto {
    pub level: String,
    pub progress: u32,
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
    pub suitable_disciples: SuitableDisciples,
    pub assigned_to: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskRewards {
    pub progress: u32,
    pub resources: u32,
    pub reputation: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct SuitableDisciples {
    pub free: Vec<usize>,
    pub busy: Vec<usize>,
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
