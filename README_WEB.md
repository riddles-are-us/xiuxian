# 修仙宗门模拟器 - Web版

## 🌐 项目简介

这是修仙宗门模拟器的Web版本，采用前后端分离架构：

- **后端**: Rust + Axum (HTTP API服务器)
- **前端**: React + TypeScript (现代化Web界面)

## ⚡ 快速开始

### 方式1: 一键启动（推荐）

```bash
# 1. 启动后端（终端1）
cargo run --release
# 选择 3 (Web服务器模式)

# 2. 创建并启动前端（终端2）
./create_frontend.sh
# 脚本会自动创建React项目并安装依赖

# 3. 访问应用
# 打开浏览器访问: http://localhost:3001
```

### 方式2: 手动设置

**后端:**
```bash
# 编译并运行
cargo build --release
cargo run --release
# 选择选项 3

# 服务器启动在: http://localhost:3000
```

**前端:**
```bash
# 创建React项目
npx create-react-app frontend --template typescript
cd frontend

# 安装依赖
npm install axios react-router-dom @tanstack/react-query

# 复制示例代码（从 WEB_DEPLOYMENT_GUIDE.md）

# 启动开发服务器
npm start

# 前端启动在: http://localhost:3001
```

## 🎮 功能特性

### 已实现的API

✅ **游戏管理**
- POST `/api/game/new` - 创建新游戏
- GET `/api/game/{game_id}` - 获取游戏信息

✅ **回合管理**
- POST `/api/game/{game_id}/turn/start` - 开始新回合
- POST `/api/game/{game_id}/turn/end` - 结束回合

✅ **弟子管理**
- GET `/api/game/{game_id}/disciples` - 获取所有弟子
- GET `/api/game/{game_id}/disciples/{id}` - 获取单个弟子

✅ **任务管理**
- GET `/api/game/{game_id}/tasks` - 获取任务列表
- POST `/api/game/{game_id}/tasks/{id}/assign` - 分配任务
- DELETE `/api/game/{game_id}/tasks/{id}/assign` - 取消分配
- POST `/api/game/{game_id}/tasks/auto-assign` - 自动分配

✅ **统计信息**
- GET `/api/game/{game_id}/statistics` - 获取统计数据

✅ **渡劫系统**
- GET `/api/game/{game_id}/tribulation/candidates` - 获取候选人
- POST `/api/game/{game_id}/tribulation` - 执行渡劫

### 前端功能

✅ **游戏界面**
- 创建新游戏
- 查看宗门状态
- 实时更新资源和声望

✅ **弟子管理**
- 查看所有弟子
- 显示修为进度
- 查看当前任务状态

✅ **任务管理**
- 查看可用任务
- 手动分配任务
- 自动分配任务
- 查看任务奖励

✅ **回合控制**
- 开始新回合
- 结束回合
- 查看回合结果

## 📁 项目结构

```
xiuxian/
├── src/
│   ├── api_types.rs        # API数据类型
│   ├── web_server.rs       # HTTP服务器
│   ├── interactive.rs      # 游戏逻辑
│   ├── ... (其他游戏模块)
│   └── main.rs             # 程序入口
├── frontend/               # React前端（运行脚本后创建）
│   ├── src/
│   │   ├── api/
│   │   │   └── gameApi.ts # API调用封装
│   │   ├── App.tsx        # 主应用组件
│   │   └── App.css        # 样式
│   └── package.json
├── create_frontend.sh      # 前端创建脚本
├── API_DESIGN.md          # API设计文档
└── WEB_DEPLOYMENT_GUIDE.md # 部署指南
```

## 🔌 API使用示例

### 创建游戏
```bash
curl -X POST http://localhost:3000/api/game/new \
  -H "Content-Type: application/json" \
  -d '{"sect_name":"青云宗"}'
```

### 获取弟子列表
```bash
curl http://localhost:3000/api/game/{game_id}/disciples
```

