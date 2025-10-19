# 🚀 快速启动指南

## 前置准备

### 检查环境

```bash
# 1. 检查 Rust 版本（需要 >= 1.70）
rustc --version

# 2. 检查 Node.js 版本（需要 >= 16）
node --version
npm --version

# 如果没有安装：
# - Rust: https://rustup.rs/
# - Node.js: https://nodejs.org/
```

## 方式一：最简单的启动方式（推荐）

### 步骤 1: 启动后端

打开**第一个终端**：

```bash
# 进入项目目录
cd /Users/xingao/xiuxian

# 运行游戏（会自动编译）
cargo run --release
```

你会看到菜单：
```
修仙宗门模拟器

欢迎来到修仙世界！

请选择游戏模式:
  [1] 交互模式 - 回合制，手动管理宗门
  [2] 自动模式 - 自动运行，观察模拟结果
  [3] Web服务器 - 启动HTTP API服务器

请选择 (1-3):
```

**输入 `3` 然后按回车**

你会看到：
```
🚀 Server running on http://localhost:3000
📚 API documentation: /api
```

✅ 后端启动成功！**保持这个终端运行**

### 步骤 2: 创建并启动前端

打开**第二个终端**：

```bash
# 进入项目目录
cd /Users/xingao/xiuxian

# 运行前端创建脚本
./create_frontend.sh
```

脚本会自动：
1. 创建 React + TypeScript 项目
2. 安装所有依赖（axios等）
3. 配置API代理
4. 创建所有必需的文件

**等待安装完成（可能需要几分钟）**

### 步骤 3: 启动前端开发服务器

```bash
# 进入前端目录
cd frontend

# 启动开发服务器
npm start
```

浏览器会自动打开 `http://localhost:3001`

✅ 完成！游戏开始运行！

---

## 方式二：手动测试 API（不需要前端）

如果你只想测试后端API，不需要前端界面：

### 1. 启动后端
```bash
cd /Users/xingao/xiuxian
cargo run --release
# 选择 3
```

### 2. 使用 curl 测试

**创建新游戏：**
```bash
curl -X POST http://localhost:3000/api/game/new \
  -H "Content-Type: application/json" \
  -d '{"sect_name":"青云宗"}'
```

你会得到类似的响应：
```json
{
  "success": true,
  "data": {
    "game_id": "a1b2c3d4-...",
    "sect": {
      "name": "青云宗",
      "year": 0,
      "resources": 1000,
      "reputation": 0,
      "disciples_count": 3
    },
    "state": "Running"
  }
}
```

**保存这个 game_id！** 后续请求都需要用到。

**开始新回合：**
```bash
# 替换 {game_id} 为上面获得的ID
curl -X POST http://localhost:3000/api/game/{game_id}/turn/start
```

**查看弟子：**
```bash
curl http://localhost:3000/api/game/{game_id}/disciples
```

**查看任务：**
```bash
curl http://localhost:3000/api/game/{game_id}/tasks
```

---

## 常见问题排查

### ❌ 问题 1: cargo build 失败

**错误信息：**
```
error: package `tokio v1.48.0` cannot be built because it requires rustc 1.71
```

**解决方法：**
```bash
# 更新 Rust
rustup update

# 或降级 tokio
cargo update -p tokio --precise 1.28.0
```

### ❌ 问题 2: 端口 3000 被占用

**错误信息：**
```
Address already in use (os error 48)
```

**解决方法：**
```bash
# 查找占用端口的进程
lsof -i :3000

# 杀死进程（替换 PID）
kill -9 PID

# 或修改端口（编辑 src/web_server.rs）
```

### ❌ 问题 3: 前端无法连接后端

**检查步骤：**

1. 确认后端正在运行
```bash
curl http://localhost:3000/api/game/new -X POST \
  -H "Content-Type: application/json" \
  -d '{"sect_name":"test"}'
```

