# 如何运行修仙宗门模拟器

## ✅ 构建成功

项目已经成功编译！使用 Rust 1.71.0。

## 🚀 启动步骤

### 第一步：启动后端服务器

打开**第一个终端**，使用以下**任一方式**启动：

#### 方式A：使用启动脚本（最简单）🎯

```bash
cd /Users/xingao/xiuxian
./start_server.sh
```

#### 方式B：直接启动Web服务器（推荐）✨

```bash
cd /Users/xingao/xiuxian
cargo run --release -- --web
```

或使用短参数：
```bash
cargo run --release -- -w
```

你会立即看到：
```
🚀 启动 Web 服务器模式...
📍 服务器地址: http://localhost:3000
📚 API 文档: 见 API_DESIGN.md
⛔ 按 Ctrl+C 停止服务器

🚀 Server running on http://localhost:3000
📚 API documentation: /api
```

✅ 后端启动成功！**保持这个终端运行**

#### 方式C：通过菜单选择

```bash
cd /Users/xingao/xiuxian
cargo run --release
```

然后输入 `3` 并按回车

✅ 后端启动成功！**保持这个终端运行**

### 第二步：安装前端依赖

打开**第二个终端**：

```bash
cd /Users/xingao/xiuxian/frontend
npm install
```

等待安装完成（首次可能需要几分钟）

### 第三步：启动前端

```bash
npm start
```

浏览器会自动打开 `http://localhost:3001`

✅ 完成！开始游戏！

## 🎮 使用界面

1. **创建新游戏** - 点击"创建新游戏"按钮，输入宗门名称
2. **查看信息** - 界面显示宗门资源、声望、弟子列表、任务列表
3. **分配任务** - 点击任务下的按钮将任务分配给弟子
4. **开始回合** - 点击"开始新回合"生成新任务
5. **自动分配** - 点击"自动分配任务"让系统智能分配
6. **结束回合** - 点击"结束回合"执行任务并查看结果

## 📊 技术栈

### 后端
- **Rust 1.71.0** - 编程语言
- **Axum 0.6** - Web 框架
- **Tokio 1.28** - 异步运行时
- **Tower-HTTP 0.4** - CORS 中间件
- **DashMap 5.5** - 并发状态管理

### 前端
- **React 18** - UI 框架
- **TypeScript** - 类型安全
- **Axios** - HTTP 客户端

## ⚠️ 注意事项

1. **后端必须先启动** - 前端依赖后端 API
2. **端口占用** - 后端使用 3000，前端使用 3001
3. **数据不持久化** - 重启后端会丢失游戏数据
4. **游戏ID保存** - 前端会将 game_id 保存在 localStorage

## 🔧 故障排查

### 后端无法启动

```bash
# 检查端口占用
lsof -i :3000

# 杀死占用进程
kill -9 <PID>
```

### 前端连接失败

1. 确认后端正在运行
2. 检查浏览器控制台错误（F12）
3. 确认 package.json 中有 `"proxy": "http://localhost:3000"`

### 编译错误

```bash
# 清理并重新编译
cargo clean
cargo build --release
```

## 📝 下次启动

### 最快速启动 🚀

**终端1 - 后端：**
```bash
cd /Users/xingao/xiuxian
./start_server.sh
```

**终端2 - 前端：**
```bash
cd /Users/xingao/xiuxian/frontend
npm start
```

### 使用命令行参数

**终端1 - 后端：**
```bash
cd /Users/xingao/xiuxian
cargo run --release -- --web
```

**终端2 - 前端：**
```bash
cd /Users/xingao/xiuxian/frontend
npm start
```

### 完整命令参数

```bash
# 启动Web服务器
cargo run --release -- --web
cargo run --release -- -w

# 启动交互模式
cargo run --release

# 查看帮助
cargo run --release -- --help
```

---

**祝你修仙之路顺利！** ✨
