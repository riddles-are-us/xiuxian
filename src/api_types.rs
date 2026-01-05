use serde::{Deserialize, Serialize};
use crate::disciple::{Disciple, Talent, Heritage};
use crate::sect::Sect;
use crate::relationship::{Relationship, RelationDimension, RelationLevel, RelationScores};

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

/// 关系分数DTO
#[derive(Debug, Serialize, Clone)]
pub struct RelationScoresDto {
    pub romance: u32,       // 男女情感 0-100
    pub mentorship: u32,    // 师徒关系 0-100
    pub comrade: u32,       // 战友关系 0-100
    pub understanding: u32, // 认知程度 0-100
    pub fateful_bond: u32,  // 机缘关系 0-100
}

impl From<&RelationScores> for RelationScoresDto {
    fn from(scores: &RelationScores) -> Self {
        Self {
            romance: scores.romance,
            mentorship: scores.mentorship,
            comrade: scores.comrade,
            understanding: scores.understanding,
            fateful_bond: scores.fateful_bond,
        }
    }
}

/// 关系DTO
#[derive(Debug, Serialize, Clone)]
pub struct RelationshipDto {
    pub target_id: usize,
    pub target_name: String,
    pub scores: RelationScoresDto,
    pub established_year: u32,
    pub is_dao_companion: bool,
    pub is_master: bool,
    pub is_disciple: bool,
    pub primary_relation: String,
    pub highest_level: String,
}

/// 关系摘要DTO（用于弟子列表）
#[derive(Debug, Serialize, Clone)]
pub struct RelationshipSummaryDto {
    pub dao_companion_id: Option<usize>,
    pub master_id: Option<usize>,
    pub disciple_ids: Vec<usize>,
    pub total_relationships: usize,
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
    pub relationship_summary: RelationshipSummaryDto,  // 关系摘要
    pub children_count: usize,
    pub current_task_info: Option<CurrentTaskInfo>,
    pub position: PositionDto,  // 弟子在地图上的位置
    pub movement_range: u32,    // 每回合可移动的最大距离（格子数）
    pub moves_remaining: u32,   // 本回合剩余移动距离
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
            relationship_summary: RelationshipSummaryDto {
                dao_companion_id: disciple.get_dao_companion_id(),
                master_id: disciple.get_master_id(),
                disciple_ids: disciple.get_disciple_ids(),
                total_relationships: disciple.relationships.len(),
            },
            children_count: disciple.children.len(),
            current_task_info: None,  // 将在web_server中填充
            movement_range: disciple.cultivation.current_level.movement_range(),
            moves_remaining: disciple.moves_remaining,
            position: PositionDto {
                x: disciple.position.x,
                y: disciple.position.y,
            },
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
    pub assigned_to: Vec<usize>,  // 已分配的弟子ID列表（支持多人）
    pub max_participants: u32,    // 最大参与人数
    pub duration: u32,           // 任务执行时间（回合数）
    pub progress: u32,            // 当前执行进度（回合数）
    pub expiry_turns: u32,        // 失效时间
    pub created_turn: u32,        // 创建回合
    pub remaining_turns: u32,     // 剩余回合数直到失效
    pub energy_cost: u32,        // 精力消耗（每回合）
    pub constitution_cost: u32,   // 体魄消耗（每回合）
    pub skill_required: Option<String>,  // 需要的技能
    pub suitable_disciples: SuitableDisciples,  // 合适的弟子
    pub enemy_info: Option<EnemyInfo>,  // 敌人信息（战斗任务，包含唯一ID）
    pub position: Option<PositionDto>,  // 任务主位置（用于显示）
    pub valid_positions: Option<Vec<PositionDto>>,  // 所有有效位置（用于大型建筑）
}

