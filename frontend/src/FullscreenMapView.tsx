import React, { useState, useRef, useEffect } from 'react';
import { MapData, MapElement, Disciple, Task, GameInfo, gameApi } from './api/gameApi';
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
  onDiscipleMoved: () => void;
  onTaskAssigned: () => void;
  onAutoAssign: () => void;
  onNextTurn: () => void;
  onResetGame: () => void;
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
  onResetGame
}) => {
  const [activePanel, setActivePanel] = useState<PanelType>(null);
  const [panelTab, setPanelTab] = useState<'disciples' | 'tasks' | 'mapinfo' | 'buildings'>('disciples');

  // 地图信息状态
  const [selectedElement, setSelectedElement] = useState<MapElement | null>(null);
  const [selectedMapDisciple, setSelectedMapDisciple] = useState<Disciple | null>(null);
  const [moveError, setMoveError] = useState<string | null>(null);

  // 地图拖拽平移状态 - 使用 transform 而不是 scroll
  const mapContainerRef = useRef<HTMLDivElement>(null);
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  const [mapPosition, setMapPosition] = useState({ x: 0, y: 0 }); // 地图的当前位置
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

      setMapPosition({
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
  }, [isPanning, panStart]);

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

  // 处理弟子移动 - 使用 transform 后不需要保存/恢复位置
  const handleDiscipleMoved = async () => {
    await onDiscipleMoved();
    // transform 方式下，mapPosition 状态会自动保持，无需额外处理
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

    setMapPosition({ x: newX, y: newY });

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
                  <div key={task.id} className={`task-list-item ${task.assigned_to !== null ? 'assigned' : ''}`}>
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
                      {task.assigned_to !== null && <span style={{marginLeft: '0.5rem', fontSize: '0.9rem'}}>✅</span>}
                      {task.position && <span style={{marginLeft: '0.5rem', fontSize: '0.8rem', opacity: 0.7}}>🗺️</span>}
                    </div>
                    <div className="task-info">
                      <div>类型: {task.task_type}</div>
                      <div>奖励: 修为+{task.rewards.progress} 资源+{task.rewards.resources}</div>
                      <div>消耗: 精力-{task.energy_cost} 体魄-{task.constitution_cost}</div>
                      <div>期限: {task.remaining_turns} 回合</div>
                      {task.position && (
                        <div>位置: ({task.position.x}, {task.position.y})</div>
                      )}
                      {task.assigned_to !== null ? (
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
                            取消分配
                          </button>
                          <span style={{marginLeft: '0.5rem', color: '#48bb78'}}>
                            分配给: {disciples.find(d => d.id === task.assigned_to)?.name}
                          </span>
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
                        <span className="detail-label">修为:</span>
                        <span className="detail-value">
                          {selectedMapDisciple.cultivation.level} {selectedMapDisciple.cultivation.sub_level}
                        </span>
                      </div>
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
                        <span className="detail-label">精力:</span>
                        <span className="detail-value">{selectedMapDisciple.energy}/100</span>
                      </div>
                      <div className="detail-row">
                        <span className="detail-label">体魄:</span>
                        <span className="detail-value">{selectedMapDisciple.constitution}/100</span>
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
                            📋 正在执行任务
                          </span>
                          <div style={{ fontSize: '12px', marginTop: '4px', color: '#2c7a7b' }}>
                            {selectedMapDisciple.current_task_info.task_name}
                          </div>
                        </div>
                      )}
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
