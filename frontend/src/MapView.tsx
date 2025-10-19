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
  const getElementIcon = (elementType: string): string => {
    switch(elementType) {
      case 'Village': return '🏘️';
      case 'Faction': return '⚔️';
      case 'DangerousLocation': return '⚠️';
      case 'SecretRealm': return '🌀';
      case 'Monster': return '👹';
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
      default: return 'tile-empty';
    }
  };

  // 渲染元素详情
  const renderElementDetails = (element: MapElement) => {
    const { details } = element;

    switch(element.element_type) {
      case 'Village':
        return (
          <>
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
          </>
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

              return (
                <div
                  key={`${x}-${y}`}
                  className={`map-tile ${element ? getElementColorClass(element.element_type) : 'tile-empty'} ${isHovered ? 'tile-hovered' : ''}`}
                  onClick={() => element && setSelectedElement(element)}
                  onMouseEnter={() => setHoveredPosition({x, y})}
                  onMouseLeave={() => setHoveredPosition(null)}
                  title={element ? element.name : `(${x}, ${y})`}
                >
                  {element && (
                    <span className="tile-icon">{getElementIcon(element.element_type)}</span>
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
              {getElementIcon(selectedElement.element_type)} {selectedElement.name}
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
