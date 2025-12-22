import React, { useState, useEffect } from 'react';
import { gameApi, HerbInventoryResponse, PillRecipe } from './api/gameApi';
import './AlchemyPanel.css';

interface AlchemyPanelProps {
  gameId: string;
  onRefineSuccess?: () => void;
}

const AlchemyPanel: React.FC<AlchemyPanelProps> = ({ gameId, onRefineSuccess }) => {
  const [herbs, setHerbs] = useState<HerbInventoryResponse | null>(null);
  const [recipes, setRecipes] = useState<PillRecipe[]>([]);
  const [loading, setLoading] = useState(true);
  const [refining, setRefining] = useState<string | null>(null);
  const [message, setMessage] = useState<{text: string, type: 'success' | 'error'} | null>(null);

  const loadData = async () => {
    try {
      setLoading(true);
      const [herbData, recipeData] = await Promise.all([
        gameApi.getHerbInventory(gameId),
        gameApi.getRecipes(gameId)
      ]);
      setHerbs(herbData);
      setRecipes(recipeData);
    } catch (err) {
      console.error('Failed to load alchemy data:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, [gameId]);

  const handleRefine = async (pillType: string) => {
    try {
      setRefining(pillType);
      const result = await gameApi.refinePill(gameId, pillType);
      setMessage({
        text: result.message,
        type: result.success ? 'success' : 'error'
      });
      await loadData();
      if (result.success && onRefineSuccess) {
        onRefineSuccess();
      }
      setTimeout(() => setMessage(null), 3000);
    } catch (err: any) {
      setMessage({
        text: err.message || '炼制失败',
        type: 'error'
      });
      setTimeout(() => setMessage(null), 3000);
    } finally {
      setRefining(null);
    }
  };

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

  if (loading) {
    return <div className="alchemy-panel loading">加载中...</div>;
  }

  return (
    <div className="alchemy-panel">
      {message && (
        <div className={`alchemy-message ${message.type}`}>
          {message.text}
        </div>
      )}

      <div className="alchemy-section">
        <h3>草药仓库 ({herbs?.total_count || 0})</h3>
        <div className="herb-grid">
          {herbs && herbs.herbs.length > 0 ? (
            herbs.herbs.map((herb, idx) => (
              <div key={idx} className="herb-item" style={{ borderColor: getQualityColor(herb.quality) }}>
                <span className="herb-name">{herb.name}</span>
                <span className="herb-quality" style={{ color: getQualityColor(herb.quality) }}>
                  {herb.quality}
                </span>
                <span className="herb-count">x{herb.count}</span>
              </div>
            ))
          ) : (
            <div className="empty-message">暂无草药，派遣弟子采集吧</div>
          )}
        </div>
      </div>

      <div className="alchemy-section">
        <h3>炼丹配方</h3>
        <div className="recipe-grid">
          {recipes.map((recipe) => (
            <div key={recipe.pill_type} className={`recipe-card ${recipe.can_craft ? 'craftable' : 'disabled'}`}>
              <div className="recipe-header">
                <span className="recipe-name">{recipe.name}</span>
                <span className="success-rate">{Math.round(recipe.success_rate * 100)}%</span>
              </div>
              <div className="recipe-desc">{recipe.description}</div>
              <div className="recipe-requirements">
                <div className="requirement">
                  <span className="req-label">草药:</span>
                  <span className="req-value" style={{ color: getQualityColor(recipe.required_herb_quality) }}>
                    {recipe.required_herb_count}x {recipe.required_herb_quality}
                  </span>
                </div>
                <div className="requirement">
                  <span className="req-label">资源:</span>
                  <span className="req-value">{recipe.resource_cost}</span>
                </div>
              </div>
              {!recipe.can_craft && recipe.reason && (
                <div className="recipe-reason">{recipe.reason}</div>
              )}
              <button
                className="refine-btn"
                disabled={!recipe.can_craft || refining !== null}
                onClick={() => handleRefine(recipe.pill_type)}
              >
                {refining === recipe.pill_type ? '炼制中...' : '炼制'}
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default AlchemyPanel;
