# 修仙模拟器 Web 部署指南

## 🏗️ 项目架构

```
修仙模拟器 Web 版
├── 后端 (Rust + Axum)
│   ├── HTTP API 服务器
│   ├── RESTful 端点
│   └── 游戏状态管理
└── 前端 (React + TypeScript)
    ├── 游戏界面
    ├── 任务管理
    └── 弟子管理
```

## 🚀 后端部署

### 1. 准备环境

**Rust 版本要求**: >= 1.70.0

```bash
# 检查 Rust 版本
rustc --version

# 如果版本不够，升级 Rust
rustup update
```

### 2. 编译后端

```bash
cd xiuxian

# 开发模式
cargo build

# 生产模式（推荐）
cargo build --release
```

### 3. 启动服务器

```bash
# 方式1: 直接运行
cargo run --release

# 然后选择 3 (Web服务器模式)

# 方式2: 运行编译后的二进制文件
./target/release/xiuxian_simulator
# 选择 3
```

服务器将在 `http://localhost:3000` 启动

### 4. 测试 API

```bash
# 创建新游戏
curl -X POST http://localhost:3000/api/game/new \
  -H "Content-Type: application/json" \
  -d '{"sect_name":"青云宗"}'

# 响应示例：
# {
#   "success": true,
#   "data": {
#     "game_id": "uuid-here",
#     "sect": { ... },
#     "state": "Running"
#   }
# }

# 获取游戏信息
curl http://localhost:3000/api/game/{game_id}

# 获取弟子列表
curl http://localhost:3000/api/game/{game_id}/disciples
```

## ⚛️ 前端部署

### 1. 创建 React 项目

```bash
# 在项目根目录
npx create-react-app frontend --template typescript
cd frontend
```

### 2. 安装依赖

```bash
npm install axios
npm install -D tailwindcss postcss autoprefixer
npm install react-router-dom
npm install @tanstack/react-query
```

### 3. 配置代理（开发环境）

编辑 `frontend/package.json`，添加：

```json
{
  "proxy": "http://localhost:3000"
}
```

### 4. 目录结构

```
frontend/
├── public/
├── src/
│   ├── api/           # API 调用
│   │   └── gameApi.ts
│   ├── components/    # React 组件
│   │   ├── Game/
│   │   ├── Disciples/
│   │   ├── Tasks/
│   │   └── Statistics/
│   ├── types/         # TypeScript 类型
│   │   └── game.ts
│   ├── App.tsx
│   └── index.tsx
└── package.json
```

### 5. 核心代码示例

#### `src/api/gameApi.ts`
```typescript
import axios from 'axios';

const API_BASE = process.env.REACT_APP_API_URL || 'http://localhost:3000/api';

export interface GameInfo {
  game_id: string;
  sect: {
    name: string;
    year: number;
    resources: number;
    reputation: number;
    disciples_count: number;
  };
  state: string;
}

export const gameApi = {
  // 创建新游戏
  createGame: async (sectName: string): Promise<GameInfo> => {
    const response = await axios.post(`${API_BASE}/game/new`, {
      sect_name: sectName
    });
    return response.data.data;
  },

  // 获取游戏信息
  getGame: async (gameId: string): Promise<GameInfo> => {
    const response = await axios.get(`${API_BASE}/game/${gameId}`);
    return response.data.data;
  },

  // 开始新回合
  startTurn: async (gameId: string) => {
    const response = await axios.post(`${API_BASE}/game/${gameId}/turn/start`);
    return response.data.data;
  },

  // 获取弟子列表
  getDisciples: async (gameId: string) => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/disciples`);
    return response.data.data;
  },

  // 获取任务列表
  getTasks: async (gameId: string) => {
    const response = await axios.get(`${API_BASE}/game/${gameId}/tasks`);
    return response.data.data;
  },

  // 分配任务
  assignTask: async (gameId: string, taskId: number, discipleId: number) => {
    const response = await axios.post(
      `${API_BASE}/game/${gameId}/tasks/${taskId}/assign`,
      { disciple_id: discipleId }
    );
    return response.data.data;
  },

  // 结束回合
  endTurn: async (gameId: string, assignments: Array<{task_id: number, disciple_id: number}>) => {
    const response = await axios.post(
      `${API_BASE}/game/${gameId}/turn/end`,
      { assignments }
    );
    return response.data.data;
  }
};
```

#### `src/App.tsx` 示例
```typescript
import React, { useState, useEffect } from 'react';
import { gameApi } from './api/gameApi';

