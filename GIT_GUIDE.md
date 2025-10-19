# Git 使用指南

## 📚 Git 仓库信息

本项目已初始化为 Git 仓库，包含完整的版本控制。

## 🚀 快速开始

### 查看状态

```bash
# 查看当前状态
git status

# 查看提交历史
git log

# 查看简洁历史
git log --oneline
```

### 基本操作

```bash
# 查看修改
git diff

# 添加文件
git add <file>
git add .  # 添加所有修改

# 提交
git commit -m "描述信息"

# 查看分支
git branch
```

## 🌿 分支管理

### 创建和切换分支

```bash
# 创建新分支
git branch feature/new-feature

# 切换分支
git checkout feature/new-feature

# 创建并切换（推荐）
git checkout -b feature/new-feature

# 查看所有分支
git branch -a
```

### 合并分支

```bash
# 切换到目标分支
git checkout main

# 合并其他分支
git merge feature/new-feature

# 删除已合并的分支
git branch -d feature/new-feature
```

## 🔄 远程仓库

### 添加远程仓库

```bash
# 添加 GitHub 远程仓库
git remote add origin https://github.com/username/xiuxian.git

# 或使用 SSH
git remote add origin git@github.com:username/xiuxian.git

# 查看远程仓库
git remote -v
```

### 推送和拉取

```bash
# 首次推送
git push -u origin main

# 后续推送
git push

# 拉取最新代码
git pull

# 拉取并变基
git pull --rebase
```

## 📝 提交规范

### 推荐的提交格式

```bash
# 功能
git commit -m "feat: 添加渡劫系统"

# 修复
git commit -m "fix: 修复弟子寿元计算错误"

# 文档
git commit -m "docs: 更新 API 文档"

# 重构
git commit -m "refactor: 优化任务分配算法"
```

### 多行提交信息

```bash
git commit -m "feat: 添加渡劫系统

- 实现渡劫判定逻辑
- 添加渡劫成功/失败处理
- 更新弟子状态管理

Closes #42"
```

## 🏷️ 标签管理

### 创建版本标签

```bash
# 创建轻量标签
git tag v1.0.0

# 创建附注标签（推荐）
git tag -a v1.0.0 -m "版本 1.0.0 - 初始发布"

# 查看所有标签
git tag

# 查看标签详情
git show v1.0.0

# 推送标签到远程
git push origin v1.0.0

# 推送所有标签
git push origin --tags
```

## 🔍 查看历史

### 常用命令

```bash
# 查看完整历史
git log

# 简洁历史
git log --oneline

# 图形化历史
git log --graph --oneline --all

# 查看某个文件的历史
git log -- src/main.rs

# 查看某次提交的详情
git show <commit-hash>
```

## ⏪ 撤销操作

### 撤销修改

```bash
# 撤销工作区的修改
git checkout -- <file>
git restore <file>

# 撤销暂存区的修改
git reset HEAD <file>
git restore --staged <file>

# 撤销最后一次提交（保留修改）
git reset --soft HEAD^

# 撤销最后一次提交（不保留修改）
git reset --hard HEAD^
```

### 修改提交

```bash
# 修改最后一次提交信息
git commit --amend -m "新的提交信息"

# 将新修改添加到最后一次提交
git add .
git commit --amend --no-edit
```

## 🗑️ 清理操作

### 删除文件

```bash
# 删除文件并暂存
git rm <file>

# 仅从 Git 删除，保留本地文件
git rm --cached <file>

# 删除文件夹
git rm -r <directory>
```

### 清理未跟踪文件

```bash
# 查看会删除什么
git clean -n

# 删除未跟踪的文件
git clean -f

# 删除未跟踪的文件和目录
git clean -fd
```

## 🔧 实用技巧

### 储藏（Stash）

```bash
# 储藏当前修改
git stash

# 储藏并添加描述
git stash save "描述"

# 查看储藏列表
git stash list

# 应用最新储藏
git stash pop

# 应用指定储藏
git stash apply stash@{0}

# 删除储藏
git stash drop stash@{0}
```

### 查看差异

```bash
# 查看工作区和暂存区差异
git diff

# 查看暂存区和最后提交的差异
git diff --staged

# 查看两个分支的差异
git diff main..feature/new
```

### Cherry-pick

```bash
# 将其他分支的提交应用到当前分支
git cherry-pick <commit-hash>
```

## 📋 .gitignore

已配置的忽略文件包括：

```
# Rust
/target/
Cargo.lock

# Frontend
/frontend/node_modules/
/frontend/build/

# IDE
.idea/
.vscode/

# OS
.DS_Store
```

### 添加新的忽略规则

编辑 `.gitignore` 文件：

```bash
echo "*.log" >> .gitignore
git add .gitignore
git commit -m "chore: 忽略日志文件"
```

## 🌐 GitHub 工作流

### Fork 和 Pull Request

```bash
# 1. Fork 项目到自己账号

# 2. Clone 自己的 Fork
git clone https://github.com/your-username/xiuxian.git

# 3. 添加上游仓库
git remote add upstream https://github.com/original/xiuxian.git

# 4. 创建功能分支
git checkout -b feature/new-feature

# 5. 开发并提交
git add .
git commit -m "feat: 新功能"

# 6. 推送到自己的仓库
git push origin feature/new-feature

# 7. 在 GitHub 上创建 Pull Request

# 8. 同步上游更新
git fetch upstream
git merge upstream/main
```

## 📊 项目统计

### 查看统计信息

```bash
# 查看代码行数
git ls-files | xargs wc -l

# 查看贡献者统计
git shortlog -sn

# 查看某个作者的提交
git log --author="name"

# 查看最近一周的提交
git log --since="1 week ago"
```

## ⚠️ 注意事项

### 不要提交的内容

- ❌ `target/` - Rust 编译输出
- ❌ `node_modules/` - Node.js 依赖
- ❌ `.env` - 环境变量（包含敏感信息）
- ❌ `*.log` - 日志文件
- ❌ IDE 配置文件

### 最佳实践

- ✅ 提交前运行测试
- ✅ 编写清晰的提交信息
- ✅ 经常提交，小步快跑
- ✅ 推送前先拉取最新代码
- ✅ 使用分支进行开发
- ✅ 代码审查后再合并

## 🆘 常见问题

### 合并冲突

```bash
# 1. 拉取最新代码时出现冲突
git pull

# 2. 手动解决冲突文件中的标记
# <<<<<<< HEAD
# =======
# >>>>>>> branch

# 3. 标记为已解决
git add <resolved-files>

# 4. 完成合并
git commit
```

### 撤销推送的提交

```bash
# ⚠️ 慎用！会改变历史
git reset --hard HEAD^
git push -f

# 更安全的方式：创建反向提交
git revert <commit-hash>
git push
```

## 📚 学习资源

- [Pro Git 中文版](https://git-scm.com/book/zh/v2)
- [Git 简明指南](http://rogerdudler.github.io/git-guide/index.zh.html)
- [Learn Git Branching](https://learngitbranching.js.org/)

---

**Happy Coding!** 🎉
