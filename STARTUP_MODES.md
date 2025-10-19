# 启动模式说明

## 🚀 Web 服务器模式的启动流程

### 方式1: 使用命令行参数（最快，无需任何交互）

```bash
cargo run --release -- --web
```

**流程：**
1. 程序启动
2. 检测到 `--web` 参数
3. **立即**打印启动信息
4. **立即**启动服务器，监听端口3000
5. ✅ **无需按回车或任何其他输入**

**输出示例：**
```
🚀 启动 Web 服务器模式...
📍 服务器地址: http://localhost:3000
📚 API 文档: 见 API_DESIGN.md
⛔ 按 Ctrl+C 停止服务器

🚀 Server running on http://localhost:3000
📚 API documentation: /api
```

### 方式2: 使用启动脚本（最简单）

```bash
./start_server.sh
```

**流程：**
- 与方式1完全相同
- 脚本内部调用 `cargo run --release -- --web`

### 方式3: 菜单模式

```bash
cargo run --release
```

然后选择 `3`

**流程：**
1. 显示菜单
2. 等待用户输入选择（1/2/3）
3. 用户输入 `3` 并按回车
4. **立即**打印启动信息
5. **立即**启动服务器
6. ✅ **选择后无需再按回车**

## ⚠️ 重要说明

### Web 服务器模式特点：
- ✅ **启动后立即运行** - 不需要任何额外的按键
- ✅ **持续监听请求** - 自动处理所有HTTP请求
- ✅ **无需手动交互** - 完全自动化运行
- ⛔ **停止服务器** - 只需按 `Ctrl+C`

### 与其他模式对比：

| 模式 | 启动方式 | 是否需要交互 | 运行方式 |
|------|---------|-------------|---------|
| Web服务器 | `--web` 或菜单选3 | ❌ 启动后无需交互 | 自动监听HTTP请求 |
| 交互模式 | 菜单选1 | ✅ 每回合需要操作 | 回合制手动管理 |
| 自动模式 | 菜单选2 | ✅ 启动时需要输入 | 自动运行指定回合数 |

## 📋 代码验证

### main.rs 中的 Web 模式处理：

```rust
Some(3) => {
    // Web服务器模式
    println!("\n🚀 启动 Web 服务器模式...");
    println!("📍 服务器地址: http://localhost:3000");
    println!("📚 API 文档: 见 API_DESIGN.md");
    println!("⛔ 按 Ctrl+C 停止服务器\n");

    web_server::start_server().await;  // ← 直接启动，无等待
}
```

### web_server.rs 中的启动函数：

```rust
pub async fn start_server() {
    let app = create_router();

    let addr = "0.0.0.0:3000".parse().unwrap();

    println!("🚀 Server running on http://localhost:3000");
    println!("📚 API documentation: /api");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();  // ← 直接启动服务器，无等待输入
}
```

## ✅ 确认

- ✅ 代码中**没有** `UI::wait_for_enter()` 调用
- ✅ 代码中**没有** `UI::get_input()` 调用
- ✅ 代码中**没有** `read_line()` 调用
- ✅ 启动后**立即**进入服务器监听循环
- ✅ **完全自动化**，无需人工干预

## 🎯 推荐使用方式

**开发/测试环境：**
```bash
# 终端1
./start_server.sh

# 终端2
cd frontend && npm start
```

**快速启动：**
```bash
cargo run --release -- --web
```

**首次使用/学习：**
```bash
cargo run --release
# 选择 3
```
