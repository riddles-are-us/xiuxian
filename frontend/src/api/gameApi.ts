import axios from 'axios';

const API_BASE = process.env.REACT_APP_API_URL || '/api';

export interface GameInfo {
  game_id: string;
  sect: {
    name: string;
    year: number;
    resources: number;
    reputation: number;
    disciples_count: number;
  };
  state: string;
}

// 关系分数
export interface RelationScores {
  romance: number;       // 男女情感 0-100
  mentorship: number;    // 师徒关系 0-100
  comrade: number;       // 战友关系 0-100
  understanding: number; // 认知程度 0-100
  fateful_bond: number;  // 机缘关系 0-100
}

// 关系详情
export interface Relationship {
  target_id: number;
  target_name: string;
  scores: RelationScores;
  established_year: number;
  is_dao_companion: boolean;
  is_master: boolean;
  is_disciple: boolean;
  primary_relation: string;
  highest_level: string;
}

// 关系摘要
export interface RelationshipSummary {
  dao_companion_id: number | null;
  master_id: number | null;
  disciple_ids: number[];
  total_relationships: number;
}

export interface Disciple {
  id: number;
  name: string;
  disciple_type: string;
  cultivation: {
    level: string;
    sub_level: string;        // 小境界
    progress: number;          // 当前小境界进度 0-100
    cultivation_path: {        // 修炼路径
      required: { [key: string]: number };   // 需要完成的任务类型和数量
      completed: { [key: string]: number };  // 每种类型已完成的数量
      total_required: number;                // 总共需要完成的任务数
      total_completed: number;               // 总共已完成的任务数
    } | null;
  };
  age: number;
  lifespan: number;
  dao_heart: number;
  energy: number;            // 精力 0-100
  constitution: number;      // 体魄 0-100
  talents: Array<{
    talent_type: string;
    level: number;
  }>;
  heritage: {
    name: string;
    level: string;
  } | null;
  relationship_summary: RelationshipSummary;  // 关系摘要
  children_count: number;
  current_task_info: {
    task_id: number;
    task_name: string;
    duration: number;
    progress: number;
  } | null;
  position: {
    x: number;
    y: number;
  };
  movement_range: number;    // 每回合可移动的最大距离（格子数）
  moves_remaining: number;   // 本回合剩余移动距离
}

export interface Task {
  id: number;
  name: string;
  task_type: string;
  rewards: {
    progress: number;
    resources: number;
    reputation: number;
  };
  dao_heart_impact: number;
  assigned_to: number[];      // 已分配的弟子ID列表（支持多人）
  max_participants: number;   // 最大参与人数
  duration: number;
  progress: number;
  expiry_turns: number;
  created_turn: number;
  remaining_turns: number;
  energy_cost: number;        // 精力消耗（每回合）
  constitution_cost: number;   // 体魄消耗（每回合）
  skill_required: string | null;  // 需要的技能
  suitable_disciples: {       // 合适的弟子
    free: number[];           // 空闲的合适弟子ID
    busy: number[];           // 忙碌的合适弟子ID
  };
  enemy_info: {               // 敌人信息（战斗任务）
    enemy_id: string;         // 怪物唯一ID
    enemy_name: string;       // 怪物名称
    enemy_level: number;      // 怪物等级
  } | null;
  position: {                 // 任务位置
    x: number;
    y: number;
  } | null;
}

export interface AttackInfo {
  attacker_name: string;
  attacker_level: number;
  is_demon: boolean;
}

export interface MapElement {
  element_type: string;
  name: string;
  position: {
    x: number;
    y: number;
  };
  details: {
    type: string;
    population?: number;
    prosperity?: number;
    under_attack?: AttackInfo;  // 受攻击信息（村庄、势力、秘境）
    power_level?: number;
    relationship?: number;
    danger_level?: number;
    realm_type?: string;
    difficulty?: number;
    monster_id?: string;  // 怪物唯一ID（格式：monster_X）
    level?: number;
    is_demon?: boolean;
    growth_rate?: number;  // 成长速率 (每回合升级概率)
    invading_location?: string;  // 妖魔正在入侵的地点ID
    terrain_type?: string;  // 地形类型：Mountain, Water, Forest, Plain
    // 草药相关
    herb_id?: string;       // 草药唯一ID
    quality?: string;       // 品质：普通、良品、稀有、珍品、仙品
    growth_stage?: number;  // 生长阶段 0-100
    max_growth?: number;    // 最大生长值
    is_mature?: boolean;    // 是否成熟
  };
}

