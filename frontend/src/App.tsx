import React, { useState, useEffect } from 'react';
import { gameApi, GameInfo, Disciple, Task } from './api/gameApi';
import './App.css';

function App() {
  const [gameId, setGameId] = useState<string | null>(
    localStorage.getItem('gameId')
  );
  const [gameInfo, setGameInfo] = useState<GameInfo | null>(null);
  const [disciples, setDisciples] = useState<Disciple[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (gameId) {
      loadGameData(gameId);
    }
  }, [gameId]);

  const loadGameData = async (id: string) => {
    try {
      setLoading(true);
      const [info, disciplesList, tasksList] = await Promise.all([
        gameApi.getGame(id),
        gameApi.getDisciples(id),
        gameApi.getTasks(id)
      ]);
      setGameInfo(info);
      setDisciples(disciplesList);
      setTasks(tasksList);
      setError(null);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const createNewGame = async () => {
    const sectName = prompt('输入宗门名称:', '青云宗') || '青云宗';
    try {
      setLoading(true);
      const game = await gameApi.createGame(sectName);
      setGameId(game.game_id);
      localStorage.setItem('gameId', game.game_id);
      await loadGameData(game.game_id);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const startNewTurn = async () => {
    if (!gameId) return;
    try {
      setLoading(true);
      await gameApi.startTurn(gameId);
      await loadGameData(gameId);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const assignTask = async (taskId: number, discipleId: number) => {
    if (!gameId) return;
    try {
      await gameApi.assignTask(gameId, taskId, discipleId);
      await loadGameData(gameId);
    } catch (err: any) {
      setError(err.message);
    }
  };

  const autoAssign = async () => {
    if (!gameId) return;
    try {
      setLoading(true);
      await gameApi.autoAssignTasks(gameId);
      await loadGameData(gameId);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const endTurn = async () => {
    if (!gameId) return;
    try {
      setLoading(true);
      await gameApi.endTurn(gameId);
      await loadGameData(gameId);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="loading">加载中...</div>;
  }

  if (!gameId || !gameInfo) {
    return (
      <div className="App">
        <div className="welcome">
          <h1>修仙宗门模拟器</h1>
          <button onClick={createNewGame} className="btn-primary">
            创建新游戏
          </button>
          {error && <div className="error">{error}</div>}
        </div>
      </div>
    );
  }

  return (
    <div className="App">
      <header>
        <h1>{gameInfo.sect.name}</h1>
        <div className="stats">
          <span>年份: {gameInfo.sect.year}</span>
          <span>资源: {gameInfo.sect.resources}</span>
          <span>声望: {gameInfo.sect.reputation}</span>
          <span>弟子: {gameInfo.sect.disciples_count}</span>
        </div>
      </header>

      <div className="controls">
        <button onClick={startNewTurn} className="btn-primary">开始新回合</button>
        <button onClick={autoAssign} className="btn-secondary">自动分配任务</button>
        <button onClick={endTurn} className="btn-warning">结束回合</button>
      </div>

      {error && <div className="error">{error}</div>}

      <div className="content">
        <div className="disciples-section">
          <h2>弟子列表 ({disciples.length})</h2>
          <div className="disciples-grid">
            {disciples.map(d => (
              <div key={d.id} className="disciple-card">
                <h3>{d.name}</h3>
                <p>类型: {d.disciple_type}</p>
                <p>修为: {d.cultivation.level} ({d.cultivation.progress}%)</p>
                <p>道心: {d.dao_heart}</p>
                <p>年龄: {d.age}/{d.lifespan}</p>
                {d.current_task && (
                  <p className="current-task">📋 {d.current_task}</p>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="tasks-section">
          <h2>任务列表 ({tasks.length})</h2>
          <div className="tasks-list">
            {tasks.map(t => (
              <div key={t.id} className="task-card">
                <h3>{t.name}</h3>
                <p>{t.task_type}</p>
                <div className="rewards">
                  <span>修为+{t.rewards.progress}</span>
                  <span>资源+{t.rewards.resources}</span>
                  <span>声望+{t.rewards.reputation}</span>
                </div>
                {t.assigned_to ? (
                  <p className="assigned">
                    ✓ 已分配给 {disciples.find(d => d.id === t.assigned_to)?.name}
                  </p>
                ) : (
                  <div className="assign-buttons">
                    {disciples
                      .filter(d => !tasks.some(task => task.assigned_to === d.id))
                      .slice(0, 3)
                      .map(d => (
                        <button
                          key={d.id}
                          onClick={() => assignTask(t.id, d.id)}
                          className="btn-small"
                        >
                          分配给 {d.name}
                        </button>
                      ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
