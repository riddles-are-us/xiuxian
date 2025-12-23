import React, { useState, useEffect } from 'react';
import { gameApi, GameInfo, Disciple, Task, MapData, VersionInfo, PillInventory } from './api/gameApi';
import FullscreenMapView from './FullscreenMapView';
import BuildingTree from './BuildingTree';
import AlchemyPanel from './AlchemyPanel';
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
  const [showBuildings, setShowBuildings] = useState(false);
  const [showAlchemy, setShowAlchemy] = useState(false);
  const [notifications, setNotifications] = useState<Array<{id: number, message: string, type: string}>>([]);
  const [pendingRecruitment, setPendingRecruitment] = useState<Disciple | null>(null);
  const [mapPosition, setMapPosition] = useState({ x: 0, y: 0 }); // 地图位置状态提升，避免loading时重置

  useEffect(() => {
    // Fetch server version on mount
    gameApi.getVersion().then(setServerVersion).catch(console.error);
  }, []);

  useEffect(() => {
    if (gameId) {
      loadGameData(gameId);
      setShowMap(true); // 自动显示地图
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

  const addNotification = (message: string, type: string = 'success') => {
    const id = Date.now();
    setNotifications(prev => [...prev, {id, message, type}]);
    setTimeout(() => {
      setNotifications(prev => prev.filter(n => n.id !== id));
    }, 5000);
  };

  const nextTurn = async () => {
    if (!gameId) return;
    try {
      setLoading(true);
      // 记录当前任务，用于查找任务名称
      const currentTasksMap = new Map(tasks.map(t => [t.id, t]));

      const turnResult = await gameApi.nextTurn(gameId);

      // 检查是否有待招募弟子
      if (turnResult.pending_recruitment) {
        setPendingRecruitment(turnResult.pending_recruitment);
      }

      // 显示任务执行结果通知
      turnResult.task_results.forEach(result => {
        const task = currentTasksMap.get(result.task_id);
        const taskName = task?.name || '未知任务';
        const discipleName = result.disciple_name || disciples.find(d => d.id === result.disciple_id)?.name || '弟子';

        if (result.disciple_died) {
          // 弟子死亡通知
          addNotification(
            `💀 ${discipleName} 在执行任务「${taskName}」时陨落`,
            'error'
          );
        } else if (result.success) {
          addNotification(
            `✅ ${discipleName} 完成了任务「${taskName}」！获得修为+${result.rewards?.progress || 0}`,
            'success'
          );
        } else {
          addNotification(
            `❌ ${discipleName} 执行任务「${taskName}」失败`,
            'error'
          );
        }
      });

      await loadGameData(gameId);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleRecruitment = async (accept: boolean) => {
    if (!gameId || !pendingRecruitment) return;
    try {
      setLoading(true);
      const result = await gameApi.recruitDisciple(gameId, accept);

      if (accept && result.success) {
        addNotification(
          `✅ ${result.message}！消耗资源${result.cost}`,
          'success'
        );
        await loadGameData(gameId);
      } else if (!accept) {
        addNotification('已拒绝招募', 'info');
      }

      setPendingRecruitment(null);
    } catch (err: any) {
      const errorMsg = err.response?.data?.error?.message || err.message || '招募失败';
      setError(errorMsg);
      addNotification(`❌ 招募失败：${errorMsg}`, 'error');
      // 即使失败也关闭招募对话框，让用户能看到错误通知
      setPendingRecruitment(null);
    } finally {
      setLoading(false);
    }
  };

  const refreshTasksAndDisciples = async () => {
    if (!gameId) return;
    try {
      const [disciplesList, tasksList] = await Promise.all([
        gameApi.getDisciples(gameId),
        gameApi.getTasks(gameId)
      ]);
      setDisciples(disciplesList);
      setTasks(tasksList);
    } catch (err: any) {
      console.error('Failed to refresh tasks and disciples:', err);
    }
  };

  const refreshDisciplesAndMap = async (_movedDiscipleId?: number): Promise<Disciple[]> => {
    if (!gameId) return [];
    try {
      const [disciplesList, map] = await Promise.all([
        gameApi.getDisciples(gameId),
        gameApi.getMap(gameId)
      ]);
      setDisciples(disciplesList);
      setMapData(map);
      return disciplesList;
    } catch (err: any) {
      console.error('Failed to refresh disciples and map:', err);
      return [];
    }
  };

  const assignTask = async (taskId: number, discipleId: number) => {
    if (!gameId) return;
    try {
      await gameApi.assignTask(gameId, taskId, discipleId);
      await refreshTasksAndDisciples();
      addNotification('✅ 任务分配成功', 'success');
    } catch (err: any) {
      const errorMsg = err.response?.data?.error?.message || err.message;
      setError(errorMsg);
      addNotification(`❌ ${errorMsg}`, 'error');
    }
  };

  const autoAssign = async () => {
    if (!gameId) return;
    try {
      setLoading(true);
      await gameApi.autoAssignTasks(gameId);
      await refreshTasksAndDisciples();
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

  // 如果有地图数据，显示全屏地图视图
  if (mapData && showMap) {
    return (
      <>
        {/* 通知区域 */}
        {notifications.length > 0 && (
          <div style={{
            position: 'fixed',
            top: '80px',
            right: '20px',
            zIndex: 9999,
            display: 'flex',
            flexDirection: 'column',
            gap: '10px',
            maxWidth: '400px'
          }}>
            {notifications.map(notif => (
              <div key={notif.id} style={{
                background: notif.type === 'success' ? 'linear-gradient(135deg, #48bb78, #38a169)' : '#f56565',
                color: 'white',
                padding: '1rem 1.5rem',
                borderRadius: '8px',
                boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
                animation: 'slideIn 0.3s ease-out',
                fontSize: '0.95rem',
                fontWeight: '500'
              }}>
                {notif.message}
              </div>
            ))}
          </div>
        )}

        {/* 招募弟子模态框 */}
        {pendingRecruitment && (
          <div style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0,0,0,0.7)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10000
          }}>
            <div style={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              borderRadius: '16px',
              padding: '2rem',
              maxWidth: '500px',
              width: '90%',
              boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
              border: '2px solid rgba(255,255,255,0.2)'
            }}>
              <h2 style={{ margin: '0 0 1.5rem 0', color: 'white', fontSize: '1.8rem', textAlign: 'center' }}>
                ⭐ 新弟子求入门
              </h2>
              <div style={{
                background: 'rgba(255,255,255,0.95)',
                borderRadius: '12px',
                padding: '1.5rem',
                marginBottom: '1.5rem'
              }}>
                <h3 style={{ margin: '0 0 1rem 0', color: '#333', fontSize: '1.4rem' }}>{pendingRecruitment.name}</h3>
                <div style={{ color: '#666', lineHeight: '1.8' }}>
                  <p><strong>类型:</strong> {pendingRecruitment.disciple_type}</p>
                  <p><strong>修为:</strong> {pendingRecruitment.cultivation.level} {pendingRecruitment.cultivation.sub_level}</p>
                  <p><strong>年龄:</strong> {pendingRecruitment.age} 岁</p>
                  <p><strong>天赋:</strong></p>
                  <ul style={{ marginLeft: '1.5rem' }}>
                    {pendingRecruitment.talents.map((t, i) => (
                      <li key={i}>{t.talent_type} Lv.{t.level}</li>
                    ))}
                  </ul>
                </div>
              </div>
              <div style={{ display: 'flex', gap: '1rem' }}>
                <button onClick={() => handleRecruitment(false)} style={{
                  flex: 1,
                  padding: '0.8rem',
                  background: '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  fontSize: '1.1rem',
                  cursor: 'pointer'
                }}>
                  ❌ 拒绝
                </button>
                <button onClick={() => handleRecruitment(true)} style={{
                  flex: 1,
                  padding: '0.8rem',
                  background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  fontSize: '1.1rem',
                  cursor: 'pointer'
                }}>
                  ✅ 接受招募
                </button>
              </div>
            </div>
          </div>
        )}

        <FullscreenMapView
          mapData={mapData}
          disciples={disciples}
          tasks={tasks}
          gameInfo={gameInfo}
          gameId={gameId}
          onDiscipleMoved={refreshDisciplesAndMap}
          onTaskAssigned={refreshTasksAndDisciples}
          onAutoAssign={autoAssign}
          onNextTurn={nextTurn}
          onResetGame={resetGame}
          mapPosition={mapPosition}
          onMapPositionChange={setMapPosition}
        />
      </>
    );
  }

  return (
    <div className="App">
      {/* 通知区域 */}
      {notifications.length > 0 && (
        <div style={{
          position: 'fixed',
          top: '20px',
          right: '20px',
          zIndex: 9999,
          display: 'flex',
          flexDirection: 'column',
          gap: '10px',
          maxWidth: '400px'
        }}>
          {notifications.map(notif => (
            <div key={notif.id} style={{
              background: notif.type === 'success' ? 'linear-gradient(135deg, #48bb78, #38a169)' : '#f56565',
              color: 'white',
              padding: '1rem 1.5rem',
              borderRadius: '8px',
              boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
              animation: 'slideIn 0.3s ease-out',
              fontSize: '0.95rem',
              fontWeight: '500'
            }}>
              {notif.message}
            </div>
          ))}
        </div>
      )}

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
        <button onClick={() => setShowBuildings(!showBuildings)} className="btn-primary">
          {showBuildings ? '隐藏建筑' : '宗门建筑'}
        </button>
        <button onClick={() => setShowPills(!showPills)} className="btn-secondary">
          {showPills ? '隐藏丹药' : '丹药库存'}
        </button>
        <button onClick={() => setShowAlchemy(!showAlchemy)} className="btn-primary">
          {showAlchemy ? '隐藏炼丹' : '炼丹炉'}
        </button>
        <button onClick={resetGame} className="btn-warning">重置游戏</button>
      </div>

      {error && <div className="error">{error}</div>}

      {/* 招募弟子模态框 */}
      {pendingRecruitment && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0,0,0,0.7)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 1000
        }}>
          <div style={{
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            borderRadius: '16px',
            padding: '2rem',
            maxWidth: '500px',
            width: '90%',
            boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
            border: '2px solid rgba(255,255,255,0.2)'
          }}>
            <h2 style={{ margin: '0 0 1.5rem 0', color: 'white', fontSize: '1.8rem', textAlign: 'center' }}>
              ⭐ 新弟子求入门
            </h2>

            <div style={{
              background: 'rgba(255,255,255,0.95)',
              borderRadius: '12px',
              padding: '1.5rem',
              marginBottom: '1.5rem'
            }}>
              <h3 style={{ margin: '0 0 1rem 0', color: '#333', fontSize: '1.4rem' }}>{pendingRecruitment.name}</h3>
              <div style={{ color: '#666', lineHeight: '1.8' }}>
                <p><strong>类型:</strong> {pendingRecruitment.disciple_type}</p>
                <p><strong>修为:</strong> {pendingRecruitment.cultivation.level} {pendingRecruitment.cultivation.sub_level}</p>
                <p><strong>年龄:</strong> {pendingRecruitment.age} 岁 (寿元 {pendingRecruitment.lifespan})</p>
                <p><strong>道心:</strong> {pendingRecruitment.dao_heart}</p>
                <p><strong>天赋:</strong></p>
                <ul style={{ marginLeft: '1.5rem', marginTop: '0.5rem' }}>
                  {pendingRecruitment.talents.map((t, i) => (
                    <li key={i}>{t.talent_type} Lv.{t.level}</li>
                  ))}
                </ul>
              </div>
            </div>

            <div style={{
              background: 'rgba(255,255,255,0.9)',
              borderRadius: '8px',
              padding: '1rem',
              marginBottom: '1.5rem',
              textAlign: 'center',
              color: '#d9534f',
              fontSize: '1.1rem',
              fontWeight: 'bold'
            }}>
              💰 招募费用: 1000 资源
            </div>

            <div style={{ display: 'flex', gap: '1rem' }}>
              <button
                onClick={() => handleRecruitment(false)}
                style={{
                  flex: 1,
                  padding: '0.8rem',
                  background: '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  fontSize: '1.1rem',
                  cursor: 'pointer',
                  transition: 'all 0.3s'
                }}
                onMouseEnter={(e) => e.currentTarget.style.background = '#5a6268'}
                onMouseLeave={(e) => e.currentTarget.style.background = '#6c757d'}
              >
                ❌ 拒绝
              </button>
              <button
                onClick={() => handleRecruitment(true)}
                style={{
                  flex: 1,
                  padding: '0.8rem',
                  background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                  color: 'white',
                  border: 'none',
                  borderRadius: '8px',
                  fontSize: '1.1rem',
                  cursor: 'pointer',
                  transition: 'all 0.3s',
                  boxShadow: '0 4px 15px rgba(102, 126, 234, 0.4)'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-2px)';
                  e.currentTarget.style.boxShadow = '0 6px 20px rgba(102, 126, 234, 0.6)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = '0 4px 15px rgba(102, 126, 234, 0.4)';
                }}
              >
                ✅ 接受招募
              </button>
            </div>
          </div>
        </div>
      )}


      {showBuildings && gameId && (
        <div style={{ padding: '1rem', maxWidth: '1400px', margin: '0 auto' }}>
          <BuildingTree
            gameId={gameId}
            onResourcesChanged={() => gameId && loadGameData(gameId)}
          />
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

      {showAlchemy && gameId && (
        <AlchemyPanel
          gameId={gameId}
          onRefineSuccess={() => {
            loadGameData(gameId);
          }}
        />
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

                  {d.relationship_summary?.dao_companion_id && (
                    <div className="companion-section">
                      <span className="companion-badge">
                        💑 道侣: {disciples.find(x => x.id === d.relationship_summary.dao_companion_id)?.name || '未知'}
                      </span>
                    </div>
                  )}
                  {d.relationship_summary?.master_id && (
                    <div className="master-section">
                      <span className="master-badge">
                        👨‍🏫 师父: {disciples.find(x => x.id === d.relationship_summary.master_id)?.name || '未知'}
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
            {tasks.map(t => {
              const isDefenseTask = t.name.includes('守卫');
              return (
              <div key={t.id} className="task-card" style={{
                border: isDefenseTask ? '2px solid #f56565' : undefined,
                background: isDefenseTask ? 'linear-gradient(135deg, #fff5f5 0%, #fed7d7 100%)' : undefined
              }}>
                <div className="task-header">
                  <h3>
                    {isDefenseTask && '🛡️ '}
                    {t.name}
                    {isDefenseTask && <span style={{
                      marginLeft: '0.5rem',
                      fontSize: '0.75rem',
                      background: '#f56565',
                      color: 'white',
                      padding: '0.125rem 0.5rem',
                      borderRadius: '4px',
                      fontWeight: 'bold'
                    }}>紧急</span>}
                  </h3>
                  <span className={`task-expiry ${t.remaining_turns <= 2 ? 'urgent' : ''}`}>
                    ⏰ {t.remaining_turns}回合后失效
                  </span>
                </div>
                {t.enemy_info && (
                  <div style={{
                    background: '#fff3cd',
                    border: '1px solid #ffc107',
                    borderRadius: '6px',
                    padding: '8px 12px',
                    marginBottom: '8px',
                    fontSize: '0.9rem'
                  }}>
                    <strong style={{color: '#856404'}}>⚔️ 入侵者:</strong>{' '}
                    <span style={{color: '#d9534f', fontWeight: 600}}>{t.enemy_info.enemy_name}</span>
                    {' '}
                    <span style={{color: '#666'}}>
                      (Lv.{t.enemy_info.enemy_level} | ID: {t.enemy_info.enemy_id})
                    </span>
                  </div>
                )}
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
                  {t.max_participants > 1 && (
                    <span style={{marginLeft: '1rem', color: '#667eea'}}>
                      👥 最多 {t.max_participants} 人
                    </span>
                  )}
                </div>
                {t.assigned_to.length > 0 ? (
                  <div>
                    <p className="assigned">
                      ✓ 已分配给 {t.assigned_to.map(id => disciples.find(d => d.id === id)?.name).filter(Boolean).join('、')}
                      {t.max_participants > 1 && (
                        <span style={{marginLeft: '0.5rem', color: '#888', fontSize: '0.85rem'}}>
                          ({t.assigned_to.length}/{t.max_participants})
                        </span>
                      )}
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
                    {/* 如果还可以添加更多弟子 */}
                    {t.assigned_to.length < t.max_participants && t.suitable_disciples && t.suitable_disciples.free.length > 0 && (
                      <div style={{marginTop: '0.5rem', paddingTop: '0.5rem', borderTop: '1px dashed #ddd'}}>
                        <div style={{fontSize: '0.8rem', color: '#667eea', marginBottom: '0.3rem'}}>
                          ➕ 添加更多弟子 (还可加 {t.max_participants - t.assigned_to.length} 人):
                        </div>
                        <div className="assign-buttons">
                          {disciples
                            .filter(d => t.suitable_disciples.free.includes(d.id) && !t.assigned_to.includes(d.id))
                            .map(d => (
                              <button
                                key={d.id}
                                onClick={() => assignTask(t.id, d.id)}
                                className="btn-small"
                                title={`${d.name} - ${d.cultivation.level} ${d.cultivation.sub_level}`}
                              >
                                + {d.name}
                              </button>
                            ))
                          }
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="assign-section">
                    {t.suitable_disciples && (t.suitable_disciples.free.length > 0 || t.suitable_disciples.busy.length > 0) ? (
                      <>
                        {/* 合适的统计信息 */}
                        <div style={{
                          fontSize: '0.85rem',
                          color: '#666',
                          marginBottom: '0.5rem',
                          paddingBottom: '0.5rem',
                          borderBottom: '1px solid #eee'
                        }}>
                          合适弟子: <span style={{color: '#48bb78', fontWeight: 'bold'}}>{t.suitable_disciples.free.length} 空闲</span>
                          {t.suitable_disciples.busy.length > 0 && (
                            <>, <span style={{color: '#ed8936', fontWeight: 'bold'}}>{t.suitable_disciples.busy.length} 忙碌</span></>
                          )}
                        </div>

                        {/* 空闲的合适弟子 */}
                        {t.suitable_disciples.free.length > 0 && (
                          <div className="assign-buttons">
                            {disciples
                              .filter(d => t.suitable_disciples.free.includes(d.id))
                              .map(d => (
                                <button
                                  key={d.id}
                                  onClick={() => assignTask(t.id, d.id)}
                                  className="btn-small"
                                  title={`${d.name} - ${d.cultivation.level} ${d.cultivation.sub_level}`}
                                >
                                  ✓ {d.name}
                                </button>
                              ))
                            }
                          </div>
                        )}

                        {/* 忙碌的合适弟子 */}
                        {t.suitable_disciples.busy.length > 0 && (
                          <div style={{marginTop: '0.5rem'}}>
                            <div style={{fontSize: '0.8rem', color: '#999', marginBottom: '0.3rem'}}>
                              忙碌中:
                            </div>
                            <div style={{display: 'flex', flexWrap: 'wrap', gap: '0.3rem'}}>
                              {disciples
                                .filter(d => t.suitable_disciples.busy.includes(d.id))
                                .map(d => (
                                  <span
                                    key={d.id}
                                    style={{
                                      fontSize: '0.75rem',
                                      padding: '0.2rem 0.5rem',
                                      background: '#f7fafc',
                                      color: '#718096',
                                      borderRadius: '4px',
                                      border: '1px solid #e2e8f0'
                                    }}
                                    title={`${d.name} 正在执行其他任务`}
                                  >
                                    ⏳ {d.name}
                                  </span>
                                ))
                              }
                            </div>
                          </div>
                        )}
                      </>
                    ) : (
                      <div style={{color: '#999', fontSize: '0.9rem'}}>
                        {t.skill_required ? `需要技能: ${t.skill_required}` : '暂无适合的弟子'}
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