2. 检查前端代理配置
```bash
cd frontend
cat package.json | grep proxy
# 应该显示: "proxy": "http://localhost:3000"
```

3. 重启前端
```bash
# 按 Ctrl+C 停止
npm start
```

### ❌ 问题 4: npm install 卡住

**解决方法：**
```bash
# 使用国内镜像
npm config set registry https://registry.npmmirror.com

# 清除缓存
npm cache clean --force

# 重试
npm install
```

### ❌ 问题 5: create_frontend.sh 无法执行

**错误信息：**
```
Permission denied
```

**解决方法：**
```bash
chmod +x create_frontend.sh
./create_frontend.sh
```

---

## 验证一切正常

### 1. 检查后端

访问：http://localhost:3000/api/game/new

应该看到JSON响应（可能是错误，因为是GET请求，但说明服务器在运行）

### 2. 检查前端

访问：http://localhost:3001

应该看到游戏界面和"创建新游戏"按钮

### 3. 完整测试流程

1. 在前端点击"创建新游戏"
2. 输入宗门名称（如"青云宗"）
3. 看到宗门信息和弟子列表
4. 点击"开始新回合"
5. 点击"自动分配任务"
6. 点击"结束回合"
7. 查看任务执行结果

---

## 停止服务

### 停止后端
在后端终端按 `Ctrl+C`

### 停止前端
在前端终端按 `Ctrl+C`

---

## 下次启动

**不需要重新创建前端！**

### 启动后端
```bash
cd /Users/xingao/xiuxian
cargo run --release
# 选择 3
```

### 启动前端
```bash
cd /Users/xingao/xiuxian/frontend
npm start
```

---

## 生产环境运行（可选）

### 构建优化版本

**后端：**
```bash
cargo build --release
./target/release/xiuxian_simulator
# 选择 3
```

**前端：**
```bash
cd frontend
npm run build
# 生成的文件在 build/ 目录
```

### 使用静态服务器部署前端
```bash
# 安装 serve
npm install -g serve

# 运行
cd frontend
serve -s build -p 3001
```

---

## 开发技巧

### 1. 后端自动重载（需要cargo-watch）

```bash
# 安装
cargo install cargo-watch

# 运行（代码改动自动重启）
cargo watch -x 'run --release'
```

### 2. 查看实时日志

```bash
# 后端详细日志
RUST_LOG=debug cargo run --release

# 前端
npm start
# 查看浏览器 Console
```

### 3. API测试工具推荐

- **Postman**: https://www.postman.com/
- **Thunder Client**: VS Code 插件
- **curl**: 命令行工具

### 4. 浏览器调试

按 `F12` 打开开发者工具：
- **Console**: 查看日志
- **Network**: 查看API请求
- **Application**: 查看localStorage（game_id存储位置）

---

## 完整的启动检查清单

- [ ] Rust 已安装（rustc --version）
- [ ] Node.js 已安装（node --version）
- [ ] 进入项目目录（cd /Users/xingao/xiuxian）
- [ ] 后端启动成功（cargo run --release，选择3）
- [ ] 看到"Server running on http://localhost:3000"
- [ ] 前端已创建（./create_frontend.sh）
- [ ] 进入前端目录（cd frontend）
- [ ] 前端启动成功（npm start）
- [ ] 浏览器自动打开 localhost:3001
- [ ] 可以创建游戏并看到界面

---

## 需要帮助？

1. **查看日志** - 大部分问题都会在终端显示错误信息
2. **检查端口** - 确保3000和3001端口未被占用
3. **重启服务** - 很多问题重启就能解决
4. **查看文档** - API_DESIGN.md, WEB_DEPLOYMENT_GUIDE.md

---

## 🎮 开始游戏！

一切正常后：

1. 浏览器访问 http://localhost:3001
2. 点击"创建新游戏"
3. 输入宗门名称
4. 享受游戏！

**祝你修仙之路顺利！** ✨
