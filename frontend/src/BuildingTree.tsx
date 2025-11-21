import React, { useState, useEffect } from 'react';
import { gameApi, BuildingDto, BuildingTreeResponse } from './api/gameApi';

interface BuildingTreeProps {
  gameId: string;
  onResourcesChanged: () => void;
}

const BuildingTree: React.FC<BuildingTreeProps> = ({ gameId, onResourcesChanged }) => {
  const [buildingTree, setBuildingTree] = useState<BuildingTreeResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [buildingInProgress, setBuildingInProgress] = useState<string | null>(null);
  const [message, setMessage] = useState<string>('');

  const loadBuildingTree = async () => {
    try {
      const data = await gameApi.getBuildingTree(gameId);
      setBuildingTree(data);
    } catch (error) {
      console.error('Failed to load building tree:', error);
    }
  };

  useEffect(() => {
    loadBuildingTree();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [gameId]);

  const handleBuild = async (buildingId: string, buildingName: string, cost: number) => {
    if (!window.confirm(`确定要建造「${buildingName}」吗？\n需要消耗 ${cost} 资源。`)) {
      return;
    }

    setBuildingInProgress(buildingId);
    try {
      const result = await gameApi.buildBuilding(gameId, buildingId);
      setMessage(`✅ ${result.message}`);

      // Reload building tree and trigger resource update
      await loadBuildingTree();
      onResourcesChanged();

      // Clear message after 3 seconds
      setTimeout(() => setMessage(''), 3000);
    } catch (error: any) {
      const errorMsg = error.response?.data?.error?.message || '建造失败';
      setMessage(`❌ ${errorMsg}`);
      setTimeout(() => setMessage(''), 3000);
    } finally {
      setBuildingInProgress(null);
    }
  };

  if (!buildingTree) {
    return <div style={{ padding: '20px', textAlign: 'center' }}>加载建筑树中...</div>;
  }

  // Group buildings by their level in the tree
  const buildingsByLevel: { [level: number]: BuildingDto[] } = {};

  const calculateLevel = (building: BuildingDto): number => {
    if (!building.parent_id) return 0;
    const parent = buildingTree.buildings.find(b => b.id === building.parent_id);
    return parent ? calculateLevel(parent) + 1 : 0;
  };

  buildingTree.buildings.forEach(building => {
    const level = calculateLevel(building);
    if (!buildingsByLevel[level]) {
      buildingsByLevel[level] = [];
    }
    buildingsByLevel[level].push(building);
  });

  const maxLevel = Math.max(...Object.keys(buildingsByLevel).map(Number));

  return (
    <div style={{
      padding: '20px',
      backgroundColor: '#f8f9fa',
      borderRadius: '8px',
      marginBottom: '20px'
    }}>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '20px',
        paddingBottom: '10px',
        borderBottom: '2px solid #dee2e6'
      }}>
        <h2 style={{ margin: 0, fontSize: '24px', color: '#2c3e50' }}>
          🏛️ 宗门建筑树
        </h2>
        <div style={{ textAlign: 'right', fontSize: '14px', color: '#6c757d' }}>
          <div>建筑进度: {buildingTree.built_count}/{buildingTree.total_buildings}</div>
          <div>成本倍数: {buildingTree.cost_multiplier}x</div>
          <div>可用资源: {buildingTree.available_resources}</div>
        </div>
      </div>

      {message && (
        <div style={{
          padding: '12px',
          marginBottom: '15px',
          borderRadius: '6px',
          backgroundColor: message.startsWith('✅') ? '#d4edda' : '#f8d7da',
          color: message.startsWith('✅') ? '#155724' : '#721c24',
          border: `1px solid ${message.startsWith('✅') ? '#c3e6cb' : '#f5c6cb'}`,
          animation: 'fadeIn 0.3s ease-in'
        }}>
          {message}
        </div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: '30px' }}>
        {[...Array(maxLevel + 1)].map((_, level) => {
          const buildings = buildingsByLevel[level] || [];
          if (buildings.length === 0) return null;

          return (
            <div key={level}>
              <div style={{
                fontSize: '12px',
                color: '#6c757d',
                marginBottom: '10px',
                fontWeight: 600
              }}>
                {level === 0 ? '基础建筑' : `第${level}层建筑`}
              </div>
              <div style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
                gap: '15px'
              }}>
                {buildings.map(building => (
                  <div
                    key={building.id}
                    style={{
                      padding: '15px',
                      border: '2px solid',
                      borderColor: building.is_built ? '#28a745' : building.can_build ? '#007bff' : '#dee2e6',
                      borderRadius: '8px',
                      backgroundColor: building.is_built ? '#e7f5e9' : building.can_build ? '#e7f3ff' : '#fff',
                      opacity: building.can_build || building.is_built ? 1 : 0.6,
                      transition: 'all 0.2s ease',
                      position: 'relative',
                      cursor: building.can_build ? 'pointer' : 'default'
                    }}
                    onMouseEnter={e => {
                      if (building.can_build || building.is_built) {
                        (e.currentTarget as HTMLElement).style.transform = 'translateY(-2px)';
                        (e.currentTarget as HTMLElement).style.boxShadow = '0 4px 8px rgba(0,0,0,0.1)';
                      }
                    }}
                    onMouseLeave={e => {
                      (e.currentTarget as HTMLElement).style.transform = 'translateY(0)';
                      (e.currentTarget as HTMLElement).style.boxShadow = 'none';
                    }}
                  >
                    {building.is_built && (
                      <div style={{
                        position: 'absolute',
                        top: '10px',
                        right: '10px',
                        backgroundColor: '#28a745',
                        color: 'white',
                        padding: '4px 8px',
                        borderRadius: '4px',
                        fontSize: '12px',
                        fontWeight: 600
                      }}>
                        已建造
                      </div>
                    )}

                    <div style={{
                      fontSize: '18px',
                      fontWeight: 600,
                      marginBottom: '8px',
                      color: building.is_built ? '#28a745' : building.can_build ? '#007bff' : '#495057'
                    }}>
                      {building.name}
                    </div>

                    <div style={{
                      fontSize: '13px',
                      color: '#6c757d',
                      marginBottom: '10px',
                      lineHeight: '1.5'
                    }}>
                      {building.description}
                    </div>

                    {building.effects.length > 0 && (
                      <div style={{ marginBottom: '10px' }}>
                        <div style={{ fontSize: '12px', color: '#6c757d', marginBottom: '5px' }}>
                          效果:
                        </div>
                        {building.effects.map((effect, idx) => (
                          <div key={idx} style={{
                            fontSize: '12px',
                            color: '#495057',
                            padding: '4px 8px',
                            backgroundColor: '#fff',
                            borderRadius: '4px',
                            marginBottom: '4px'
                          }}>
                            ✨ {effect}
                          </div>
                        ))}
                      </div>
                    )}

                    <div style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      marginTop: '10px',
                      paddingTop: '10px',
                      borderTop: '1px solid #dee2e6'
                    }}>
                      <div style={{ fontSize: '13px', color: '#6c757d' }}>
                        成本: <span style={{ fontWeight: 600, color: '#495057' }}>
                          {building.actual_cost}
                        </span>
                        {building.actual_cost !== building.base_cost && (
                          <span style={{ fontSize: '11px', marginLeft: '5px' }}>
                            (基础: {building.base_cost})
                          </span>
                        )}
                      </div>

                      {building.can_build && !building.is_built && (
                        <button
                          onClick={() => handleBuild(building.id, building.name, building.actual_cost)}
                          disabled={buildingInProgress === building.id || buildingTree.available_resources < building.actual_cost}
                          style={{
                            padding: '6px 14px',
                            backgroundColor: buildingTree.available_resources >= building.actual_cost ? '#007bff' : '#6c757d',
                            color: 'white',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: buildingTree.available_resources >= building.actual_cost ? 'pointer' : 'not-allowed',
                            fontSize: '13px',
                            fontWeight: 600,
                            transition: 'background-color 0.2s'
                          }}
                          onMouseEnter={e => {
                            if (buildingTree.available_resources >= building.actual_cost) {
                              (e.target as HTMLButtonElement).style.backgroundColor = '#0056b3';
                            }
                          }}
                          onMouseLeave={e => {
                            if (buildingTree.available_resources >= building.actual_cost) {
                              (e.target as HTMLButtonElement).style.backgroundColor = '#007bff';
                            }
                          }}
                        >
                          {buildingInProgress === building.id ? '建造中...' : '建造'}
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default BuildingTree;
