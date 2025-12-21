import React, { useState, useRef, useEffect } from 'react';
import { MapData, MapElement, Disciple, Task, GameInfo, gameApi, Relationship } from './api/gameApi';
import MapView from './MapView';
import { getElementIcon, renderElementDetails } from './MapElementDetails';
import BuildingTree from './BuildingTree';
import './FullscreenMapView.css';

interface FullscreenMapViewProps {
  mapData: MapData;
  disciples: Disciple[];
  tasks: Task[];
  gameInfo: GameInfo;
  gameId: string;
  onDiscipleMoved: (movedDiscipleId: number) => Promise<Disciple[]>;  // 返回刷新后的弟子列表
  onTaskAssigned: () => void;
  onAutoAssign: () => void;
  onNextTurn: () => void;
  onResetGame: () => void;
  mapPosition: { x: number; y: number };
  onMapPositionChange: (pos: { x: number; y: number }) => void;
}

type PanelType = 'disciples' | 'tasks' | 'mapinfo' | 'buildings' | null;

const FullscreenMapView: React.FC<FullscreenMapViewProps> = ({
  mapData,
  disciples,
  tasks,
  gameInfo,
  gameId,
  onDiscipleMoved,
  onTaskAssigned,
  onAutoAssign,
  onNextTurn,
  onResetGame,
  mapPosition,
  onMapPositionChange
}) => {
  const [activePanel, setActivePanel] = useState<PanelType>(null);
  const [panelTab, setPanelTab] = useState<'disciples' | 'tasks' | 'mapinfo' | 'buildings'>('disciples');

  // 地图信息状态
  const [selectedElement, setSelectedElement] = useState<MapElement | null>(null);
  const [selectedMapDisciple, setSelectedMapDisciple] = useState<Disciple | null>(null);
  const [moveError, setMoveError] = useState<string | null>(null);
  const [discipleRelationships, setDiscipleRelationships] = useState<Relationship[]>([]);
  const [showRelationships, setShowRelationships] = useState(false);

  // 当 disciples 数据更新时，同步更新选中的弟子状态
  useEffect(() => {
    if (selectedMapDisciple) {
      const updatedDisciple = disciples.find(d => d.id === selectedMapDisciple.id);
      if (updatedDisciple) {
        setSelectedMapDisciple(updatedDisciple);
      }
    }
  }, [disciples]);

  // 当选中弟子变化时，加载其关系数据
  useEffect(() => {
    if (selectedMapDisciple) {
      gameApi.getDiscipleRelationships(gameId, selectedMapDisciple.id)
        .then(setDiscipleRelationships)
        .catch(() => setDiscipleRelationships([]));
    } else {
      setDiscipleRelationships([]);
      setShowRelationships(false);
    }
  }, [selectedMapDisciple, gameId]);

  // 地图拖拽平移状态 - 使用 transform 而不是 scroll
  const mapContainerRef = useRef<HTMLDivElement>(null);
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  // mapPosition 现在由父组件管理，不再使用本地 state
  const savedMapPosition = useRef({ x: 0, y: 0 }); // 用于拖拽开始时保存位置

  const togglePanel = (panel: PanelType) => {
    if (activePanel === panel) {
      setActivePanel(null);
    } else {
      setActivePanel(panel);
      if (panel) {
        setPanelTab(panel);
      }
    }
  };

  // 地图拖拽处理 - 在地图网格上拖拽
  const handleMapMouseDown = (e: React.MouseEvent) => {
    // 只在左键点击时开始拖拽
    if (e.button !== 0) return;

    // 阻止事件冒泡，避免触发地图格子的点击事件
    e.stopPropagation();

    setIsPanning(true);
    setPanStart({ x: e.clientX, y: e.clientY });
    savedMapPosition.current = { ...mapPosition };
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isPanning) return;

      const deltaX = e.clientX - panStart.x;
      const deltaY = e.clientY - panStart.y;

      onMapPositionChange({
        x: savedMapPosition.current.x + deltaX,
        y: savedMapPosition.current.y + deltaY
      });
    };

    const handleMouseUp = () => {
      if (isPanning) {
        setIsPanning(false);
      }
    };

    if (isPanning) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isPanning, panStart, onMapPositionChange]);

  const assignTask = async (taskId: number, discipleId: number) => {
    try {
      await gameApi.assignTask(gameId, taskId, discipleId);
      onTaskAssigned();
    } catch (error: any) {
      alert(error.response?.data?.error?.message || '分配失败');
    }
  };

  const unassignTask = async (taskId: number) => {
    try {
      await gameApi.unassignTask(gameId, taskId);
      onTaskAssigned();
    } catch (error: any) {
      alert(error.response?.data?.error?.message || '取消分配失败');
    }
  };

  // 处理元素选择 - 自动打开地图信息面板
  const handleElementSelected = (element: MapElement | null) => {
    setSelectedElement(element);
    if (element) {
      setActivePanel('mapinfo');
      setPanelTab('mapinfo');
    }
  };

  // 处理弟子选择 - 自动打开地图信息面板
  const handleDiscipleSelected = (disciple: Disciple | null) => {
    setSelectedMapDisciple(disciple);
    if (disciple) {
      setActivePanel('mapinfo');
      setPanelTab('mapinfo');
    }
  };

  // 处理弟子移动 - 刷新数据并重新选中弟子
  const handleDiscipleMoved = async (movedDiscipleId: number) => {
    const updatedDisciples = await onDiscipleMoved(movedDiscipleId);
    // 从刷新后的数据中找到移动的弟子并重新选中
    const movedDisciple = updatedDisciples.find(d => d.id === movedDiscipleId);
    if (movedDisciple) {
      setSelectedMapDisciple(movedDisciple);
      setActivePanel('mapinfo');
      setPanelTab('mapinfo');
    }
  };

  // 处理任务点击 - 聚焦到任务位置（使用 transform）
  const handleTaskClick = (task: Task) => {
    console.log('=== handleTaskClick called ===');
    console.log('Task:', task);

    if (!task.position) {
      console.log('Task has no position:', task);
      return;
    }

    if (!mapContainerRef.current) {
      console.log('mapContainerRef not available');
      return;
    }

    const { x, y } = task.position;
    const tileSize = 50; // 每个格子的大小
    const gap = 2; // 格子间隙
    const tileTotalSize = tileSize + gap;

    // 计算目标位置（格子左上角）
    const targetLeft = x * tileTotalSize;
    const targetTop = y * tileTotalSize;

    // 获取容器尺寸
    const containerWidth = mapContainerRef.current.clientWidth;
    const containerHeight = mapContainerRef.current.clientHeight;

    // 计算地图位置，使目标格子位于视口中心
    // 注意：transform 的正值是向右/向下移动，所以要取反
    const newX = (containerWidth / 2) - targetLeft - (tileSize / 2);
    const newY = (containerHeight / 2) - targetTop - (tileSize / 2);

    console.log('Focusing on task:', task.name, 'at position:', { x, y });
    console.log('Setting map position to:', { x: newX, y: newY });

    onMapPositionChange({ x: newX, y: newY });

    // 选中该位置的元素并打开地图信息面板
    const element = mapData.elements.find(
      el => el.position.x === x && el.position.y === y
    );
    if (element) {
      handleElementSelected(element);
    }
  };

  return (
    <div className="fullscreen-map-container">
      {/* 顶部信息栏 */}
      <div className="top-bar">
        <div className="top-bar-left">
          <div className="sect-name">{gameInfo.sect.name}</div>
          <div className="sect-stats">
            <div className="stat-item">
              <span className="stat-label">年份:</span>
              <span className="stat-value">{gameInfo.sect.year}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">💰</span>
              <span className="stat-value">{gameInfo.sect.resources}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">⭐</span>
              <span className="stat-value">{gameInfo.sect.reputation}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">👥</span>
              <span className="stat-value">{gameInfo.sect.disciples_count}</span>
            </div>
          </div>
        </div>
      </div>

      {/* 主要内容区域 */}
      <div className="main-content">
        {/* 地图区域 */}
        <div className="map-area">
          <div
            ref={mapContainerRef}
            className="map-wrapper"
            style={{
              overflow: 'hidden',
              position: 'relative'
            }}
          >
            <MapView
              mapData={mapData}
              disciples={disciples}
              gameId={gameId}
              onDiscipleMoved={handleDiscipleMoved}
              onElementSelected={handleElementSelected}
              onDiscipleSelected={handleDiscipleSelected}
              onMoveError={setMoveError}
              transform={mapPosition}
              onMapMouseDown={handleMapMouseDown}
              isPanning={isPanning}
            />
          </div>

          {/* 移动错误提示 */}
          {moveError && (
            <div style={{
              position: 'absolute',
              top: '20px',
              left: '50%',
              transform: 'translateX(-50%)',
              backgroundColor: '#fed7d7',
              color: '#c53030',
              padding: '12px 16px',
              borderRadius: '8px',
              border: '2px solid #fc8181',
              boxShadow: '0 4px 6px rgba(0,0,0,0.1)',
              zIndex: 1000
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span style={{ fontSize: '18px' }}>⚠️</span>
                <span style={{ fontWeight: 'bold' }}>{moveError}</span>
                <button
                  onClick={() => setMoveError(null)}
                  style={{
                    marginLeft: '8px',
                    background: 'none',
                    border: 'none',
                    color: '#c53030',
                    cursor: 'pointer',
                    fontSize: '16px',
                    fontWeight: 'bold'
                  }}
                >
                  ✕
                </button>
              </div>
            </div>
          )}

          {/* 面板切换按钮 */}
          <div className="panel-toggle-buttons">
            <button
              className={`panel-toggle-btn ${activePanel === 'disciples' ? 'active' : ''}`}
              onClick={() => togglePanel('disciples')}
              title="弟子列表"
            >
              👥
            </button>
            <button
              className={`panel-toggle-btn ${activePanel === 'tasks' ? 'active' : ''}`}
              onClick={() => togglePanel('tasks')}
              title="任务列表"
            >
              📋
            </button>
            <button
              className={`panel-toggle-btn ${activePanel === 'buildings' ? 'active' : ''}`}
              onClick={() => togglePanel('buildings')}
              title="宗门建筑"
            >
              🏛️
            </button>
          </div>
        </div>

        {/* 侧边面板 */}
        <div className={`side-panel ${activePanel === null ? 'collapsed' : ''}`}>
          <div className="panel-header">
            <div className="panel-tabs">
              <button
                className={`panel-tab ${panelTab === 'disciples' ? 'active' : ''}`}
                onClick={() => setPanelTab('disciples')}
              >
                弟子列表 ({disciples.length})
              </button>
              <button
                className={`panel-tab ${panelTab === 'tasks' ? 'active' : ''}`}
                onClick={() => setPanelTab('tasks')}
              >
                任务列表 ({tasks.length})
              </button>
              <button
                className={`panel-tab ${panelTab === 'mapinfo' ? 'active' : ''}`}
                onClick={() => setPanelTab('mapinfo')}
              >
                地图信息
              </button>
              <button
                className={`panel-tab ${panelTab === 'buildings' ? 'active' : ''}`}
                onClick={() => setPanelTab('buildings')}
              >
                宗门建筑
              </button>
            </div>
            <button className="panel-close" onClick={() => setActivePanel(null)}>
              ✕
            </button>
          </div>

          <div className="panel-content">
            {panelTab === 'disciples' && (
              <div>
                {disciples.map(disciple => (
                  <div key={disciple.id} className="disciple-list-item">
                    <div className="disciple-name">
                      {disciple.name}
                      {disciple.current_task_info && <span style={{marginLeft: '0.5rem', fontSize: '0.9rem'}}>🔨</span>}
                    </div>
                    <div className="disciple-info">
                      <div>修为: {disciple.cultivation.level} {disciple.cultivation.sub_level}</div>
                      <div>位置: ({disciple.position.x}, {disciple.position.y})</div>
                      <div>移动范围: {disciple.movement_range} 格</div>
                      <div>精力: {disciple.energy}/100 | 体魄: {disciple.constitution}/100</div>
                      {disciple.current_task_info && (
                        <div style={{color: '#48bb78', marginTop: '0.5rem'}}>
                          执行任务: {disciple.current_task_info.task_name} ({disciple.current_task_info.progress}/{disciple.current_task_info.duration})
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {panelTab === 'tasks' && (
              <div>
                {tasks.map(task => (
                  <div key={task.id} className={`task-list-item ${task.assigned_to.length > 0 ? 'assigned' : ''}`}>
                    <div
                      className="task-name"
                      onClick={() => handleTaskClick(task)}
                      style={{
                        cursor: task.position ? 'pointer' : 'default',
                        transition: 'color 0.2s'
                      }}
                      onMouseEnter={(e) => {
                        if (task.position) {
                          e.currentTarget.style.color = '#667eea';
                        }
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.color = '';
                      }}
                    >
                      {task.name}
                      {task.assigned_to.length > 0 && <span style={{marginLeft: '0.5rem', fontSize: '0.9rem'}}>✅</span>}
                      {task.position && <span style={{marginLeft: '0.5rem', fontSize: '0.8rem', opacity: 0.7}}>🗺️</span>}
                      {task.max_participants > 1 && (
                        <span style={{marginLeft: '0.5rem', fontSize: '0.75rem', color: '#667eea'}}>
                          👥{task.assigned_to.length}/{task.max_participants}
                        </span>
                      )}
                    </div>
                    <div className="task-info">
                      <div>类型: {task.task_type}</div>
                      <div>奖励: 修为+{task.rewards.progress} 资源+{task.rewards.resources}</div>
                      <div>消耗: 精力-{task.energy_cost} 体魄-{task.constitution_cost}</div>
                      <div>期限: {task.remaining_turns} 回合</div>
                      {task.position && (
                        <div>位置: ({task.position.x}, {task.position.y})</div>
                      )}
                      {task.assigned_to.length > 0 ? (
                        <div style={{marginTop: '0.5rem'}}>
                          <button
                            onClick={() => unassignTask(task.id)}
                            style={{
                              padding: '0.4rem 0.8rem',
                              background: '#ed8936',
                              color: 'white',
                              border: 'none',
                              borderRadius: '4px',
                              cursor: 'pointer',
                              fontSize: '0.85rem'
                            }}
                          >
                            取消全部
                          </button>
                          <span style={{marginLeft: '0.5rem', color: '#48bb78'}}>
                            已分配: {task.assigned_to.map(id => disciples.find(d => d.id === id)?.name).filter(Boolean).join('、')}
                          </span>
                          {/* 如果还能添加更多弟子 */}
                          {task.assigned_to.length < task.max_participants && (() => {
                            const availableDisciples = disciples
                              .filter(d => !d.current_task_info &&
                                          task.suitable_disciples.free.includes(d.id) &&
                                          !task.assigned_to.includes(d.id));
                            if (availableDisciples.length === 0) return null;
                            return (
                              <div style={{marginTop: '0.5rem'}}>
                                <select
                                  onChange={(e) => {
                                    if (e.target.value) {
                                      assignTask(task.id, parseInt(e.target.value));
                                      e.target.value = '';
                                    }
                                  }}
                                  style={{
                                    padding: '0.3rem',
                                    background: 'rgba(102, 126, 234, 0.2)',
                                    color: 'white',
                                    border: '1px solid #667eea',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    fontSize: '0.8rem'
                                  }}
                                >
                                  <option value="">➕ 添加弟子...</option>
                                  {availableDisciples.map(d => (
                                    <option key={d.id} value={d.id} style={{background: '#2a2a40'}}>
                                      {d.name} ({d.cultivation.level})
                                    </option>
                                  ))}
                                </select>
                              </div>
                            );
                          })()}
                        </div>
                      ) : (
                        <div style={{marginTop: '0.5rem'}}>
                          {(() => {
                            const suitableDisciplesFiltered = disciples
                              .filter(d => !d.current_task_info && task.suitable_disciples.free.includes(d.id));

                            if (suitableDisciplesFiltered.length === 0) {
                              return (
                                <div style={{
                                  color: '#ed8936',
                                  fontSize: '0.85rem',
                                  fontStyle: 'italic'
                                }}>
                                  当前无弟子可以胜任此任务
                                </div>
                              );
                            }

                            return (
                              <select
                                onChange={(e) => {
                                  if (e.target.value) {
                                    assignTask(task.id, parseInt(e.target.value));
                                    e.target.value = '';
                                  }
                                }}
                                style={{
                                  padding: '0.4rem',
                                  background: 'rgba(255, 255, 255, 0.1)',
                                  color: 'white',
                                  border: '1px solid rgba(255, 255, 255, 0.2)',
                                  borderRadius: '4px',
                                  cursor: 'pointer',
                                  fontSize: '0.85rem'
                                }}
                              >
                                <option value="">选择弟子...</option>
                                {suitableDisciplesFiltered.map(d => (
                                  <option key={d.id} value={d.id} style={{background: '#2a2a40'}}>
                                    {d.name} ({d.cultivation.level})
                                  </option>
                                ))}
                              </select>
                            );
                          })()}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {panelTab === 'mapinfo' && (
              <div>
                {selectedElement && (
                  <div className="map-info-container">
                    <div className="map-info-header">
                      <span style={{ fontSize: '1.5rem', marginRight: '0.5rem' }}>
                        {getElementIcon(selectedElement.element_type, selectedElement.details)}
                      </span>
                      <span style={{ fontSize: '1.1rem', fontWeight: 'bold' }}>
                        {selectedElement.name}
                      </span>
                    </div>
                    <div className="map-info-details">
                      <div className="detail-row">
                        <span className="detail-label">类型:</span>
                        <span className="detail-value">{selectedElement.element_type}</span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">位置:</span>
                        <span className="detail-value">
                          ({selectedElement.position.x}, {selectedElement.position.y})
                        </span>
                      </div>
                      {renderElementDetails(selectedElement)}

                      {/* 显示该位置的可用任务 */}
                      {(() => {
                        const tasksAtPosition = tasks.filter(t => {
                          // 按位置匹配
                          if (t.position &&
                              t.position.x === selectedElement.position.x &&
                              t.position.y === selectedElement.position.y) {
                            return true;
                          }

                          // 如果是怪物元素，匹配相关的战斗任务（通过 monster_id）
                          if (selectedElement.element_type === 'Monster' &&
                              selectedElement.details.monster_id &&
                              t.enemy_info &&
                              t.enemy_info.enemy_id === selectedElement.details.monster_id) {
                            return true;
                          }

                          return false;
                        });
                        if (tasksAtPosition.length === 0) return null;
                        return (
                          <div style={{
                            marginTop: '12px',
                            padding: '10px',
                            backgroundColor: '#fffaf0',
                            borderRadius: '6px',
                            border: '1px solid #ed8936'
                          }}>
                            <div style={{ fontWeight: 'bold', marginBottom: '8px', color: '#c05621' }}>
                              📋 此位置可接受的任务 ({tasksAtPosition.length})
                            </div>
                            {tasksAtPosition.map(task => (
                              <div key={task.id} style={{
                                padding: '8px',
                                marginBottom: '6px',
                                backgroundColor: 'white',
                                borderRadius: '4px',
                                border: '1px solid #e2e8f0'
                              }}>
                                <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>
                                  <span style={{ color: '#718096', fontWeight: 'normal' }}>任务: </span>
                                  {task.name}
                                  {task.max_participants > 1 && (
                                    <span style={{ marginLeft: '6px', fontSize: '0.8rem', color: '#667eea' }}>
                                      👥 {task.assigned_to.length}/{task.max_participants}
                                    </span>
                                  )}
                                </div>
                                <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                  类型: {task.task_type.split('(')[0]}
                                </div>
                                <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                  奖励: 修为+{task.rewards.progress} 资源+{task.rewards.resources}
                                </div>
                                <div style={{ fontSize: '0.8rem', color: '#888', marginTop: '2px' }}>
                                  ⏱️ 需要 {task.duration} 回合完成 | ⏰ {task.remaining_turns}回合后失效
                                </div>
                                {task.assigned_to.length > 0 && (
                                  <div style={{ fontSize: '0.8rem', color: '#48bb78', marginTop: '4px' }}>
                                    ✓ 已分配: {task.assigned_to.map(id => disciples.find(d => d.id === id)?.name).filter(Boolean).join('、')}
                                  </div>
                                )}
                              </div>
                            ))}
                          </div>
                        );
                      })()}
                    </div>
                  </div>
                )}

                {selectedMapDisciple && (
                  <div className="map-info-container">
                    <div className="map-info-header">
                      <span style={{ fontSize: '1.5rem', marginRight: '0.5rem' }}>🧙</span>
                      <span style={{ fontSize: '1.1rem', fontWeight: 'bold' }}>
                        {selectedMapDisciple.name}
                      </span>
                    </div>
                    <div className="map-info-details">
                      <div className="detail-row">
                        <span className="detail-label">类型:</span>
                        <span className="detail-value">{selectedMapDisciple.disciple_type}</span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">年龄:</span>
                        <span className="detail-value">
                          {selectedMapDisciple.age} 岁 / {selectedMapDisciple.lifespan} 岁
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">修为:</span>
                        <span className="detail-value">
                          {selectedMapDisciple.cultivation.level} {selectedMapDisciple.cultivation.sub_level}
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">修为进度:</span>
                        <span className="detail-value">
                          <span style={{
                            display: 'inline-block',
                            width: '60px',
                            height: '8px',
                            backgroundColor: '#e2e8f0',
                            borderRadius: '4px',
                            marginRight: '6px',
                            verticalAlign: 'middle'
                          }}>
                            <span style={{
                              display: 'block',
                              width: `${selectedMapDisciple.cultivation.progress}%`,
                              height: '100%',
                              backgroundColor: '#667eea',
                              borderRadius: '4px'
                            }}></span>
                          </span>
                          {selectedMapDisciple.cultivation.progress}%
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">道心:</span>
                        <span className="detail-value" style={{
                          color: selectedMapDisciple.dao_heart >= 80 ? '#48bb78' :
                                 selectedMapDisciple.dao_heart >= 50 ? '#ed8936' : '#f56565',
                          fontWeight: 'bold'
                        }}>
                          {selectedMapDisciple.dao_heart}
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">精力:</span>
                        <span className="detail-value" style={{
                          color: selectedMapDisciple.energy >= 70 ? '#48bb78' :
                                 selectedMapDisciple.energy >= 30 ? '#ed8936' : '#f56565'
                        }}>
                          {selectedMapDisciple.energy}/100
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">体魄:</span>
                        <span className="detail-value" style={{
                          color: selectedMapDisciple.constitution >= 70 ? '#48bb78' :
                                 selectedMapDisciple.constitution >= 30 ? '#ed8936' : '#f56565'
                        }}>
                          {selectedMapDisciple.constitution}/100
                        </span>
                      </div>
                      {selectedMapDisciple.talents.length > 0 && (
                        <div className="detail-row">
                          <span className="detail-label">天赋:</span>
                          <span className="detail-value">
                            {selectedMapDisciple.talents.map(t => `${t.talent_type}(${t.level})`).join('、')}
                          </span>
                        </div>
                      )}
                      {selectedMapDisciple.heritage && (
                        <div className="detail-row">
                          <span className="detail-label">传承:</span>
                          <span className="detail-value" style={{ color: '#805ad5' }}>
                            {selectedMapDisciple.heritage.name} ({selectedMapDisciple.heritage.level})
                          </span>
                        </div>
                      )}
                      <div className="detail-row">
                        <span className="detail-label">位置:</span>
                        <span className="detail-value">
                          ({selectedMapDisciple.position.x}, {selectedMapDisciple.position.y})
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">移动范围:</span>
                        <span className="detail-value" style={{
                          color: '#4299e1',
                          fontWeight: 'bold'
                        }}>
                          {selectedMapDisciple.movement_range} 格
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">剩余移动:</span>
                        <span className="detail-value" style={{
                          color: selectedMapDisciple.moves_remaining === 0 ? '#f56565' :
                                 selectedMapDisciple.moves_remaining < selectedMapDisciple.movement_range / 2 ? '#ed8936' : '#48bb78',
                          fontWeight: 'bold'
                        }}>
                          {selectedMapDisciple.moves_remaining} 格
                        </span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">状态:</span>
                        {selectedMapDisciple.current_task_info ? (
                          <span className="detail-value" style={{ color: '#2c7a7b' }}>
                            执行任务中
                          </span>
                        ) : (
                          <span className="detail-value" style={{ color: '#48bb78' }}>
                            空闲
                          </span>
                        )}
                      </div>
                      {selectedMapDisciple.current_task_info && (
                        <div style={{
                          backgroundColor: '#e6fffa',
                          padding: '8px',
                          borderRadius: '4px',
                          marginTop: '8px',
                          color: '#234e52'
                        }}>
                          <span style={{ fontWeight: 'bold' }}>
                            📋 当前任务
                          </span>
                          <div style={{ fontSize: '12px', marginTop: '4px', color: '#2c7a7b' }}>
                            {selectedMapDisciple.current_task_info.task_name}
                          </div>
                          <div style={{ fontSize: '11px', marginTop: '2px', color: '#4a5568' }}>
                            进度: {selectedMapDisciple.current_task_info.progress}/{selectedMapDisciple.current_task_info.duration} 回合
                          </div>
                        </div>
                      )}

                      {/* 人物关系 */}
                      <div style={{
                        marginTop: '12px',
                        padding: '10px',
                        backgroundColor: '#faf5ff',
                        borderRadius: '6px',
                        border: '1px solid #d6bcfa'
                      }}>
                        <div
                          style={{
                            fontWeight: 'bold',
                            marginBottom: '8px',
                            color: '#6b46c1',
                            cursor: 'pointer',
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center'
                          }}
                          onClick={() => setShowRelationships(!showRelationships)}
                        >
                          <span>💜 人物关系 ({discipleRelationships.length})</span>
                          <span style={{ fontSize: '0.8rem' }}>{showRelationships ? '▼' : '▶'}</span>
                        </div>

                        {/* 关系摘要 */}
                        {selectedMapDisciple.relationship_summary && (
                          <div style={{ fontSize: '0.85rem', color: '#553c9a', marginBottom: showRelationships ? '8px' : 0 }}>
                            {selectedMapDisciple.relationship_summary.master_id && (
                              <div>师父: {disciples.find(d => d.id === selectedMapDisciple.relationship_summary.master_id)?.name || '未知'}</div>
                            )}
                            {selectedMapDisciple.relationship_summary.dao_companion_id && (
                              <div>道侣: {disciples.find(d => d.id === selectedMapDisciple.relationship_summary.dao_companion_id)?.name || '未知'}</div>
                            )}
                            {selectedMapDisciple.relationship_summary.disciple_ids.length > 0 && (
                              <div>徒弟: {selectedMapDisciple.relationship_summary.disciple_ids.map(id => disciples.find(d => d.id === id)?.name).filter(Boolean).join('、')}</div>
                            )}
                            {!selectedMapDisciple.relationship_summary.master_id &&
                             !selectedMapDisciple.relationship_summary.dao_companion_id &&
                             selectedMapDisciple.relationship_summary.disciple_ids.length === 0 &&
                             discipleRelationships.length === 0 && (
                              <div style={{ color: '#a0aec0', fontStyle: 'italic' }}>暂无特殊关系</div>
                            )}
                          </div>
                        )}

                        {/* 详细关系列表 */}
                        {showRelationships && discipleRelationships.length > 0 && (
                          <div style={{ marginTop: '8px' }}>
                            {discipleRelationships.map(rel => (
                              <div key={rel.target_id} style={{
                                padding: '8px',
                                marginBottom: '6px',
                                backgroundColor: 'white',
                                borderRadius: '4px',
                                border: '1px solid #e9d8fd'
                              }}>
                                <div style={{ fontWeight: 'bold', marginBottom: '4px', display: 'flex', justifyContent: 'space-between' }}>
                                  <span>{rel.target_name}</span>
                                  <span style={{ fontSize: '0.8rem', color: '#805ad5' }}>{rel.primary_relation}</span>
                                </div>
                                <div style={{ fontSize: '0.8rem', color: '#718096' }}>
                                  {rel.is_dao_companion && <span style={{ marginRight: '6px', color: '#e53e3e' }}>💕道侣</span>}
                                  {rel.is_master && <span style={{ marginRight: '6px', color: '#3182ce' }}>👨‍🏫师父</span>}
                                  {rel.is_disciple && <span style={{ marginRight: '6px', color: '#38a169' }}>👨‍🎓徒弟</span>}
                                </div>
                                <div style={{ fontSize: '0.75rem', color: '#a0aec0', marginTop: '4px' }}>
                                  <span title="情感">💕{rel.scores.romance}</span>
                                  <span style={{ marginLeft: '8px' }} title="师徒">📚{rel.scores.mentorship}</span>
                                  <span style={{ marginLeft: '8px' }} title="战友">⚔️{rel.scores.comrade}</span>
                                  <span style={{ marginLeft: '8px' }} title="认知">🧠{rel.scores.understanding}</span>
                                  <span style={{ marginLeft: '8px' }} title="机缘">🍀{rel.scores.fateful_bond}</span>
                                </div>
                                <div style={{ fontSize: '0.75rem', color: '#805ad5', marginTop: '2px' }}>
                                  关系等级: {rel.highest_level}
                                </div>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>

                      {/* 显示弟子当前位置可接受的任务 */}
                      {!selectedMapDisciple.current_task_info && (() => {
                        // 获取该位置所有任务（不过滤弟子条件）
                        const tasksAtPosition = tasks.filter(t =>
                          t.position &&
                          t.position.x === selectedMapDisciple.position.x &&
                          t.position.y === selectedMapDisciple.position.y
                        );
                        if (tasksAtPosition.length === 0) return null;

                        // 判断弟子是否可以接受任务，返回原因
                        const getTaskStatus = (task: Task) => {
                          if (task.assigned_to.includes(selectedMapDisciple.id)) {
                            return { canAccept: false, reason: '已接受此任务' };
                          }
                          if (task.assigned_to.length >= task.max_participants) {
                            return { canAccept: false, reason: '任务人数已满' };
                          }
                          if (task.suitable_disciples.free.includes(selectedMapDisciple.id)) {
                            return { canAccept: true, reason: '' };
                          }
                          if (task.suitable_disciples.busy.includes(selectedMapDisciple.id)) {
                            return { canAccept: false, reason: '需要完成当前任务' };
                          }
                          // 不在 free 也不在 busy，说明不满足技能要求
                          if (task.skill_required) {
                            return { canAccept: false, reason: `需要技能: ${task.skill_required}` };
                          }
                          return { canAccept: false, reason: '不满足任务条件' };
                        };

                        const acceptableTasks = tasksAtPosition.filter(t => getTaskStatus(t).canAccept);
                        const unacceptableTasks = tasksAtPosition.filter(t => !getTaskStatus(t).canAccept);

                        return (
                          <div style={{
                            marginTop: '12px',
                            padding: '10px',
                            backgroundColor: '#f0fff4',
                            borderRadius: '6px',
                            border: '1px solid #48bb78'
                          }}>
                            <div style={{ fontWeight: 'bold', marginBottom: '8px', color: '#276749' }}>
                              📋 此位置的任务 ({tasksAtPosition.length})
                            </div>
                            {/* 可接受的任务 */}
                            {acceptableTasks.map(task => (
                              <div key={task.id} style={{
                                padding: '8px',
                                marginBottom: '6px',
                                backgroundColor: 'white',
                                borderRadius: '4px',
                                border: '1px solid #c6f6d5'
                              }}>
                                <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>
                                  <span style={{ color: '#718096', fontWeight: 'normal' }}>任务: </span>
                                  {task.name}
                                  {task.max_participants > 1 && (
                                    <span style={{ marginLeft: '6px', fontSize: '0.8rem', color: '#667eea' }}>
                                      👥 {task.assigned_to.length}/{task.max_participants}
                                    </span>
                                  )}
                                </div>
                                <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                  类型: {task.task_type.split('(')[0]}
                                </div>
                                <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                  奖励: 修为+{task.rewards.progress} 资源+{task.rewards.resources}
                                </div>
                                <div style={{ fontSize: '0.8rem', color: '#888', marginTop: '2px' }}>
                                  消耗: 精力-{task.energy_cost} 体魄-{task.constitution_cost}
                                </div>
                                <div style={{ fontSize: '0.8rem', color: '#888', marginTop: '2px' }}>
                                  ⏱️ 需要 {task.duration} 回合 | ⏰ {task.remaining_turns}回合后失效
                                </div>
                                {task.assigned_to.length > 0 && (
                                  <div style={{ fontSize: '0.8rem', color: '#48bb78', marginTop: '4px' }}>
                                    已有: {task.assigned_to.map(id => disciples.find(d => d.id === id)?.name).filter(Boolean).join('、')}
                                  </div>
                                )}
                                <button
                                  onClick={() => assignTask(task.id, selectedMapDisciple.id)}
                                  style={{
                                    marginTop: '6px',
                                    padding: '6px 12px',
                                    backgroundColor: '#48bb78',
                                    color: 'white',
                                    border: 'none',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    fontSize: '0.85rem',
                                    fontWeight: 'bold'
                                  }}
                                >
                                  ✓ 接受任务
                                </button>
                              </div>
                            ))}
                            {/* 无法接受的任务 */}
                            {unacceptableTasks.map(task => {
                              const status = getTaskStatus(task);
                              return (
                                <div key={task.id} style={{
                                  padding: '8px',
                                  marginBottom: '6px',
                                  backgroundColor: '#f7f7f7',
                                  borderRadius: '4px',
                                  border: '1px solid #e2e2e2',
                                  opacity: 0.8
                                }}>
                                  <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>
                                    <span style={{ color: '#718096', fontWeight: 'normal' }}>任务: </span>
                                    {task.name}
                                    {task.max_participants > 1 && (
                                      <span style={{ marginLeft: '6px', fontSize: '0.8rem', color: '#667eea' }}>
                                        👥 {task.assigned_to.length}/{task.max_participants}
                                      </span>
                                    )}
                                  </div>
                                  <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                    类型: {task.task_type.split('(')[0]}
                                  </div>
                                  <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                    奖励: 修为+{task.rewards.progress} 资源+{task.rewards.resources}
                                  </div>
                                  <div style={{ fontSize: '0.8rem', color: '#888', marginTop: '2px' }}>
                                    ⏱️ 需要 {task.duration} 回合 | ⏰ {task.remaining_turns}回合后失效
                                  </div>
                                  {task.assigned_to.length > 0 && (
                                    <div style={{ fontSize: '0.8rem', color: '#48bb78', marginTop: '4px' }}>
                                      已有: {task.assigned_to.map(id => disciples.find(d => d.id === id)?.name).filter(Boolean).join('、')}
                                    </div>
                                  )}
                                  <div style={{
                                    marginTop: '6px',
                                    padding: '6px 12px',
                                    backgroundColor: '#fed7d7',
                                    color: '#c53030',
                                    borderRadius: '4px',
                                    fontSize: '0.85rem',
                                    fontWeight: 'bold',
                                    textAlign: 'center'
                                  }}>
                                    🚫 {status.reason}
                                  </div>
                                </div>
                              );
                            })}
                          </div>
                        );
                      })()}

                      {selectedMapDisciple.current_task_info ? (
                        <div style={{
                          marginTop: '12px',
                          padding: '8px',
                          backgroundColor: '#fed7d7',
                          borderRadius: '4px',
                          fontSize: '13px',
                          color: '#c53030'
                        }}>
                          🚫 正在执行任务，无法移动
                        </div>
                      ) : (
                        <div style={{
                          marginTop: '12px',
                          padding: '8px',
                          backgroundColor: '#bee3f8',
                          borderRadius: '4px',
                          fontSize: '13px',
                          color: '#2c5282'
                        }}>
                          💡 点击地图上的任意位置来移动弟子
                        </div>
                      )}
                    </div>
                  </div>
                )}

                {!selectedElement && !selectedMapDisciple && (
                  <div style={{
                    padding: '2rem',
                    textAlign: 'center',
                    color: '#a0aec0'
                  }}>
                    <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🗺️</div>
                    <div>点击地图上的元素或弟子查看详情</div>
                  </div>
                )}
              </div>
            )}

            {panelTab === 'buildings' && (
              <div>
                <BuildingTree
                  gameId={gameId}
                  onResourcesChanged={onTaskAssigned}
                />
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 底部控制栏 */}
      <div className="bottom-bar">
        <button className="control-button primary" onClick={onNextTurn}>
          ⏭ 下一回合
        </button>
        <button className="control-button secondary" onClick={onAutoAssign}>
          🤖 自动分配任务
        </button>
        <button className="control-button warning" onClick={onResetGame}>
          🔄 重置游戏
        </button>
      </div>
    </div>
  );
};

export default FullscreenMapView;
