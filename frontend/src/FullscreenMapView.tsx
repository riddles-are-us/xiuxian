import React, { useState, useRef, useEffect } from 'react';
import { MapData, MapElement, Disciple, Task, GameInfo, gameApi, Relationship, HerbInventoryResponse, PillRecipe, PillInventory, TaskEligibilityResponse } from './api/gameApi';
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

type PanelType = 'disciples' | 'tasks' | 'mapinfo' | 'buildings' | 'alchemy' | 'pills' | null;

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
  const [panelTab, setPanelTab] = useState<'disciples' | 'tasks' | 'mapinfo' | 'buildings' | 'alchemy' | 'pills'>('disciples');

  // 炼丹相关状态
  const [herbInventory, setHerbInventory] = useState<HerbInventoryResponse | null>(null);
  const [recipes, setRecipes] = useState<PillRecipe[]>([]);
  const [pillInventory, setPillInventory] = useState<PillInventory | null>(null);
  const [alchemyLoading, setAlchemyLoading] = useState(false);
  const [alchemyMessage, setAlchemyMessage] = useState<{text: string, type: 'success' | 'error'} | null>(null);
  const [selectedPillDisciple, setSelectedPillDisciple] = useState<number | null>(null);

  // 地图信息状态
  const [selectedElement, setSelectedElement] = useState<MapElement | null>(null);
  const [selectedMapDisciple, setSelectedMapDisciple] = useState<Disciple | null>(null);
  const [moveError, setMoveError] = useState<string | null>(null);
  const [discipleRelationships, setDiscipleRelationships] = useState<Relationship[]>([]);
  const [showRelationships, setShowRelationships] = useState(false);
  const [taskEligibilities, setTaskEligibilities] = useState<Map<number, TaskEligibilityResponse>>(new Map());

  // 从 localStorage 恢复待移动路径
  const loadPendingPaths = (): Map<number, {x: number, y: number}[]> => {
    try {
      const saved = localStorage.getItem(`pendingPaths_${gameId}`);
      if (saved) {
        const parsed = JSON.parse(saved);
        return new Map(parsed);
      }
    } catch (e) {
      console.error('Failed to load pending paths:', e);
    }
    return new Map();
  };

  // 保存待移动路径到 localStorage
  const savePendingPaths = (paths: Map<number, {x: number, y: number}[]>) => {
    try {
      const serialized = Array.from(paths.entries());
      localStorage.setItem(`pendingPaths_${gameId}`, JSON.stringify(serialized));
    } catch (e) {
      console.error('Failed to save pending paths:', e);
    }
  };

  // 待移动路径状态：记录每个弟子的剩余移动路径
  const [pendingPaths, setPendingPaths] = useState<Map<number, {x: number, y: number}[]>>(() => loadPendingPaths());
  // 使用 ref 备份路径数据
  const pendingPathsRef = useRef<Map<number, {x: number, y: number}[]>>(loadPendingPaths());

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

  // 当选中弟子变化时，加载任务资格检查数据
  useEffect(() => {
    if (selectedMapDisciple && !selectedMapDisciple.current_task_info) {
      // 找出弟子位置的任务
      const monsterAtPosition = mapData?.elements.find(e =>
        e.element_type === 'Monster' &&
        e.position.x === selectedMapDisciple.position.x &&
        e.position.y === selectedMapDisciple.position.y
      );

      const tasksAtPosition = tasks.filter(t => {
        if (t.position &&
            t.position.x === selectedMapDisciple.position.x &&
            t.position.y === selectedMapDisciple.position.y) {
          return true;
        }
        if (monsterAtPosition &&
            monsterAtPosition.details.monster_id &&
            t.enemy_info &&
            t.enemy_info.enemy_id === monsterAtPosition.details.monster_id) {
          return true;
        }
        return false;
      });

      // 批量获取任务资格
      const fetchEligibilities = async () => {
        const newEligibilities = new Map<number, TaskEligibilityResponse>();
        for (const task of tasksAtPosition) {
          try {
            const result = await gameApi.checkTaskEligibility(gameId, task.id, selectedMapDisciple.id);
            newEligibilities.set(task.id, result);
          } catch (err) {
            console.error('Failed to check eligibility for task', task.id, err);
          }
        }
        setTaskEligibilities(newEligibilities);
      };

      fetchEligibilities();
    } else {
      setTaskEligibilities(new Map());
    }
  }, [selectedMapDisciple, tasks, gameId, mapData]);

  // 加载炼丹数据
  const loadAlchemyData = async () => {
    try {
      setAlchemyLoading(true);
      const [herbData, recipeData, pillData] = await Promise.all([
        gameApi.getHerbInventory(gameId),
        gameApi.getRecipes(gameId),
        gameApi.getPillInventory(gameId)
      ]);
      setHerbInventory(herbData);
      setRecipes(recipeData);
      setPillInventory(pillData);
    } catch (err) {
      console.error('Failed to load alchemy data:', err);
    } finally {
      setAlchemyLoading(false);
    }
  };

  // 使用丹药
  const handleUsePill = async (pillType: string, discipleId: number) => {
    try {
      setAlchemyLoading(true);
      const result = await gameApi.usePill(gameId, discipleId, pillType);
      setAlchemyMessage({
        text: result.message,
        type: result.success ? 'success' : 'error'
      });
      await loadAlchemyData();
      // 刷新弟子数据
      await onDiscipleMoved(discipleId);
      setTimeout(() => setAlchemyMessage(null), 3000);
    } catch (err: any) {
      setAlchemyMessage({
        text: err.response?.data?.error?.message || err.message || '使用丹药失败',
        type: 'error'
      });
      setTimeout(() => setAlchemyMessage(null), 3000);
    } finally {
      setAlchemyLoading(false);
    }
  };

  // 当切换到炼丹或丹药 tab 时加载数据
  useEffect(() => {
    if (panelTab === 'alchemy' || panelTab === 'pills') {
      loadAlchemyData();
    }
  }, [panelTab, gameId]);

  // 炼制丹药
  const handleRefine = async (pillType: string) => {
    try {
      setAlchemyLoading(true);
      const result = await gameApi.refinePill(gameId, pillType);
      setAlchemyMessage({
        text: result.message,
        type: result.success ? 'success' : 'error'
      });
      await loadAlchemyData();
      setTimeout(() => setAlchemyMessage(null), 3000);
    } catch (err: any) {
      setAlchemyMessage({
        text: err.message || '炼制失败',
        type: 'error'
      });
      setTimeout(() => setAlchemyMessage(null), 3000);
    } finally {
      setAlchemyLoading(false);
    }
  };

  // 获取品质颜色
  const getQualityColor = (quality: string) => {
    switch (quality) {
      case '普通': return '#9ca3af';
      case '良品': return '#22c55e';
      case '稀有': return '#3b82f6';
      case '珍品': return '#a855f7';
      case '仙品': return '#f59e0b';
      default: return '#6b7280';
    }
  };

  // 计算从起点到终点的简单曼哈顿路径（一步一格）
  const calculatePath = (
    startX: number, startY: number,
    endX: number, endY: number
  ): {x: number, y: number}[] => {
    const path: {x: number, y: number}[] = [];
    let currentX = startX;
    let currentY = startY;

    // 先走X方向，再走Y方向（简单的L形路径）
    while (currentX !== endX) {
      currentX += currentX < endX ? 1 : -1;
      path.push({ x: currentX, y: currentY });
    }
    while (currentY !== endY) {
      currentY += currentY < endY ? 1 : -1;
      path.push({ x: currentX, y: currentY });
    }

    return path;
  };

  // 执行路径移动（移动尽可能多的步数，返回剩余路径）
  const executePathMove = async (
    discipleId: number,
    path: {x: number, y: number}[],
    movesRemaining: number
  ): Promise<{x: number, y: number}[]> => {
    let stepsToMove = Math.min(path.length, movesRemaining);
    let currentStep = 0;

    while (currentStep < stepsToMove) {
      const target = path[currentStep];
      try {
        await gameApi.moveDisciple(gameId, discipleId, target.x, target.y);
        currentStep++;
      } catch (error: any) {
        // 如果移动失败（可能是moves用完了），停止移动
        console.log('Move stopped:', error.response?.data?.error?.message);
        break;
      }
    }

    // 返回剩余的路径
    return path.slice(currentStep);
  };

  // 处理远距离移动请求
  const handleLongDistanceMove = async (
    disciple: Disciple,
    targetX: number,
    targetY: number
  ) => {
    // 计算完整路径
    const fullPath = calculatePath(
      disciple.position.x, disciple.position.y,
      targetX, targetY
    );

    if (fullPath.length === 0) return;

    // 执行移动
    const remainingPath = await executePathMove(
      disciple.id,
      fullPath,
      disciple.moves_remaining
    );

    // 保存剩余路径（同时更新 state、ref 和 localStorage）
    if (remainingPath.length > 0) {
      setPendingPaths(prev => {
        const newMap = new Map(prev);
        newMap.set(disciple.id, remainingPath);
        pendingPathsRef.current = newMap;
        savePendingPaths(newMap);
        return newMap;
      });
      setMoveError(`移动距离不足，已规划路径，下回合将自动继续移动 (剩余${remainingPath.length}格)`);
    } else {
      // 移动完成，清除待移动路径
      setPendingPaths(prev => {
        const newMap = new Map(prev);
        newMap.delete(disciple.id);
        pendingPathsRef.current = newMap;
        savePendingPaths(newMap);
        return newMap;
      });
    }

    // 刷新弟子数据
    await onDiscipleMoved(disciple.id);
  };

  // 取消弟子的待移动路径
  const cancelPendingPath = (discipleId: number) => {
    setPendingPaths(prev => {
      const newMap = new Map(prev);
      newMap.delete(discipleId);
      pendingPathsRef.current = newMap;
      savePendingPaths(newMap);
      return newMap;
    });
    setMoveError(null);
  };

  // 获取弟子的待移动路径
  const getPendingPath = (discipleId: number): {x: number, y: number}[] => {
    return pendingPaths.get(discipleId) || [];
  };

  // 标记是否需要在下次数据刷新后续行（从 localStorage 恢复）
  const loadShouldContinue = (): boolean => {
    try {
      const saved = localStorage.getItem(`shouldContinuePaths_${gameId}`);
      return saved === 'true';
    } catch (e) {
      return false;
    }
  };
  const saveShouldContinue = (value: boolean) => {
    try {
      localStorage.setItem(`shouldContinuePaths_${gameId}`, value ? 'true' : 'false');
    } catch (e) {
      console.error('Failed to save shouldContinue:', e);
    }
  };
  const shouldContinuePathsRef = useRef(loadShouldContinue());
  // 防止重复执行的标记（也需要持久化）
  const loadIsProcessing = (): boolean => {
    try {
      return localStorage.getItem(`isProcessingPaths_${gameId}`) === 'true';
    } catch (e) {
      return false;
    }
  };
  const saveIsProcessing = (value: boolean) => {
    try {
      localStorage.setItem(`isProcessingPaths_${gameId}`, value ? 'true' : 'false');
    } catch (e) {
      console.error('Failed to save isProcessing:', e);
    }
  };
  const isProcessingPathsRef = useRef(loadIsProcessing());

  // 同步更新 pendingPaths state 和 ref，并保存到 localStorage
  const updatePendingPaths = (updater: (prev: Map<number, {x: number, y: number}[]>) => Map<number, {x: number, y: number}[]>) => {
    setPendingPaths(prev => {
      const newMap = updater(prev);
      pendingPathsRef.current = newMap;
      savePendingPaths(newMap);
      return newMap;
    });
  };

  // 组件挂载时，从 localStorage 同步 refs（处理组件重新挂载的情况）
  useEffect(() => {
    const loadedPaths = loadPendingPaths();
    const loadedShouldContinue = loadShouldContinue();
    let loadedIsProcessing = loadIsProcessing();

    // 如果上次处理中断了（isProcessing = true 但组件已重新挂载），重置状态
    // 这允许continuation effect 重新尝试
    if (loadedIsProcessing && loadedShouldContinue && loadedPaths.size > 0) {
      console.log('检测到上次处理中断，重置 isProcessing');
      loadedIsProcessing = false;
      saveIsProcessing(false);
    }

    pendingPathsRef.current = loadedPaths;
    shouldContinuePathsRef.current = loadedShouldContinue;
    isProcessingPathsRef.current = loadedIsProcessing;
    setPendingPaths(loadedPaths);
    console.log('组件挂载，从localStorage恢复: shouldContinue:', loadedShouldContinue, 'paths:', loadedPaths.size, 'isProcessing:', loadedIsProcessing);
  }, [gameId]);

  // 当弟子数据更新时，检查是否需要续行
  useEffect(() => {
    console.log('disciples 更新, shouldContinue:', shouldContinuePathsRef.current, 'pathsSize:', pendingPathsRef.current.size, 'isProcessing:', isProcessingPathsRef.current);

    // 如果正在处理中，跳过
    if (isProcessingPathsRef.current) {
      console.log('正在处理中，跳过');
      return;
    }

    if (shouldContinuePathsRef.current && pendingPathsRef.current.size > 0) {
      // 检查弟子移动力是否已恢复（确认是新回合）
      const firstPathEntry = Array.from(pendingPathsRef.current.entries())[0];
      if (firstPathEntry) {
        const disciple = disciples.find(d => d.id === firstPathEntry[0]);
        console.log('检查弟子移动力:', disciple?.moves_remaining);

        // 如果移动力为0，说明数据还没刷新，等待下次更新
        if (disciple && disciple.moves_remaining === 0) {
          console.log('移动力为0，等待数据刷新');
          return;
        }
      }

      // 设置处理中标记（防止重复执行）
      isProcessingPathsRef.current = true;
      saveIsProcessing(true);

      // 从 ref 恢复路径数据
      const pathsToProcess = new Map(pendingPathsRef.current);
      console.log('开始续行移动，待处理路径数:', pathsToProcess.size);

      // 执行移动（注意：只有在完成后才清除 shouldContinue）
      (async () => {
        try {
          const entries = Array.from(pathsToProcess.entries());
          for (const [discipleId, path] of entries) {
            const disciple = disciples.find(d => d.id === discipleId);
            console.log(`弟子 ${discipleId} 移动力: ${disciple?.moves_remaining}, 路径长度: ${path.length}`);

            if (!disciple || path.length === 0) continue;
            if (disciple.current_task_info) {
              // 弟子正在执行任务，清除其路径
              const newMap = new Map(pendingPathsRef.current);
              newMap.delete(discipleId);
              pendingPathsRef.current = newMap;
              savePendingPaths(newMap);
              setPendingPaths(newMap);
              continue;
            }

            // 执行移动
            const remainingPath = await executePathMove(
              discipleId,
              path,
              disciple.moves_remaining
            );

            // 更新剩余路径（直接操作 ref 和 localStorage，确保持久化）
            const newMap = new Map(pendingPathsRef.current);
            if (remainingPath.length > 0) {
              newMap.set(discipleId, remainingPath);
            } else {
              newMap.delete(discipleId);
            }
            pendingPathsRef.current = newMap;
            savePendingPaths(newMap);
            setPendingPaths(newMap);

            // 刷新弟子数据
            await onDiscipleMoved(discipleId);
          }
          console.log('续行移动完成');

          // 移动完成后，清除续行标记
          shouldContinuePathsRef.current = false;
          saveShouldContinue(false);
        } finally {
          isProcessingPathsRef.current = false;
          saveIsProcessing(false);
        }
      })();
    }
  }, [disciples]);

  // 处理下一回合，包含自动续行逻辑
  const handleNextTurnWithPaths = () => {
    // 标记需要续行（使用 ref 和 localStorage 确保不会丢失）
    if (pendingPathsRef.current.size > 0) {
      shouldContinuePathsRef.current = true;
      saveShouldContinue(true);
      console.log('标记续行，路径数:', pendingPathsRef.current.size);
    }
    // 调用原始的 onNextTurn（开始新回合）
    onNextTurn();
  };

  // 地图拖拽平移状态 - 使用 transform 而不是 scroll
  const mapContainerRef = useRef<HTMLDivElement>(null);
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  // mapPosition 现在由父组件管理，不再使用本地 state
  const savedMapPosition = useRef({ x: 0, y: 0 }); // 用于拖拽开始时保存位置

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
        <div className="top-bar-right">
          <button
            className={`top-tab ${panelTab === 'disciples' && activePanel !== null ? 'active' : ''}`}
            onClick={() => { setPanelTab('disciples'); setActivePanel('disciples'); }}
          >
            👥 弟子
          </button>
          <button
            className={`top-tab ${panelTab === 'tasks' && activePanel !== null ? 'active' : ''}`}
            onClick={() => { setPanelTab('tasks'); setActivePanel('tasks'); }}
          >
            📋 任务
          </button>
          <button
            className={`top-tab ${panelTab === 'mapinfo' && activePanel !== null ? 'active' : ''}`}
            onClick={() => { setPanelTab('mapinfo'); setActivePanel('mapinfo'); }}
          >
            🗺️ 地图
          </button>
          <button
            className={`top-tab ${panelTab === 'buildings' && activePanel !== null ? 'active' : ''}`}
            onClick={() => { setPanelTab('buildings'); setActivePanel('buildings'); }}
          >
            🏛️ 建筑
          </button>
          <button
            className={`top-tab ${panelTab === 'alchemy' && activePanel !== null ? 'active' : ''}`}
            onClick={() => { setPanelTab('alchemy'); setActivePanel('alchemy'); }}
          >
            🧪 炼丹
          </button>
          <button
            className={`top-tab ${panelTab === 'pills' && activePanel !== null ? 'active' : ''}`}
            onClick={() => { setPanelTab('pills'); setActivePanel('pills'); }}
          >
            💊 丹药
          </button>
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
              onLongDistanceMove={handleLongDistanceMove}
              pendingPaths={pendingPaths}
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

        </div>

        {/* 侧边面板 */}
        <div className={`side-panel ${activePanel === null ? 'collapsed' : ''}`}>
          <div className="panel-header">
            <div className="panel-title">
              {panelTab === 'disciples' && `👥 弟子列表 (${disciples.length})`}
              {panelTab === 'tasks' && `📋 任务列表 (${tasks.length})`}
              {panelTab === 'mapinfo' && '🗺️ 地图信息'}
              {panelTab === 'buildings' && '🏛️ 宗门建筑'}
              {panelTab === 'alchemy' && '🧪 炼丹'}
              {panelTab === 'pills' && '💊 丹药'}
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

                      {/* 待移动路径 */}
                      {getPendingPath(selectedMapDisciple.id).length > 0 && (
                        <div style={{
                          backgroundColor: '#fffbeb',
                          padding: '8px',
                          borderRadius: '4px',
                          marginTop: '8px',
                          border: '1px dashed #f59e0b'
                        }}>
                          <div style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center'
                          }}>
                            <span style={{ fontWeight: 'bold', color: '#b45309' }}>
                              🗺️ 待移动路径
                            </span>
                            <button
                              onClick={() => cancelPendingPath(selectedMapDisciple.id)}
                              style={{
                                padding: '2px 8px',
                                fontSize: '11px',
                                backgroundColor: '#fef3c7',
                                border: '1px solid #f59e0b',
                                borderRadius: '4px',
                                cursor: 'pointer',
                                color: '#b45309'
                              }}
                            >
                              ✕ 取消
                            </button>
                          </div>
                          <div style={{ fontSize: '12px', marginTop: '4px', color: '#92400e' }}>
                            剩余 {getPendingPath(selectedMapDisciple.id).length} 格
                          </div>
                          <div style={{ fontSize: '11px', marginTop: '2px', color: '#78716c' }}>
                            目标: ({getPendingPath(selectedMapDisciple.id).slice(-1)[0]?.x}, {getPendingPath(selectedMapDisciple.id).slice(-1)[0]?.y})
                          </div>
                          <div style={{ fontSize: '10px', marginTop: '4px', color: '#a8a29e' }}>
                            下回合将自动继续移动
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
                        // 查找弟子位置的怪物（如果有）
                        const monsterAtPosition = mapData?.elements.find(e =>
                          e.element_type === 'Monster' &&
                          e.position.x === selectedMapDisciple.position.x &&
                          e.position.y === selectedMapDisciple.position.y
                        );

                        // 获取该位置所有任务（不过滤弟子条件）
                        const tasksAtPosition = tasks.filter(t => {
                          // 按位置匹配
                          if (t.position &&
                              t.position.x === selectedMapDisciple.position.x &&
                              t.position.y === selectedMapDisciple.position.y) {
                            return true;
                          }

                          // 如果弟子位置有怪物，匹配相关的战斗任务（通过 monster_id）
                          if (monsterAtPosition &&
                              monsterAtPosition.details.monster_id &&
                              t.enemy_info &&
                              t.enemy_info.enemy_id === monsterAtPosition.details.monster_id) {
                            return true;
                          }

                          return false;
                        });
                        if (tasksAtPosition.length === 0) return null;

                        // 判断弟子是否可以接受任务，使用后端API返回的结果
                        const getTaskStatus = (task: Task) => {
                          const eligibility = taskEligibilities.get(task.id);
                          if (eligibility) {
                            return {
                              canAccept: eligibility.eligible,
                              reason: eligibility.reason || '',
                              successRate: eligibility.success_rate,
                              discipleLevel: eligibility.disciple_combat_level,
                              enemyLevel: eligibility.enemy_level
                            };
                          }
                          // 如果API结果还没加载，返回加载中状态
                          return { canAccept: false, reason: '检查中...', successRate: null, discipleLevel: null, enemyLevel: null };
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
                            {acceptableTasks.map(task => {
                              const status = getTaskStatus(task);
                              const isCombatTask = task.task_type.startsWith('Combat');
                              const successRate = status.successRate;
                              const successRateColor = successRate != null
                                ? successRate >= 0.7 ? '#48bb78'
                                : successRate >= 0.4 ? '#ed8936'
                                : '#e53e3e'
                                : '#666';

                              return (
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
                                  {isCombatTask && successRate != null && (
                                    <div style={{ fontSize: '0.85rem', marginTop: '2px' }}>
                                      <span style={{ color: successRateColor, fontWeight: 'bold' }}>
                                        🎯 成功率: {Math.round(successRate * 100)}%
                                      </span>
                                      <span style={{ color: '#888', marginLeft: '8px', fontSize: '0.8rem' }}>
                                        (弟子Lv{status.discipleLevel} vs 敌人Lv{status.enemyLevel})
                                      </span>
                                    </div>
                                  )}
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
                              );
                            })}
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

            {panelTab === 'alchemy' && (
              <div style={{ padding: '0.5rem' }}>
                {alchemyMessage && (
                  <div style={{
                    padding: '0.5rem 0.75rem',
                    borderRadius: '6px',
                    marginBottom: '0.75rem',
                    background: alchemyMessage.type === 'success' ? '#d1fae5' : '#fee2e2',
                    color: alchemyMessage.type === 'success' ? '#047857' : '#dc2626',
                    border: `1px solid ${alchemyMessage.type === 'success' ? '#34d399' : '#f87171'}`,
                    fontSize: '0.9rem'
                  }}>
                    {alchemyMessage.text}
                  </div>
                )}

                {alchemyLoading ? (
                  <div style={{ textAlign: 'center', padding: '1rem', color: '#666' }}>加载中...</div>
                ) : (
                  <>
                    {/* 草药仓库 */}
                    <div style={{ marginBottom: '1rem' }}>
                      <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '1rem', color: '#374151' }}>
                        🌿 草药仓库 ({herbInventory?.total_count || 0})
                      </h4>
                      {herbInventory && herbInventory.herbs.length > 0 ? (
                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
                          {herbInventory.herbs.map((herb, idx) => (
                            <div key={idx} style={{
                              padding: '0.4rem 0.6rem',
                              background: '#fff',
                              border: `2px solid ${getQualityColor(herb.quality)}`,
                              borderRadius: '6px',
                              fontSize: '0.85rem'
                            }}>
                              <span style={{ fontWeight: 600 }}>{herb.name}</span>
                              <span style={{ color: getQualityColor(herb.quality), marginLeft: '0.25rem' }}>
                                ({herb.quality})
                              </span>
                              <span style={{ color: '#6b7280', marginLeft: '0.25rem' }}>x{herb.count}</span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div style={{
                          padding: '1rem',
                          background: '#f9fafb',
                          borderRadius: '6px',
                          color: '#9ca3af',
                          textAlign: 'center',
                          border: '1px dashed #d1d5db',
                          fontSize: '0.9rem'
                        }}>
                          暂无草药，派遣弟子去采集吧
                        </div>
                      )}
                    </div>

                    {/* 炼丹配方 */}
                    <div>
                      <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '1rem', color: '#374151' }}>
                        🧪 炼丹配方
                      </h4>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        {recipes.map((recipe) => (
                          <div key={recipe.pill_type} style={{
                            padding: '0.75rem',
                            background: recipe.can_craft ? '#f0fdf4' : '#f9fafb',
                            border: `1px solid ${recipe.can_craft ? '#34d399' : '#e5e7eb'}`,
                            borderRadius: '8px'
                          }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
                              <span style={{ fontWeight: 700, color: '#1f2937' }}>{recipe.name}</span>
                              <span style={{
                                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                                color: 'white',
                                padding: '0.15rem 0.4rem',
                                borderRadius: '4px',
                                fontSize: '0.75rem',
                                fontWeight: 600
                              }}>
                                {Math.round(recipe.success_rate * 100)}%
                              </span>
                            </div>
                            <div style={{ fontSize: '0.8rem', color: '#6b7280', marginBottom: '0.5rem' }}>
                              {recipe.description}
                            </div>
                            <div style={{ fontSize: '0.8rem', color: '#9ca3af', marginBottom: '0.5rem' }}>
                              需要: {recipe.required_herb_count}x
                              <span style={{ color: getQualityColor(recipe.required_herb_quality) }}>
                                {recipe.required_herb_quality}
                              </span>
                              草药 + {recipe.resource_cost}资源
                            </div>
                            {!recipe.can_craft && recipe.reason && (
                              <div style={{
                                fontSize: '0.75rem',
                                color: '#dc2626',
                                background: '#fef2f2',
                                padding: '0.25rem 0.5rem',
                                borderRadius: '4px',
                                marginBottom: '0.5rem'
                              }}>
                                {recipe.reason}
                              </div>
                            )}
                            <button
                              onClick={() => handleRefine(recipe.pill_type)}
                              disabled={!recipe.can_craft || alchemyLoading}
                              style={{
                                width: '100%',
                                padding: '0.4rem',
                                border: 'none',
                                borderRadius: '6px',
                                fontWeight: 600,
                                fontSize: '0.85rem',
                                cursor: recipe.can_craft ? 'pointer' : 'not-allowed',
                                background: recipe.can_craft
                                  ? 'linear-gradient(135deg, #10b981 0%, #059669 100%)'
                                  : '#d1d5db',
                                color: recipe.can_craft ? 'white' : '#6b7280'
                              }}
                            >
                              {alchemyLoading ? '炼制中...' : '炼制'}
                            </button>
                          </div>
                        ))}
                      </div>
                    </div>
                  </>
                )}
              </div>
            )}

            {panelTab === 'pills' && (
              <div style={{ padding: '0.5rem' }}>
                {alchemyMessage && (
                  <div style={{
                    padding: '0.5rem 0.75rem',
                    borderRadius: '6px',
                    marginBottom: '0.75rem',
                    background: alchemyMessage.type === 'success' ? '#d1fae5' : '#fee2e2',
                    color: alchemyMessage.type === 'success' ? '#047857' : '#dc2626',
                    border: `1px solid ${alchemyMessage.type === 'success' ? '#34d399' : '#f87171'}`,
                    fontSize: '0.9rem'
                  }}>
                    {alchemyMessage.text}
                  </div>
                )}

                {alchemyLoading ? (
                  <div style={{ textAlign: 'center', padding: '1rem', color: '#666' }}>加载中...</div>
                ) : (
                  <div>
                    <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '1rem', color: '#374151' }}>
                      💊 宗门丹药库存
                    </h4>
                    {pillInventory && Object.keys(pillInventory.pills).length > 0 ? (
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                        {Object.entries(pillInventory.pills).map(([pillType, pill]) => (
                          <div key={pillType} style={{
                            padding: '1rem',
                            background: 'linear-gradient(135deg, #fef3c7 0%, #fde68a 100%)',
                            border: '2px solid #f59e0b',
                            borderRadius: '12px',
                            boxShadow: '0 2px 8px rgba(245, 158, 11, 0.2)'
                          }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
                              <span style={{ fontWeight: 700, fontSize: '1.1rem', color: '#92400e' }}>
                                {pill.name}
                              </span>
                              <span style={{
                                background: '#f59e0b',
                                color: 'white',
                                padding: '0.25rem 0.75rem',
                                borderRadius: '20px',
                                fontSize: '0.9rem',
                                fontWeight: 600
                              }}>
                                库存: {pill.count}
                              </span>
                            </div>
                            <div style={{ fontSize: '0.9rem', color: '#78716c', marginBottom: '0.75rem' }}>
                              {pill.description}
                            </div>
                            <div style={{
                              fontSize: '0.85rem',
                              color: '#059669',
                              background: '#d1fae5',
                              padding: '0.5rem',
                              borderRadius: '6px',
                              marginBottom: '0.75rem'
                            }}>
                              效果: {pill.energy_restore > 0 && `精力+${pill.energy_restore} `}
                              {pill.constitution_restore > 0 && `体魄+${pill.constitution_restore} `}
                              {pill.cultivation_boost > 0 && `修为进度+${pill.cultivation_boost}`}
                            </div>
                            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                              <select
                                value={selectedPillDisciple || ''}
                                onChange={(e) => setSelectedPillDisciple(e.target.value ? parseInt(e.target.value) : null)}
                                style={{
                                  flex: 1,
                                  padding: '0.5rem',
                                  border: '2px solid #d4d4d4',
                                  borderRadius: '6px',
                                  fontSize: '0.9rem',
                                  background: 'white'
                                }}
                              >
                                <option value="">选择弟子服用...</option>
                                {disciples.map(d => (
                                  <option key={d.id} value={d.id}>
                                    {d.name} ({pill.cultivation_boost > 0
                                      ? `修为:${d.cultivation.progress}% ${d.cultivation.level} ${d.cultivation.sub_level}`
                                      : `精力:${d.energy}/100 体魄:${d.constitution}/100`})
                                  </option>
                                ))}
                              </select>
                              <button
                                onClick={() => {
                                  if (selectedPillDisciple) {
                                    handleUsePill(pillType, selectedPillDisciple);
                                    setSelectedPillDisciple(null);
                                  }
                                }}
                                disabled={!selectedPillDisciple || alchemyLoading || pill.count <= 0}
                                style={{
                                  padding: '0.5rem 1rem',
                                  border: 'none',
                                  borderRadius: '6px',
                                  fontWeight: 700,
                                  fontSize: '0.9rem',
                                  cursor: selectedPillDisciple ? 'pointer' : 'not-allowed',
                                  background: selectedPillDisciple
                                    ? 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)'
                                    : '#d1d5db',
                                  color: selectedPillDisciple ? 'white' : '#6b7280',
                                  boxShadow: selectedPillDisciple ? '0 2px 4px rgba(245, 158, 11, 0.3)' : 'none'
                                }}
                              >
                                服用
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div style={{
                        padding: '2rem',
                        background: '#f9fafb',
                        borderRadius: '12px',
                        color: '#9ca3af',
                        textAlign: 'center',
                        border: '2px dashed #d1d5db'
                      }}>
                        <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>💊</div>
                        <div style={{ fontSize: '1rem' }}>暂无丹药</div>
                        <div style={{ fontSize: '0.85rem', marginTop: '0.5rem' }}>
                          前往「炼丹」页面炼制丹药吧！
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 底部控制栏 */}
      <div className="bottom-bar">
        <button className="control-button primary" onClick={handleNextTurnWithPaths}>
          ⏭ 下一回合 {pendingPaths.size > 0 && `(${pendingPaths.size}个弟子待续行)`}
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
