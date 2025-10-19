# 贡献指南

感谢你对修仙宗门模拟器项目的关注！

## 📋 开发环境

### 必要工具

- Rust 1.71+ ([安装指南](https://rustup.rs/))
- Node.js 16+ ([下载](https://nodejs.org/))
- Git

### 设置开发环境

```bash
# 1. Clone 项目
git clone <repository-url>
cd xiuxian

# 2. 编译后端
cargo build --release

# 3. 设置前端
cd frontend
npm install
```

## 🔧 开发工作流

### 后端开发

```bash
# 编译
cargo build

# 运行测试
cargo test

# 运行 CLI 模式
cargo run

# 运行 Web 服务器
cargo run -- --web

# 检查代码
cargo clippy

# 格式化代码
cargo fmt
```

### 前端开发

```bash
cd frontend

# 启动开发服务器
npm start

# 构建生产版本
npm run build

# 运行测试
npm test
```

## 📝 提交规范

### Commit Message 格式

```
<type>: <subject>

<body>

<footer>
```

### Type 类型

- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具相关

### 示例

```bash
git commit -m "feat: 添加弟子突破动画效果

- 添加突破成功的视觉反馈
- 增加音效提示
- 优化UI交互流程

Closes #123"
```

## 🌿 分支管理

### 分支命名

- `main` - 主分支，稳定版本
- `develop` - 开发分支
- `feature/xxx` - 新功能分支
- `fix/xxx` - 修复分支
- `docs/xxx` - 文档分支

### 工作流程

```bash
# 1. 从 develop 创建功能分支
git checkout develop
git pull
git checkout -b feature/new-feature

# 2. 开发并提交
git add .
git commit -m "feat: 描述"

# 3. 推送到远程
git push origin feature/new-feature

# 4. 创建 Pull Request
```

## 🧪 测试要求

### 后端测试

新功能需要添加单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cultivation_breakthrough() {
        // 测试代码
    }
}
```

### 前端测试

UI 组件需要测试：

```typescript
import { render, screen } from '@testing-library/react';

test('renders game title', () => {
  render(<App />);
  const title = screen.getByText(/修仙宗门模拟器/i);
  expect(title).toBeInTheDocument();
});
```

## 📐 代码规范

### Rust 代码

- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 检查
- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 公共 API 必须有文档注释

### TypeScript 代码

- 使用 ESLint 检查
- 遵循 React 最佳实践
- 组件必须有 TypeScript 类型

## 🎯 提交 PR 前检查清单

- [ ] 代码已通过 `cargo test` 和 `npm test`
- [ ] 代码已格式化（`cargo fmt`, `npm run format`）
- [ ] 没有 clippy 警告
- [ ] 更新了相关文档
- [ ] 添加了必要的测试
- [ ] Commit message 符合规范
- [ ] 已在本地测试运行

## 📚 项目结构

```
xiuxian/
├── src/              # Rust 后端源代码
│   ├── main.rs      # 程序入口
│   ├── web_server.rs # Web API
│   ├── interactive.rs # 游戏逻辑
│   └── ...
├── frontend/         # React 前端
│   ├── src/
│   │   ├── api/     # API 调用
│   │   └── ...
│   └── ...
└── docs/            # 文档
```

## 🐛 报告 Bug

### Bug 报告应包含

1. **环境信息**
   - OS 版本
   - Rust 版本
   - Node.js 版本

2. **重现步骤**
   - 详细的操作步骤
   - 预期结果 vs 实际结果

3. **相关日志**
   - 错误信息
   - 控制台输出

### 示例

```markdown
## Bug 描述
Web 模式下，创建游戏后无法分配任务

## 环境
- macOS 14.0
- Rust 1.71.0
- Node.js 18.0.0

## 重现步骤
1. 启动 Web 服务器：`cargo run -- --web`
2. 打开前端：`npm start`
3. 创建新游戏
4. 点击"分配任务"按钮
5. 报错：...

## 预期行为
应该成功分配任务

## 实际行为
前端显示错误，后端日志：...
```

## 💡 功能建议

欢迎提出新功能建议！请：

1. 先检查 [Issues](链接) 是否已有类似建议
2. 创建新 Issue 描述功能
3. 说明使用场景和价值
4. 讨论实现方案

## 🤝 Pull Request 流程

1. Fork 项目
2. 创建功能分支
3. 编写代码和测试
4. 提交 PR
5. 等待 Review
6. 根据反馈修改
7. 合并！

## ❓ 获取帮助

- 📖 阅读 [文档](README.md)
- 💬 在 Issues 提问
- 📧 联系维护者

## 📜 许可证

提交代码即表示同意项目许可证。

---

感谢你的贡献！🎉
