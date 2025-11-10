import React, { useState } from 'react';
import { MapData, MapElement } from './api/gameApi';
import './MapView.css';

interface MapViewProps {
  mapData: MapData;
}

const MapView: React.FC<MapViewProps> = ({ mapData }) => {
  const [selectedElement, setSelectedElement] = useState<MapElement | null>(null);
  const [hoveredPosition, setHoveredPosition] = useState<{x: number, y: number} | null>(null);

  // 获取指定位置的元素
  const getElementAt = (x: number, y: number): MapElement | undefined => {
    return mapData.elements.find(
      el => el.position.x === x && el.position.y === y
    );
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
          style={{
            gridTemplateColumns: `repeat(${mapData.width}, 1fr)`,
            gridTemplateRows: `repeat(${mapData.height}, 1fr)`
          }}
        >
          {Array.from({ length: mapData.height }).map((_, y) =>
            Array.from({ length: mapData.width }).map((_, x) => {
              const element = getElementAt(x, y);
              const isHovered = hoveredPosition?.x === x && hoveredPosition?.y === y;

              const underAttack = element?.details?.under_attack;
              const isInvading = element?.element_type === 'Monster' && element?.details?.invading_location;

              return (
                <div
                  key={`${x}-${y}`}
                  className={`map-tile ${element ? getElementColorClass(element.element_type) : 'tile-empty'} ${isHovered ? 'tile-hovered' : ''}`}
                  onClick={() => element && setSelectedElement(element)}
                  onMouseEnter={() => setHoveredPosition({x, y})}
                  onMouseLeave={() => setHoveredPosition(null)}
                  title={element ? element.name : `(${x}, ${y})`}
                  style={{
                    border: underAttack ? `2px solid ${underAttack.is_demon ? '#c53030' : '#ed8936'}` :
                            isInvading ? '2px solid #fc8181' : undefined,
                    boxShadow: underAttack ? `0 0 10px ${underAttack.is_demon ? '#c53030' : '#ed8936'}` :
                               isInvading ? '0 0 10px #fc8181' : undefined
                  }}
                >
                  {element && (
                    <span className="tile-icon">{getElementIcon(element.element_type, element.details)}</span>
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

      {selectedElement && (
        <div className="element-details-panel">
          <div className="details-header">
            <h3>
              {getElementIcon(selectedElement.element_type, selectedElement.details)} {selectedElement.name}
            </h3>
            <button
              className="close-btn"
              onClick={() => setSelectedElement(null)}
            >
              ✕
            </button>
          </div>
          <div className="details-body">
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
    </div>
  );
};

export default MapView;
