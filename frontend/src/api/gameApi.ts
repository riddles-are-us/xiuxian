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
  talents: Array<{
    talent_type: string;
    level: number;
  }>;
  heritage: {
    name: string;
    level: string;
  } | null;
  dao_companion: {
    companion_id: number;
    affinity: number;
  } | null;
  children_count: number;
  current_task: string | null;
  current_task_info: {
    task_id: number;
    task_name: string;
    duration: number;
    progress: number;
  } | null;
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
  suitable_disciples: {
    free: number[];
    busy: number[];
  };
  assigned_to: number | null;
  duration: number;
  progress: number;
  expiry_turns: number;
  created_turn: number;
  remaining_turns: number;
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
    power_level?: number;
    relationship?: number;
    danger_level?: number;
    realm_type?: string;
    difficulty?: number;
    level?: number;
    is_demon?: boolean;
  };
}

export interface MapData {
  width: number;
  height: number;
  elements: MapElement[];
}

export const gameApi = {
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

  nextTurn: async (gameId: string) => {
    // 先结束当前回合
    await axios.post(`${API_BASE}/game/${gameId}/turn/end`, {
      assignments: []
    });
    // 再开始新回合
    const response = await axios.post(`${API_BASE}/game/${gameId}/turn/start`);
    return response.data.data;
  }
};