function App() {
  const [gameId, setGameId] = useState<string | null>(null);
  const [gameInfo, setGameInfo] = useState(null);
  const [disciples, setDisciples] = useState([]);
  const [tasks, setTasks] = useState([]);

  const createNewGame = async () => {
    const sectName = prompt('输入宗门名称:', '青云宗');
    if (!sectName) return;

    const game = await gameApi.createGame(sectName);
    setGameId(game.game_id);
    loadGameInfo(game.game_id);
  };

  const loadGameInfo = async (id: string) => {
    const info = await gameApi.getGame(id);
    setGameInfo(info);

    const disciplesList = await gameApi.getDisciples(id);
    setDisciples(disciplesList);

    const tasksList = await gameApi.getTasks(id);
    setTasks(tasksList);
  };

  return (
    <div className="App">
      <h1>修仙宗门模拟器</h1>

      {!gameId ? (
        <button onClick={createNewGame}>创建新游戏</button>
      ) : (
        <div>
          {/* 游戏界面 */}
          <div>宗门: {gameInfo?.sect.name}</div>
          <div>年份: {gameInfo?.sect.year}</div>

          {/* 弟子列表 */}
          <h2>弟子列表</h2>
          <ul>
            {disciples.map(d => (
              <li key={d.id}>{d.name} - {d.cultivation.level}</li>
            ))}
          </ul>

          {/* 任务列表 */}
          <h2>任务列表</h2>
          <ul>
            {tasks.map(t => (
              <li key={t.id}>{t.name}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

export default App;
```

### 6. 启动前端

```bash
cd frontend
npm start
```

前端将在 `http://localhost:3001` 启动（自动避开3000端口）

## 🔗 完整部署流程

### 开发环境

1. **终端1 - 启动后端**
   ```bash
   cd xiuxian
   cargo run --release
   # 选择 3 (Web服务器)
   ```

2. **终端2 - 启动前端**
   ```bash
   cd xiuxian/frontend
   npm start
   ```

3. **访问应用**
   - 前端: http://localhost:3001
   - 后端API: http://localhost:3000/api

### 生产环境

#### 后端
```bash
# 编译
cargo build --release

# 运行（使用systemd或其他进程管理器）
./target/release/xiuxian_simulator
```

#### 前端
```bash
# 构建
npm run build

# 部署 build/ 目录到 nginx 或其他静态服务器
```

#### Nginx 配置示例
```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端静态文件
    location / {
        root /path/to/frontend/build;
        try_files $uri /index.html;
    }

    # 代理API请求到后端
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## 🐳 Docker 部署（可选）

### Dockerfile (后端)
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/xiuxian_simulator /usr/local/bin/
EXPOSE 3000
CMD ["xiuxian_simulator"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  backend:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info

  frontend:
    build: ./frontend
    ports:
      - "80:80"
    depends_on:
      - backend
```

## 🔧 环境变量配置

### 后端
```bash
# .env
PORT=3000
RUST_LOG=info
CORS_ORIGINS=http://localhost:3001
```

### 前端
```bash
# .env
REACT_APP_API_URL=http://localhost:3000/api
```

## 📊 性能优化

### 后端
- 使用 `--release` 模式编译
- 配置适当的日志级别
- 使用连接池（如果需要数据库）

### 前端
- 代码分割
- 懒加载组件
- 优化图片和资源
- 使用 React Query 缓存

## 🔍 故障排查

### 后端无法启动
```bash
# 检查端口占用
lsof -i :3000

# 查看日志
RUST_LOG=debug cargo run
```

### 前端无法连接后端
- 检查CORS配置
- 确认代理设置
- 查看浏览器Console错误

### API返回错误
- 检查请求格式
- 查看Network面板
- 确认game_id正确

## 📚 相关文档

- API设计: `API_DESIGN.md`
- 游戏机制: `GUIDE.md`
- 架构说明: `ARCHITECTURE.md`

## ✅ 部署检查清单

- [ ] Rust 版本 >= 1.70
- [ ] Node.js 版本 >= 16
- [ ] 后端编译成功
- [ ] 前端构建成功
- [ ] 后端服务器启动
- [ ] 前端可以访问
- [ ] API调用正常
- [ ] CORS配置正确
- [ ] 生产环境安全配置

## 🎯 下一步

1. 完善React组件
2. 添加用户认证
3. 实现WebSocket实时更新
4. 添加数据持久化
5. 部署到云服务器

---

**需要帮助？**
- 查看 API_DESIGN.md 了解API详情
- 运行测试确保功能正常
- 检查日志排查问题