export interface MapData {
  width: number;
  height: number;
  elements: MapElement[];
}

export interface VersionInfo {
  api_version: string;
  app_name: string;
}

export interface PillInfo {
  count: number;
  name: string;
  description: string;
  energy_restore: number;
  constitution_restore: number;
  cultivation_boost: number;
}

export interface PillInventory {
  pills: { [key: string]: PillInfo };
}

export interface UsePillResponse {
  success: boolean;
  message: string;
  disciple_name: string;
  energy_before: number;
  energy_after: number;
  constitution_before: number;
  constitution_after: number;
  progress_before: number;
  progress_after: number;
}

export interface BuildingDto {
  id: string;
  name: string;
  description: string;
  base_cost: number;
  actual_cost: number;
  parent_id: string | null;
  is_built: boolean;
  can_build: boolean;
  effects: string[];
}

export interface BuildingTreeResponse {
  total_buildings: number;
  built_count: number;
  buildings_built_count: number;
  cost_multiplier: number;
  available_resources: number;
  buildings: BuildingDto[];
}

export interface BuildBuildingResponse {
  success: boolean;
  message: string;
  building_name: string;
  cost: number;
  resources_before: number;
  resources_after: number;
  effects_count: number;
}

// 草药条目
export interface HerbEntry {
  name: string;
  quality: string;
  count: number;
}

// 草药仓库响应
export interface HerbInventoryResponse {
  total_count: number;
  herbs: HerbEntry[];
}

// 丹药配方
export interface PillRecipe {
  pill_type: string;
  name: string;
  description: string;
  required_herb_quality: string;
  required_herb_count: number;
  resource_cost: number;
  success_rate: number;
  output_count: number;
  can_craft: boolean;
  reason: string | null;
}

// 炼制丹药响应
export interface RefinePillResponse {
  success: boolean;
  message: string;
  pill_name: string | null;
  output_count: number | null;
}

// 采集草药信息
export interface CollectedHerbInfo {
  name: string;
  quality: string;
}

// 移动弟子响应（更新）
export interface MoveDiscipleResponse {
  success: boolean;
  message: string;
  disciple_id: number;
  disciple_name: string;
  old_position: { x: number; y: number };
  new_position: { x: number; y: number };
  collected_herb: CollectedHerbInfo | null;
}

// 任务资格检查响应
export interface TaskEligibilityResponse {
  task_id: number;
  task_name: string;
  disciple_id: number;
  disciple_name: string;
  eligible: boolean;
  reason: string | null;
  success_rate: number | null;  // 战斗任务的成功率 (0.0 - 1.0)
  disciple_combat_level: number | null;  // 弟子战斗等级
  enemy_level: number | null;  // 敌人等级
}

// 任务执行结果
export interface TaskResultDto {
  task_id: number;
  disciple_id: number;
  success: boolean;
  rewards: {
    progress: number;
    resources: number;
    reputation: number;
  } | null;
  message: string;
}

// 回合结束响应
export interface TurnEndResponse {
  results: TaskResultDto[];
  game_state: string;
}

// 下一回合结果
export interface NextTurnResult {
  task_results: TaskResultDto[];
  pending_recruitment: Disciple | null;
}

