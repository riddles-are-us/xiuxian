import React from 'react';
import { MapElement, Disciple } from './api/gameApi';
import './MapElementDetails.css';

// 获取元素图标
export const getElementIcon = (elementType: string, details?: any): string => {
  switch(elementType) {
    case 'Village': return '🏘️';
    case 'Faction': return '⚔️';
    case 'DangerousLocation': return '⚠️';
    case 'SecretRealm': return '🌀';
    case 'Monster': return '👹';
    case 'Terrain': {
      const terrainType = details?.terrain_type;
      if (terrainType === 'Mountain') return '⛰️';
      if (terrainType === 'Water') return '💧';
      if (terrainType === 'Forest') return '🌲';
      if (terrainType === 'Plain') return '🌾';
      return '🗺️';
    }
    case 'Herb': {
      // 根据成熟度显示不同图标
      if (details?.is_mature) return '🌿';
      return '🌱';
    }
    default: return '?';
  }
};

// 渲染攻击警告
export const renderAttackWarning = (attackInfo?: any) => {
  if (!attackInfo) return null;

  return (
    <div style={{
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
export const renderElementDetails = (element: MapElement) => {
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
            <div style={{
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
    case 'Herb': {
      const qualityColors: { [key: string]: string } = {
        '普通': '#718096',
        '良品': '#48bb78',
        '稀有': '#4299e1',
        '珍品': '#9f7aea',
        '仙品': '#f6ad55',
      };
      const qualityColor = qualityColors[details.quality || '普通'] || '#718096';
      const growthPercent = Math.round((details.growth_stage || 0) / (details.max_growth || 100) * 100);

      return (
        <>
          <div className="detail-row">
            <span className="detail-label">品质:</span>
            <span className="detail-value" style={{ color: qualityColor, fontWeight: 'bold' }}>
              {details.quality || '普通'}
            </span>
          </div>
          <div className="detail-row">
            <span className="detail-label">生长度:</span>
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
                  width: `${growthPercent}%`,
                  height: '100%',
                  backgroundColor: details.is_mature ? '#48bb78' : '#667eea',
                  borderRadius: '4px'
                }}></span>
              </span>
              {growthPercent}%
            </span>
          </div>
          <div className="detail-row">
            <span className="detail-label">状态:</span>
            <span className="detail-value" style={{
              color: details.is_mature ? '#48bb78' : '#ed8936',
              fontWeight: 'bold'
            }}>
              {details.is_mature ? '✓ 成熟' : '生长中...'}
            </span>
          </div>
          <div style={{
            marginTop: '8px',
            padding: '8px',
            backgroundColor: '#fffaf0',
            borderRadius: '4px',
            fontSize: '12px',
            color: '#744210'
          }}>
            💡 怪物路过时会吞噬草药提升等级
          </div>
        </>
      );
    }
    default:
      return null;
  }
};

// 地图元素详情面板组件
interface MapElementDetailsPanelProps {
  element: MapElement | null;
  disciple: Disciple | null;
  onClose: () => void;
}

export const MapElementDetailsPanel: React.FC<MapElementDetailsPanelProps> = ({
  element,
  disciple,
  onClose
}) => {
  if (!element && !disciple) return null;

  return (
    <div style={{
      position: 'absolute',
      right: '420px',
      top: '70px',
      width: '320px',
      maxHeight: 'calc(100vh - 140px)',
      background: 'linear-gradient(180deg, #2d3748 0%, #1a202c 100%)',
      borderRadius: '12px',
      padding: '1rem',
      boxShadow: '0 4px 20px rgba(0,0,0,0.5)',
      border: '1px solid rgba(255,255,255,0.2)',
      zIndex: 60,
      overflowY: 'auto'
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        marginBottom: '1rem',
        paddingBottom: '0.5rem',
        borderBottom: '1px solid rgba(255,255,255,0.1)'
      }}>
        <h3 style={{
          color: 'white',
          fontSize: '1.1rem',
          fontWeight: 'bold',
          margin: 0
        }}>
          {element && `${getElementIcon(element.element_type, element.details)} ${element.name}`}
          {disciple && `🧙 ${disciple.name}`}
        </h3>
        <button
          onClick={onClose}
          style={{
            background: 'transparent',
            border: 'none',
            color: 'white',
            fontSize: '1.5rem',
            cursor: 'pointer',
            padding: '0.25rem 0.5rem',
            borderRadius: '4px',
            transition: 'background 0.2s'
          }}
          onMouseEnter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.1)'}
          onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
        >
          ✕
        </button>
      </div>

      <div style={{ color: '#e2e8f0', fontSize: '0.9rem' }}>
        {element && (
          <>
            <div className="detail-row">
              <span className="detail-label">类型:</span>
              <span className="detail-value">{element.element_type}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">位置:</span>
              <span className="detail-value">
                ({element.position.x}, {element.position.y})
              </span>
            </div>
            {renderElementDetails(element)}
          </>
        )}

        {disciple && (
          <>
            <div className="detail-row">
              <span className="detail-label">类型:</span>
              <span className="detail-value">{disciple.disciple_type}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">修为:</span>
              <span className="detail-value">
                {disciple.cultivation.level} {disciple.cultivation.sub_level}
              </span>
            </div>
            <div className="detail-row">
              <span className="detail-label">位置:</span>
              <span className="detail-value">
                ({disciple.position.x}, {disciple.position.y})
              </span>
            </div>
            <div className="detail-row">
              <span className="detail-label">移动范围:</span>
              <span className="detail-value" style={{
                color: '#4299e1',
                fontWeight: 'bold'
              }}>
                {disciple.movement_range} 格
              </span>
            </div>
            <div className="detail-row">
              <span className="detail-label">剩余移动:</span>
              <span className="detail-value" style={{
                color: disciple.moves_remaining === 0 ? '#f56565' :
                       disciple.moves_remaining < disciple.movement_range / 2 ? '#ed8936' : '#48bb78',
                fontWeight: 'bold'
              }}>
                {disciple.moves_remaining} 格
              </span>
            </div>
            <div className="detail-row">
              <span className="detail-label">精力:</span>
              <span className="detail-value">{disciple.energy}/100</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">体魄:</span>
              <span className="detail-value">{disciple.constitution}/100</span>
            </div>
            {disciple.current_task_info && (
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
                  {disciple.current_task_info.task_name}
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
          </>
        )}
      </div>
    </div>
  );
};
