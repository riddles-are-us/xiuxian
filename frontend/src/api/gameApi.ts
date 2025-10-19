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
    progress: number;
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
  }
};