### 分配任务
```bash
curl -X POST http://localhost:3000/api/game/{game_id}/tasks/1/assign \
  -H "Content-Type: application/json" \
  -d '{"disciple_id":3}'
```

## 🎨 前端界面预览

**主界面:**
- 顶部：宗门名称、年份、资源、声望
- 中部：操作按钮（开始回合、自动分配、结束回合）
- 左侧：弟子列表（卡片式展示）
- 右侧：任务列表（带分配按钮）

**弟子卡片:**
```
┌─────────────────┐
│   云飞扬         │
│ 类型: 内门       │
│ 修为: 筑基 (65%) │
│ 道心: 75        │
│ 年龄: 85/300    │
│ 📋 讨伐噬魂虎    │
└─────────────────┘
```

**任务卡片:**
```
┌────────────────────────┐
│ 讨伐噬魂虎              │
│ Combat                 │
│ 修为+15 资源+40 声望+25 │
│ ✓ 已分配给 云飞扬       │
│ 或                     │
│ [分配给 张三] [分配...]  │
└────────────────────────┘
```

## 🚀 部署

### 开发环境
```bash
# 终端1: 后端
cargo run --release
选择 3

# 终端2: 前端
cd frontend
npm start
```

### 生产环境

**后端:**
```bash
cargo build --release
./target/release/xiuxian_simulator
```

**前端:**
```bash
cd frontend
npm run build
# 部署 build/ 到静态服务器
```

详细部署说明见 `WEB_DEPLOYMENT_GUIDE.md`

## 🔧 配置

### 环境变量

**后端 (.env):**
```bash
PORT=3000
RUST_LOG=info
```

**前端 (.env):**
```bash
REACT_APP_API_URL=http://localhost:3000/api
```

## 📊 技术栈

### 后端
- **Axum** - 现代化Rust Web框架
- **Tokio** - 异步运行时
- **Serde** - JSON序列化
- **Tower** - 中间件
- **UUID** - 游戏ID生成
- **DashMap** - 并发HashMap

### 前端
- **React 18** - UI框架
- **TypeScript** - 类型安全
- **Axios** - HTTP客户端
- **React Router** - 路由（可选）
- **TanStack Query** - 数据获取（可选）
- **Tailwind CSS** - 样式（可选）

## 🐛 故障排查

### 后端无法启动
```bash
# 检查端口
lsof -i :3000

# 查看详细日志
RUST_LOG=debug cargo run
```

### 前端连接失败
- 检查后端是否运行在 3000 端口
- 确认 proxy 配置正确
- 查看浏览器 Console 错误

### CORS错误
后端已配置允许所有来源，如果还有问题：
- 检查请求头
- 确认API路径正确

## 📚 相关文档

- **API文档**: `API_DESIGN.md`
- **部署指南**: `WEB_DEPLOYMENT_GUIDE.md`
- **游戏机制**: `GUIDE.md`
- **架构设计**: `ARCHITECTURE.md`

## 🎯 下一步计划

- [ ] 完善前端UI组件
- [ ] 添加用户认证
- [ ] 实现WebSocket实时更新
- [ ] 添加数据持久化（数据库）
- [ ] 游戏存档功能
- [ ] 多人游戏支持
- [ ] 移动端适配
- [ ] Docker化部署

## 💡 使用技巧

1. **游戏ID保存**: 前端会自动保存game_id到localStorage
2. **自动分配**: 使用自动分配快速开始游戏
3. **任务筛选**: 只显示空闲弟子可分配
4. **实时更新**: 每次操作后自动刷新数据

## ⚠️ 注意事项

1. 后端不持久化数据，重启后游戏丢失
2. 建议使用 release 模式运行后端
3. 生产环境请配置适当的CORS策略
4. 单个服务器实例支持多个游戏（通过game_id区分）

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📝 许可证

本项目为学习示例项目

---

**享受Web版修仙体验！** 🎮✨
