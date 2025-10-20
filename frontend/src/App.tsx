import React, { useState, useEffect } from 'react';
import { gameApi, GameInfo, Disciple, Task, MapData } from './api/gameApi';
import MapView from './MapView';
import './App.css';

function App() {
  const [gameId, setGameId] = useState<string | null>(
    localStorage.getItem('gameId')
  );
  const [gameInfo, setGameInfo] = useState<GameInfo | null>(null);
  const [disciples, setDisciples] = useState<Disciple[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [mapData, setMapData] = useState<MapData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showMap, setShowMap] = useState(false);

  useEffect(() => {
    if (gameId) {
      loadGameData(gameId);
    }
  }, [gameId]);

  const loadGameData = async (id: string) => {
    try {
      setLoading(true);
      const [info, disciplesList, tasksList, map] = await Promise.all([
        gameApi.getGame(id),
        gameApi.getDisciples(id),
        gameApi.getTasks(id),
        gameApi.getMap(id)
      ]);
      setGameInfo(info);
      setDisciples(disciplesList);
      setTasks(tasksList);
      setMapData(map);
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
        <button onClick={() => setShowMap(!showMap)} className="btn-primary">
          {showMap ? '隐藏地图' : '显示地图'}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {showMap && mapData && (
        <div style={{ padding: '2rem', maxWidth: '1400px', margin: '0 auto' }}>
          <MapView mapData={mapData} />
        </div>
      )}

      <div className="content">
        <div className="disciples-section">
          <h2>弟子列表 ({disciples.length})</h2>
          <div className="disciples-grid">
            {disciples.map(d => (
              <div key={d.id} className="disciple-card">
                <div className="disciple-header">
                  <h3>{d.name}</h3>
                  <span className="disciple-type-badge">{d.disciple_type}</span>
                </div>

                <div className="disciple-info">
                  <div className="info-row">
                    <span className="label">修为:</span>
                    <span className="value">{d.cultivation.level} ({d.cultivation.progress}%)</span>
                  </div>

                  <div className="progress-bar">
                    <div className="progress-fill" style={{width: `${d.cultivation.progress}%`}}></div>
                  </div>

                  <div className="info-row">
                    <span className="label">道心:</span>
                    <span className="value">{d.dao_heart}/100</span>
                  </div>

                  <div className="info-row">
                    <span className="label">寿元:</span>
                    <span className="value">{d.age}/{d.lifespan}岁</span>
                  </div>

                  {d.talents && d.talents.length > 0 && (
                    <div className="talents-section">
                      <span className="label">天赋:</span>
                      <div className="talents">
                        {d.talents.map((t, i) => (
                          <span key={i} className="talent-badge">
                            {t.talent_type} Lv.{t.level}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {d.heritage && (
                    <div className="heritage-section">
                      <span className="heritage-badge">
                        📜 {d.heritage.name} ({d.heritage.level})
                      </span>
                    </div>
                  )}

                  {d.dao_companion && (
                    <div className="companion-section">
                      <span className="companion-badge">
                        💑 道侣 (亲密度: {d.dao_companion.affinity})
                      </span>
                    </div>
                  )}

                  {d.children_count > 0 && (
                    <div className="children-section">
                      <span className="children-badge">
                        👶 子女: {d.children_count}
                      </span>
                    </div>
                  )}

                  {d.current_task_info && (
                    <div className="current-task">
                      <div className="task-name">📋 {d.current_task_info.task_name}</div>
                      <div className="task-progress-container">
                        <div className="task-progress-bar">
                          <div
                            className="task-progress-fill"
                            style={{width: `${(d.current_task_info.progress / d.current_task_info.duration) * 100}%`}}
                          ></div>
                        </div>
                        <span className="task-progress-text">
                          {d.current_task_info.progress}/{d.current_task_info.duration} 回合
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="tasks-section">
          <h2>任务列表 ({tasks.length})</h2>
          <div className="tasks-list">
            {tasks.map(t => (
              <div key={t.id} className="task-card">
                <div className="task-header">
                  <h3>{t.name}</h3>
                  <span className={`task-expiry ${t.remaining_turns <= 2 ? 'urgent' : ''}`}>
                    ⏰ {t.remaining_turns}回合后失效
                  </span>
                </div>
                <p>{t.task_type}</p>
                <div className="task-duration">
                  ⏱️ 需要执行 {t.duration} 回合
                </div>
                <div className="rewards">
                  <span>修为+{t.rewards.progress}</span>
                  <span>资源+{t.rewards.resources}</span>
                  <span>声望+{t.rewards.reputation}</span>
                </div>
                {t.assigned_to ? (
                  <div>
                    <p className="assigned">
                      ✓ 已分配给 {disciples.find(d => d.id === t.assigned_to)?.name}
                    </p>
                    {t.progress > 0 && (
                      <div className="task-progress-container">
                        <div className="task-progress-bar">
                          <div
                            className="task-progress-fill"
                            style={{width: `${(t.progress / t.duration) * 100}%`}}
                          ></div>
                        </div>
                        <span className="task-progress-text">
                          进度: {t.progress}/{t.duration}
                        </span>
                      </div>
                    )}
                  </div>
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
