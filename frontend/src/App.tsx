import React, { useState, useEffect } from 'react';
import { gameApi, GameInfo, Disciple, Task, MapData, VersionInfo, PillInventory } from './api/gameApi';
import MapView from './MapView';
import APP_CONFIG from './config';
import './App.css';

function App() {
  const [gameId, setGameId] = useState<string | null>(
    localStorage.getItem('gameId')
  );
  const [gameInfo, setGameInfo] = useState<GameInfo | null>(null);
  const [disciples, setDisciples] = useState<Disciple[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [mapData, setMapData] = useState<MapData | null>(null);
  const [serverVersion, setServerVersion] = useState<VersionInfo | null>(null);
  const [pillInventory, setPillInventory] = useState<PillInventory | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showMap, setShowMap] = useState(false);
  const [showPills, setShowPills] = useState(false);

  useEffect(() => {
    // Fetch server version on mount
    gameApi.getVersion().then(setServerVersion).catch(console.error);
  }, []);

  useEffect(() => {
    if (gameId) {
      loadGameData(gameId);
    }
  }, [gameId]);

  const loadGameData = async (id: string) => {
    try {
      setLoading(true);
      const [info, disciplesList, tasksList, map, pills] = await Promise.all([
        gameApi.getGame(id),
        gameApi.getDisciples(id),
        gameApi.getTasks(id),
        gameApi.getMap(id),
        gameApi.getPillInventory(id)
      ]);
      setGameInfo(info);
      setDisciples(disciplesList);
      setTasks(tasksList);
      setMapData(map);
      setPillInventory(pills);
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

  const nextTurn = async () => {
    if (!gameId) return;
    try {
      setLoading(true);
      await gameApi.nextTurn(gameId);
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

  const resetGame = () => {
    const confirmed = window.confirm('确定要重置当前游戏吗？所有进度将被清除！');
    if (confirmed) {
      localStorage.removeItem('gameId');
      setGameId(null);
      setGameInfo(null);
      setDisciples([]);
      setTasks([]);
      setMapData(null);
      setError(null);
    }
  };

  const givePillToDisciple = async (discipleId: number, pillType: string) => {
    if (!gameId) return;
    try {
      const result = await gameApi.usePill(gameId, discipleId, pillType);
      alert(result.message + `\n精力: ${result.energy_before} → ${result.energy_after}\n体魄: ${result.constitution_before} → ${result.constitution_after}`);
      await loadGameData(gameId);
    } catch (err: any) {
      setError(err.message);
      alert('服用丹药失败: ' + err.message);
    }
  };

  if (loading) {
    return <div className="loading">加载中...</div>;
  }

  if (!gameId || !gameInfo) {
    return (
      <div className="App">
        {APP_CONFIG.SHOW_VERSION && (
          <div className="version-badge">
            <div className="version-item">
              <span className="version-label">前端</span>
              <span className="version-value">v{APP_CONFIG.VERSION}</span>
            </div>
            {serverVersion && (
              <div className="version-item">
                <span className="version-label">后端</span>
                <span className="version-value">v{serverVersion.api_version}</span>
              </div>
            )}
          </div>
        )}
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
      {APP_CONFIG.SHOW_VERSION && (
        <div className="version-badge">
          <div className="version-item">
            <span className="version-label">前端</span>
            <span className="version-value">v{APP_CONFIG.VERSION}</span>
          </div>
          {serverVersion && (
            <div className="version-item">
              <span className="version-label">后端</span>
              <span className="version-value">v{serverVersion.api_version}</span>
            </div>
          )}
        </div>
      )}
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
        <button onClick={nextTurn} className="btn-primary">下一回合</button>
        <button onClick={autoAssign} className="btn-secondary">自动分配任务</button>
        <button onClick={() => setShowMap(!showMap)} className="btn-primary">
          {showMap ? '隐藏地图' : '显示地图'}
        </button>
        <button onClick={resetGame} className="btn-warning">重置游戏</button>
        <button onClick={() => setShowPills(!showPills)} className="btn-secondary">
          {showPills ? '隐藏丹药' : '丹药库存'}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {showMap && mapData && (
        <div style={{ padding: '2rem', maxWidth: '1400px', margin: '0 auto' }}>
          <MapView mapData={mapData} />
        </div>
      )}

      {showPills && pillInventory && (
        <div style={{ padding: '1rem', maxWidth: '1200px', margin: '0 auto' }}>
          <h2 style={{ marginBottom: '1rem' }}>丹药库存</h2>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))', gap: '1rem' }}>
            {Object.entries(pillInventory.pills).map(([pillType, info]) => (
              <div key={pillType} style={{
                border: '1px solid #ddd',
                borderRadius: '8px',
                padding: '1rem',
                backgroundColor: info.count > 0 ? '#f9f9f9' : '#eee'
              }}>
                <div style={{ fontSize: '1.2rem', fontWeight: 'bold', marginBottom: '0.5rem' }}>
                  {info.name} <span style={{ color: info.count > 0 ? '#2d7a3e' : '#999' }}>×{info.count}</span>
                </div>
                <div style={{ fontSize: '0.9rem', color: '#666', marginBottom: '0.5rem' }}>
                  {info.description}
                </div>
                <div style={{ fontSize: '0.85rem', color: '#888' }}>
                  {info.energy_restore > 0 && <div>恢复精力: +{info.energy_restore}</div>}
                  {info.constitution_restore > 0 && <div>恢复体魄: +{info.constitution_restore}</div>}
                </div>
              </div>
            ))}
          </div>
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
                    <span className="value">{d.cultivation.level} {d.cultivation.sub_level}</span>
                  </div>

                  <div className="info-row">
                    <span className="label">小境界进度:</span>
                    <span className="value">{d.cultivation.progress}%</span>
                  </div>

                  <div className="progress-bar">
                    <div className="progress-fill" style={{width: `${d.cultivation.progress}%`}}></div>
                  </div>

                  {d.cultivation.cultivation_path && d.cultivation.cultivation_path.total_required > 0 && (
                    <div className="cultivation-path">
                      <div className="path-header">
                        <span className="label">🔮 修炼路径:</span>
                        <span className="value">
                          {d.cultivation.cultivation_path.total_completed}/{d.cultivation.cultivation_path.total_required}
                        </span>
                      </div>
                      <div className="progress-bar">
                        <div
                          className="progress-fill"
                          style={{
                            width: `${(d.cultivation.cultivation_path.total_completed / d.cultivation.cultivation_path.total_required) * 100}%`,
                            background: 'linear-gradient(90deg, #f6ad55 0%, #ed8936 100%)'
                          }}
                        ></div>
                      </div>
                      <div className="path-tasks">
                        {Object.entries(d.cultivation.cultivation_path.required).map(([taskType, required]) => {
                          const completed = d.cultivation.cultivation_path!.completed[taskType] || 0;
                          const isCompleted = completed >= required;
                          const taskTypeNames: {[key: string]: string} = {
                            'Combat': '战斗',
                            'Exploration': '探索',
                            'Gathering': '采集',
                            'Auxiliary': '辅助',
                            'Investment': '投资'
                          };
                          return (
                            <div key={taskType} className={`path-task-item ${isCompleted ? 'completed' : ''}`}>
                              {isCompleted ? '✓' : '○'} {taskTypeNames[taskType] || taskType}: {completed}/{required}
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  )}

                  <div className="info-row">
                    <span className="label">道心:</span>
                    <span className="value">{d.dao_heart}/100</span>
                  </div>

                  <div className="info-row">
                    <span className="label">精力:</span>
                    <span className="value" style={{color: d.energy < 20 ? '#e53e3e' : d.energy < 50 ? '#dd6b20' : '#48bb78'}}>
                      {d.energy}/100
                    </span>
                  </div>
                  <div className="progress-bar">
                    <div className="progress-fill" style={{
                      width: `${d.energy}%`,
                      background: d.energy < 20 ? '#e53e3e' : d.energy < 50 ? '#dd6b20' : '#48bb78'
                    }}></div>
                  </div>

                  <div className="info-row">
                    <span className="label">体魄:</span>
                    <span className="value" style={{color: d.constitution < 20 ? '#e53e3e' : d.constitution < 50 ? '#dd6b20' : '#48bb78'}}>
                      {d.constitution}/100
                    </span>
                  </div>
                  <div className="progress-bar">
                    <div className="progress-fill" style={{
                      width: `${d.constitution}%`,
                      background: d.constitution < 20 ? '#e53e3e' : d.constitution < 50 ? '#dd6b20' : '#48bb78'
                    }}></div>
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

                  {pillInventory && (d.energy < 100 || d.constitution < 100) && (
                    <div style={{ marginTop: '1rem', paddingTop: '1rem', borderTop: '1px solid #eee' }}>
                      <div style={{ fontSize: '0.9rem', fontWeight: 'bold', marginBottom: '0.5rem' }}>服用丹药</div>
                      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
                        {Object.entries(pillInventory.pills)
                          .filter(([_, info]) => info.count > 0)
                          .map(([pillType, info]) => (
                            <button
                              key={pillType}
                              onClick={() => givePillToDisciple(d.id, pillType)}
                              className="btn-small"
                              style={{
                                fontSize: '0.8rem',
                                padding: '0.3rem 0.6rem'
                              }}
                              title={info.description}
                            >
                              {info.name}
                            </button>
                          ))}
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
                <div className="task-costs" style={{marginTop: '0.5rem', fontSize: '0.85rem', color: '#888'}}>
                  <span>消耗: 精力 {t.energy_cost}/回合 | 体魄 {t.constitution_cost}/回合</span>
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