export const gameApi = {
  getVersion: async (): Promise<VersionInfo> => {
    const response = await axios.get(`${API_BASE}/version`);
    return response.data.data;
  },

  createGame: async (sectName: string) => {
    const response = await axios.post(`${API_BASE}/game/new`, {
      sect_name: sectName
    });
    return response.data.data;
  },

  getGame: async (gameId: string): Promise<GameInfo> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}`);
    return response.data.data;
  },

  startTurn: async (gameId: string) => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/turn/start`);
    return response.data.data;
  },

  endTurn: async (gameId: string) => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/turn/end`, {
      assignments: []
    });
    return response.data.data;
  },

  getDisciples: async (gameId: string): Promise<Disciple[]> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/disciples`);
    return response.data.data;
  },

  getTasks: async (gameId: string): Promise<Task[]> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/tasks`);
    return response.data.data;
  },

  assignTask: async (gameId: string, taskId: number, discipleId: number) => {
    const response = await axios.post(
      `${API_BASE}/game/${gameId}/tasks/${taskId}/assign`,
      { disciple_id: discipleId }
    );
    return response.data.data;
  },

  unassignTask: async (gameId: string, taskId: number) => {
    const response = await axios.delete(
      `${API_BASE}/game/${gameId}/tasks/${taskId}/assign`
    );
    return response.data.data;
  },

  autoAssignTasks: async (gameId: string) => {
    const response = await axios.post(
      `${API_BASE}/game/${gameId}/tasks/auto-assign`
    );
    return response.data.data;
  },

  getStatistics: async (gameId: string) => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/statistics`);
    return response.data.data;
  },

  getMap: async (gameId: string): Promise<MapData> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/map`);
    return response.data.data;
  },

  nextTurn: async (gameId: string): Promise<NextTurnResult> => {
    // 先结束当前回合，获取任务执行结果
    const endTurnResponse = await axios.post(`${API_BASE}/game/${gameId}/turn/end`, {
      assignments: []
    });
    const turnEndData: TurnEndResponse = endTurnResponse.data.data;

    // 再开始新回合
    const response = await axios.post(`${API_BASE}/game/${gameId}/turn/start`);
    const startTurnData = response.data.data;

    return {
      task_results: turnEndData.results,
      pending_recruitment: startTurnData.pending_recruitment
    };
  },

  getPillInventory: async (gameId: string): Promise<PillInventory> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/pills`);
    return response.data.data;
  },

  usePill: async (gameId: string, discipleId: number, pillType: string): Promise<UsePillResponse> => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/pills/use`, {
      disciple_id: discipleId,
      pill_type: pillType
    });
    return response.data.data;
  },

  getBuildingTree: async (gameId: string): Promise<BuildingTreeResponse> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/buildings`);
    return response.data.data;
  },

  buildBuilding: async (gameId: string, buildingId: string): Promise<BuildBuildingResponse> => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/buildings/build`, {
      building_id: buildingId
    });
    return response.data.data;
  },

  recruitDisciple: async (gameId: string, accept: boolean) => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/recruit`, {
      accept
    });
    return response.data.data;
  },

  moveDisciple: async (gameId: string, discipleId: number, x: number, y: number) => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/disciples/${discipleId}/move`, {
      x,
      y
    });
    return response.data.data;
  },

  // 获取弟子的所有关系
  getDiscipleRelationships: async (gameId: string, discipleId: number): Promise<Relationship[]> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/disciples/${discipleId}/relationships`);
    return response.data.data.relationships;
  },

  // 获取草药仓库
  getHerbInventory: async (gameId: string): Promise<HerbInventoryResponse> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/herbs`);
    return response.data.data;
  },

  // 获取所有配方
  getRecipes: async (gameId: string): Promise<PillRecipe[]> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/recipes`);
    return response.data.data.recipes;
  },

  // 炼制丹药
  refinePill: async (gameId: string, pillType: string): Promise<RefinePillResponse> => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/refine`, {
      pill_type: pillType
    });
    return response.data.data;
  },

  // 检查弟子是否可以接受任务
  checkTaskEligibility: async (gameId: string, taskId: number, discipleId: number): Promise<TaskEligibilityResponse> => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/tasks/check-eligibility`, {
      task_id: taskId,
      disciple_id: discipleId
    });
    return response.data.data;
  }
};