/// 敌人信息（用于定位地图上的具体怪物）
#[derive(Debug, Serialize, Clone)]
pub struct EnemyInfo {
    pub enemy_id: String,      // 怪物唯一ID（格式：monster_X）
    pub enemy_name: String,    // 怪物名称
    pub enemy_level: u32,      // 怪物等级
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskRewards {
    pub progress: u32,
    pub resources: u32,
    pub reputation: i32,
}

/// 合适的弟子列表
#[derive(Debug, Serialize, Clone)]
pub struct SuitableDisciples {
    pub free: Vec<usize>,  // 空闲的合适弟子ID
    pub busy: Vec<usize>,  // 忙碌的合适弟子ID
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

/// 宗门被袭击状态
#[derive(Debug, Serialize, Clone)]
pub struct SectInvasionDto {
    pub monster_id: usize,        // 袭击宗门的怪物ID
    pub monster_name: String,     // 怪物名称
    pub turns_remaining: u32,     // 剩余回合数
}

/// 回合开始响应
#[derive(Debug, Serialize)]
pub struct TurnStartResponse {
    pub year: u32,
    pub events: Vec<GameEventDto>,
    pub tasks: Vec<TaskDto>,
    pub disciples: Vec<DiscipleDto>,
    pub pending_recruitment: Option<DiscipleDto>,  // 待招募的弟子（需要确认）
    pub sect_invasion: Option<SectInvasionDto>,    // 宗门被袭击状态
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
    pub disciple_name: String,
    pub success: bool,
    pub rewards: Option<TaskRewards>,
    pub message: String,
    pub disciple_died: bool,  // 弟子是否死亡
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
    pub size: Option<SizeDto>,  // 建筑尺寸，None 表示 1x1
    pub details: MapElementDetails,
}

#[derive(Debug, Serialize, Clone)]
pub struct PositionDto {
    pub x: i32,
    pub y: i32,
}

/// 尺寸DTO（用于大型建筑）
#[derive(Debug, Serialize, Clone)]
pub struct SizeDto {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum MapElementDetails {
    Village { population: u32, prosperity: u32, under_attack: Option<AttackInfo> },
    Faction { power_level: u32, relationship: i32, under_attack: Option<AttackInfo> },
    DangerousLocation { danger_level: u32 },
    SecretRealm { realm_type: String, difficulty: u32, under_attack: Option<AttackInfo> },
    Monster { monster_id: String, level: u32, is_demon: bool, growth_rate: f64, invading_location: Option<String> },
    Terrain { terrain_type: String },
    Herb { herb_id: String, quality: String, growth_stage: u32, max_growth: u32, is_mature: bool },
}

/// 攻击信息
#[derive(Debug, Serialize, Clone)]
pub struct AttackInfo {
    pub attacker_name: String,  // 攻击者名称
    pub attacker_level: u32,     // 攻击者等级
    pub is_demon: bool,          // 是否为魔物
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
    pub cultivation_boost: u32,
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
    pub progress_before: u32,
    pub progress_after: u32,
}

/// 建筑DTO
#[derive(Debug, Serialize, Clone)]
pub struct BuildingDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: u32,
    pub actual_cost: u32,  // 考虑倍增后的实际成本
    pub parent_id: Option<String>,
    pub is_built: bool,
    pub can_build: bool,  // 是否可以建造（父节点已建造且自己未建造）
    pub effects: Vec<String>,  // 效果描述
}

/// 建筑树响应
#[derive(Debug, Serialize)]
pub struct BuildingTreeResponse {
    pub total_buildings: usize,
    pub built_count: usize,
    pub buildings_built_count: u32,  // 已建造数量（用于成本计算）
    pub cost_multiplier: u32,  // 当前成本倍数 2^n
    pub available_resources: u32,  // 宗门当前资源
    pub buildings: Vec<BuildingDto>,
}

/// 建造建筑请求
#[derive(Debug, Deserialize)]
pub struct BuildBuildingRequest {
    pub building_id: String,
}

/// 建造建筑响应
#[derive(Debug, Serialize)]
pub struct BuildBuildingResponse {
    pub success: bool,
    pub message: String,
    pub building_name: String,
    pub cost: u32,
    pub resources_before: u32,
    pub resources_after: u32,
    pub effects_count: usize,
}

/// 任务资格检查请求
#[derive(Debug, Deserialize)]
pub struct TaskEligibilityRequest {
    pub task_id: usize,
    pub disciple_id: usize,
}

/// 任务资格检查响应
#[derive(Debug, Serialize)]
pub struct TaskEligibilityResponse {
    pub task_id: usize,
    pub task_name: String,
    pub disciple_id: usize,
    pub disciple_name: String,
    pub eligible: bool,
    pub reason: Option<String>,
    pub success_rate: Option<f64>,  // 战斗任务的成功率 (0.0 - 1.0)
    pub disciple_combat_level: Option<u32>,  // 弟子战斗等级
    pub enemy_level: Option<u32>,  // 敌人等级
}

/// 招募弟子请求
#[derive(Debug, Deserialize)]
pub struct RecruitDiscipleRequest {
    pub accept: bool,  // true=接受招募, false=拒绝招募
}

/// 招募弟子响应
#[derive(Debug, Serialize)]
pub struct RecruitDiscipleResponse {
    pub success: bool,
    pub message: String,
    pub disciple: Option<DiscipleDto>,  // 招募成功时返回弟子信息
    pub resources_before: u32,
    pub resources_after: u32,
    pub cost: u32,  // 招募成本（1000）
}

/// 移动弟子请求
#[derive(Debug, Deserialize)]
pub struct MoveDiscipleRequest {
    pub x: i32,
    pub y: i32,
}

/// 采集的草药信息
#[derive(Debug, Serialize)]
pub struct CollectedHerbInfo {
    pub name: String,
    pub quality: String,
}

/// 移动弟子响应
#[derive(Debug, Serialize)]
pub struct MoveDiscipleResponse {
    pub success: bool,
    pub message: String,
    pub disciple_id: usize,
    pub disciple_name: String,
    pub old_position: PositionDto,
    pub new_position: PositionDto,
    pub collected_herb: Option<CollectedHerbInfo>,
}

// === 关系系统相关 ===

/// 弟子关系列表响应
#[derive(Debug, Serialize)]
pub struct DiscipleRelationshipsResponse {
    pub disciple_id: usize,
    pub disciple_name: String,
    pub relationships: Vec<RelationshipDto>,
}

/// 设置师徒关系请求
#[derive(Debug, Deserialize)]
pub struct SetMentorshipRequest {
    pub master_id: usize,
    pub disciple_id: usize,
}

/// 设置师徒关系响应
#[derive(Debug, Serialize)]
pub struct SetMentorshipResponse {
    pub success: bool,
    pub message: String,
    pub master_name: String,
    pub disciple_name: String,
}

/// 设置道侣关系请求
#[derive(Debug, Deserialize)]
pub struct SetDaoCompanionRequest {
    pub disciple1_id: usize,
    pub disciple2_id: usize,
}

/// 设置道侣关系响应
#[derive(Debug, Serialize)]
pub struct SetDaoCompanionResponse {
    pub success: bool,
    pub message: String,
    pub disciple1_name: String,
    pub disciple2_name: String,
}

/// 更新关系分数请求
#[derive(Debug, Deserialize)]
pub struct UpdateRelationshipRequest {
    pub from_id: usize,
    pub to_id: usize,
    pub dimension: String,  // Romance, Mentorship, Comrade, Understanding, FatefulBond
    pub delta: i32,
}

/// 更新关系分数响应
#[derive(Debug, Serialize)]
pub struct UpdateRelationshipResponse {
    pub success: bool,
    pub message: String,
    pub from_name: String,
    pub to_name: String,
    pub dimension: String,
    pub old_score: u32,
    pub new_score: u32,
    pub level_up: Option<String>,
}

/// 所有关系响应
#[derive(Debug, Serialize)]
pub struct AllRelationshipsResponse {
    pub total_relationships: usize,
    pub relationships: Vec<RelationshipPairDto>,
}

/// 关系对DTO
#[derive(Debug, Serialize, Clone)]
pub struct RelationshipPairDto {
    pub from_id: usize,
    pub from_name: String,
    pub to_id: usize,
    pub to_name: String,
    pub scores: RelationScoresDto,
    pub primary_relation: String,
}

// === 草药和丹药仓库相关 ===

/// 草药条目DTO
#[derive(Debug, Serialize)]
pub struct HerbEntryDto {
    pub name: String,
    pub quality: String,
    pub count: u32,
}

/// 草药仓库响应
#[derive(Debug, Serialize)]
pub struct HerbInventoryResponse {
    pub total_count: u32,
    pub herbs: Vec<HerbEntryDto>,
}


/// 丹药配方DTO
#[derive(Debug, Serialize)]
pub struct PillRecipeDto {
    pub pill_type: String,
    pub name: String,
    pub description: String,
    pub required_herb_quality: String,
    pub required_herb_count: u32,
    pub resource_cost: u32,
    pub success_rate: f64,
    pub output_count: u32,
    pub can_craft: bool,
    pub reason: Option<String>,
}

/// 所有配方响应
#[derive(Debug, Serialize)]
pub struct AllRecipesResponse {
    pub recipes: Vec<PillRecipeDto>,
}

/// 炼制丹药请求
#[derive(Debug, Deserialize)]
pub struct RefinePillRequest {
    pub pill_type: String,
}

/// 炼制丹药响应
#[derive(Debug, Serialize)]
pub struct RefinePillResponse {
    pub success: bool,
    pub message: String,
    pub pill_name: Option<String>,
    pub output_count: Option<u32>,
}
