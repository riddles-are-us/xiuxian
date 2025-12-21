import React, { useState, useEffect } from 'react';
import { MapData, MapElement, Disciple, gameApi } from './api/gameApi';
import './MapView.css';

interface MapViewProps {
  mapData: MapData;
  disciples: Disciple[];
  gameId: string;
  onDiscipleMoved?: (movedDiscipleId: number) => void;  // 传递移动的弟子ID
  onElementSelected?: (element: MapElement | null) => void;
  onDiscipleSelected?: (disciple: Disciple | null) => void;
  onMoveError?: (error: string | null) => void;
  // 地图平移相关
  transform?: { x: number; y: number };
  onMapMouseDown?: (e: React.MouseEvent) => void;
  isPanning?: boolean;
}

const MapView: React.FC<MapViewProps> = ({
  mapData,
  disciples,
  gameId,
  onDiscipleMoved,
  onElementSelected,
  onDiscipleSelected,
  onMoveError,
  transform,
  onMapMouseDown,
  isPanning
}) => {
  const [hoveredPosition, setHoveredPosition] = useState<{x: number, y: number} | null>(null);
  const [selectedDisciple, setSelectedDisciple] = useState<Disciple | null>(null);

  // 当弟子数据更新时，同步更新选中的弟子状态（保持选中但更新数据）
  useEffect(() => {
    if (selectedDisciple) {
      const updatedDisciple = disciples.find(d => d.id === selectedDisciple.id);
      if (updatedDisciple) {
        setSelectedDisciple(updatedDisciple);
      }
    }
  }, [disciples]);

  // 获取指定位置的元素
  const getElementAt = (x: number, y: number): MapElement | undefined => {
    return mapData.elements.find(
      el => el.position.x === x && el.position.y === y
    );
  };

  // 获取指定位置的弟子
  const getDisciplesAt = (x: number, y: number): Disciple[] => {
    return disciples.filter(d => d.position.x === x && d.position.y === y);
  };

  // 计算曼哈顿距离
  const getManhattanDistance = (x1: number, y1: number, x2: number, y2: number): number => {
    return Math.abs(x2 - x1) + Math.abs(y2 - y1);
  };

  // 检查位置是否在弟子移动范围内
  const isInMovementRange = (x: number, y: number, disciple: Disciple): boolean => {
    const distance = getManhattanDistance(disciple.position.x, disciple.position.y, x, y);
    return distance <= disciple.movement_range;
  };

  // 处理地图格子点击
  const handleTileClick = async (x: number, y: number) => {
    const disciplesAtPosition = getDisciplesAt(x, y);

    // 如果当前有选中的弟子
    if (selectedDisciple) {
      // 如果点击的是自己当前的位置，取消选中
      if (selectedDisciple.position.x === x && selectedDisciple.position.y === y) {
        // 如果该位置有其他弟子，切换到下一个弟子
        const otherDisciples = disciplesAtPosition.filter(d => d.id !== selectedDisciple.id);
        if (otherDisciples.length > 0) {
          setSelectedDisciple(otherDisciples[0]);
          onDiscipleSelected?.(otherDisciples[0]);
          onMoveError?.(null);
        } else {
          // 没有其他弟子，取消选中
          setSelectedDisciple(null);
          onDiscipleSelected?.(null);
          onMoveError?.(null);
        }
        return;
      }

      // 检查弟子是否正在执行任务
      if (selectedDisciple.current_task_info) {
        onMoveError?.(`${selectedDisciple.name}正在执行任务，无法移动`);
        return;
      }

      // 检查是否在移动范围内，如果在范围内则移动
      if (isInMovementRange(x, y, selectedDisciple)) {
        await moveDisciple(selectedDisciple.id, x, y);
        return;
      }

      // 不在移动范围内，如果点击位置有弟子则切换选中
      if (disciplesAtPosition.length > 0) {
        const disciple = disciplesAtPosition[0];
        setSelectedDisciple(disciple);
        onDiscipleSelected?.(disciple);
        onElementSelected?.(null);
        onMoveError?.(null);
        return;
      }

      // 不在范围内且没有弟子，显示错误
      const distance = getManhattanDistance(selectedDisciple.position.x, selectedDisciple.position.y, x, y);
      const error = `移动距离(${distance})超出范围！${selectedDisciple.name}的最大移动距离为${selectedDisciple.movement_range}格`;
      onMoveError?.(error);
      return;
    }

    // 没有选中弟子时，如果点击位置有弟子则选中
    if (disciplesAtPosition.length > 0) {
      const disciple = disciplesAtPosition[0];
      setSelectedDisciple(disciple);
      onDiscipleSelected?.(disciple);
      onElementSelected?.(null);
      onMoveError?.(null);
      return;
    }

    // 否则，选择该位置的地图元素
    const element = getElementAt(x, y);
    if (element) {
      onElementSelected?.(element);
      onDiscipleSelected?.(null);
    }
  };

  // 移动弟子
  const moveDisciple = async (discipleId: number, x: number, y: number) => {
    onMoveError?.(null);

    try {
      await gameApi.moveDisciple(gameId, discipleId, x, y);
      // 不清除选中状态，让父组件刷新后重新选中该弟子
      if (onDiscipleMoved) {
        await onDiscipleMoved(discipleId);
      }
    } catch (error: any) {
      const errorMsg = error.response?.data?.error?.message || '移动失败';
      onMoveError?.(errorMsg);
    }
  };

  // 获取元素图标
  const getElementIcon = (elementType: string, details?: any): string => {
    switch(elementType) {
      case 'Village': return '🏘️';
      case 'Faction': return '⚔️';
      case 'DangerousLocation': return '⚠️';
      case 'SecretRealm': return '🌀';
      case 'Monster': return '👹';
      case 'Terrain': {
        // 根据地形类型显示不同图标
        const terrainType = details?.terrain_type;
        if (terrainType === 'Mountain') return '⛰️';
        if (terrainType === 'Water') return '💧';
        if (terrainType === 'Forest') return '🌲';
        if (terrainType === 'Plain') return '🌾';
        return '🗺️';
      }
      default: return '?';
    }
  };

  // 获取元素颜色类
  const getElementColorClass = (elementType: string): string => {
    switch(elementType) {
      case 'Village': return 'tile-village';
      case 'Faction': return 'tile-faction';
      case 'DangerousLocation': return 'tile-dangerous';
      case 'SecretRealm': return 'tile-secret';
      case 'Monster': return 'tile-monster';
      case 'Terrain': return 'tile-terrain';
      default: return 'tile-empty';
    }
  };

  // 渲染攻击警告
  const renderAttackWarning = (attackInfo?: any) => {
    if (!attackInfo) return null;

    return (
      <div className="detail-row" style={{
        backgroundColor: attackInfo.is_demon ? '#fed7d7' : '#fef5e7',
        padding: '8px',
        borderRadius: '4px',
        marginBottom: '8px',
        border: attackInfo.is_demon ? '2px solid #c53030' : '2px solid #ed8936'
      }}>
        <span style={{ fontSize: '16px', marginRight: '4px' }}>
          {attackInfo.is_demon ? '⚠️' : '🛡️'}
        </span>
        <span style={{
          fontWeight: 'bold',
          color: attackInfo.is_demon ? '#c53030' : '#ed8936'
        }}>
          受到攻击！
        </span>
        <div style={{ marginTop: '4px', fontSize: '12px', color: '#4a5568' }}>
          攻击者: {attackInfo.attacker_name} (等级 {attackInfo.attacker_level})
          {attackInfo.is_demon && <span style={{color: '#c53030', marginLeft: '4px'}}>【魔物】</span>}
        </div>
      </div>
    );
  };

  // 渲染元素详情
  const renderElementDetails = (element: MapElement) => {
    const { details } = element;

    switch(element.element_type) {
      case 'Village':
        return (
          <>
            {renderAttackWarning(details.under_attack)}
            <div className="detail-row">
              <span className="detail-label">人口:</span>
              <span className="detail-value">{details.population}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">繁荣度:</span>
              <span className="detail-value">{details.prosperity}</span>
            </div>
          </>
        );
      case 'Faction':
        return (
          <>
            {renderAttackWarning(details.under_attack)}
            <div className="detail-row">
              <span className="detail-label">实力等级:</span>
              <span className="detail-value">{details.power_level}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">关系:</span>
              <span className="detail-value" style={{
                color: (details.relationship || 0) >= 0 ? '#48bb78' : '#f56565'
              }}>
                {details.relationship}
              </span>
            </div>
          </>
        );
      case 'DangerousLocation':
        return (
          <div className="detail-row">
            <span className="detail-label">危险等级:</span>
            <span className="detail-value">{details.danger_level}</span>
          </div>
        );
      case 'SecretRealm':
        return (
          <>
            {renderAttackWarning(details.under_attack)}
            <div className="detail-row">
              <span className="detail-label">类型:</span>
              <span className="detail-value">{details.realm_type}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">难度:</span>
              <span className="detail-value">{details.difficulty}</span>
            </div>
          </>
        );
      case 'Monster':
        return (
          <>
            <div className="detail-row">
              <span className="detail-label">等级:</span>
              <span className="detail-value">{details.level}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">状态:</span>
              <span className="detail-value" style={{
                color: details.is_demon ? '#c53030' : '#2d3748'
              }}>
                {details.is_demon ? '成魔' : '正常'}
              </span>
            </div>
            {details.invading_location && (
              <div className="detail-row" style={{
                backgroundColor: '#fed7d7',
                padding: '8px',
                borderRadius: '4px',
                marginTop: '8px',
                marginBottom: '8px',
                border: '2px solid #fc8181'
              }}>
                <span style={{ fontSize: '16px', marginRight: '4px' }}>
                  ⚔️
                </span>
                <span style={{
                  fontWeight: 'bold',
                  color: '#c53030'
                }}>
                  正在入侵
                </span>
                <div style={{ marginTop: '4px', fontSize: '14px', color: '#2d3748' }}>
                  目标: {details.invading_location}
                </div>
              </div>
            )}
            {details.growth_rate !== undefined && (
              <>
                <div className="detail-row">
                  <span className="detail-label">成长速率:</span>
                  <span className="detail-value">
                    {(details.growth_rate * 100).toFixed(1)}%/回合
                  </span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">升级预测:</span>
                  <span className="detail-value" style={{
                    color: details.growth_rate > 0.15 ? '#ed8936' : '#48bb78'
                  }}>
                    {details.growth_rate > 0.15 ? '⚠️ 快速' : '✓ 缓慢'}
                  </span>
                </div>
                {!details.is_demon && (
                  <div className="detail-row">
                    <span className="detail-label">成魔风险:</span>
                    <span className="detail-value" style={{
                      color: (details.level || 0) > 70 ? '#c53030' : (details.level || 0) > 50 ? '#ed8936' : '#48bb78'
                    }}>
                      {(details.level || 0) >= 100 ? '已成魔' :
                       (details.level || 0) > 70 ? '⚠️ 高' :
                       (details.level || 0) > 50 ? '⚠ 中' : '✓ 低'}
                    </span>
                  </div>
                )}
              </>
            )}
          </>
        );
      case 'Terrain':
        return (
          <div className="detail-row">
            <span className="detail-label">地形类型:</span>
            <span className="detail-value">
              {details.terrain_type === 'Mountain' && '山脉 ⛰️'}
              {details.terrain_type === 'Water' && '水域 💧'}
              {details.terrain_type === 'Forest' && '森林 🌲'}
              {details.terrain_type === 'Plain' && '平原 🌾'}
            </span>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="map-view-container">
      <div className="map-grid-wrapper">
        <div
          className="map-grid"
          onMouseDown={onMapMouseDown}
          style={{
            gridTemplateColumns: `repeat(${mapData.width}, 1fr)`,
            gridTemplateRows: `repeat(${mapData.height}, 1fr)`,
            transform: transform ? `translate(${transform.x}px, ${transform.y}px)` : undefined,
            transition: isPanning ? 'none' : 'transform 0.3s ease-out',
            cursor: isPanning ? 'grabbing' : 'grab',
            userSelect: 'none'
          }}
        >
          {Array.from({ length: mapData.height }).map((_, y) =>
            Array.from({ length: mapData.width }).map((_, x) => {
              const element = getElementAt(x, y);
              const disciplesHere = getDisciplesAt(x, y);
              const isHovered = hoveredPosition?.x === x && hoveredPosition?.y === y;
              const isSelected = selectedDisciple && selectedDisciple.position.x === x && selectedDisciple.position.y === y;
              const isInRange = selectedDisciple ? isInMovementRange(x, y, selectedDisciple) : false;
              const isOutOfRange = selectedDisciple && !isInRange && !(selectedDisciple.position.x === x && selectedDisciple.position.y === y);

              const underAttack = element?.details?.under_attack;
              const isInvading = element?.element_type === 'Monster' && element?.details?.invading_location;

              return (
                <div
                  key={`${x}-${y}`}
                  className={`map-tile ${element ? getElementColorClass(element.element_type) : 'tile-empty'} ${isHovered ? 'tile-hovered' : ''} ${isSelected ? 'tile-selected' : ''}`}
                  onClick={() => handleTileClick(x, y)}
                  onMouseEnter={() => setHoveredPosition({x, y})}
                  onMouseLeave={() => setHoveredPosition(null)}
                  title={element ? element.name : `(${x}, ${y})`}
                  style={{
                    border: isSelected ? '3px solid #4299e1' :
                            underAttack ? `2px solid ${underAttack.is_demon ? '#c53030' : '#ed8936'}` :
                            isInvading ? '2px solid #fc8181' : undefined,
                    boxShadow: isSelected ? '0 0 15px #4299e1' :
                               underAttack ? `0 0 10px ${underAttack.is_demon ? '#c53030' : '#ed8936'}` :
                               isInvading ? '0 0 10px #fc8181' : undefined,
                    backgroundColor: isInRange && !isSelected ? 'rgba(66, 153, 225, 0.2)' :
                                     isOutOfRange ? 'rgba(0, 0, 0, 0.3)' : undefined,
                    cursor: selectedDisciple ? (isInRange ? 'pointer' : 'not-allowed') : (disciplesHere.length > 0 || element) ? 'pointer' : 'default',
                    opacity: isOutOfRange ? 0.5 : 1
                  }}
                >
                  {element && (
                    <span className="tile-icon">{getElementIcon(element.element_type, element.details)}</span>
                  )}
                  {disciplesHere.length > 0 && (
                    <span style={{
                      position: 'absolute',
                      fontSize: '24px',
                      fontWeight: 'bold',
                      zIndex: 10,
                      textShadow: '0 0 3px white, 0 0 5px white'
                    }}>
                      🧙
                    </span>
                  )}
                  {disciplesHere.length > 1 && (
                    <span style={{
                      position: 'absolute',
                      bottom: '2px',
                      right: '2px',
                      fontSize: '10px',
                      backgroundColor: '#4299e1',
                      color: 'white',
                      borderRadius: '50%',
                      width: '16px',
                      height: '16px',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      fontWeight: 'bold',
                      zIndex: 11
                    }}>
                      {disciplesHere.length}
                    </span>
                  )}
                  {underAttack && (
                    <span style={{
                      position: 'absolute',
                      top: '2px',
                      right: '2px',
                      fontSize: '12px'
                    }}>
                      {underAttack.is_demon ? '⚠️' : '🛡️'}
                    </span>
                  )}
                  {isInvading && !underAttack && (
                    <span style={{
                      position: 'absolute',
                      top: '2px',
                      right: '2px',
                      fontSize: '12px'
                    }}>
                      ⚔️
                    </span>
                  )}
                  <span className="tile-coords">{x},{y}</span>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};

// 导出辅助函数供外部使用
export { MapView };

export default MapView;
