# Contributing to Sift

感谢你有兴趣为 Sift 做贡献！以下指南会帮助你快速上手。

## 开发环境

### 前置要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install)（最新 stable）
- macOS: Xcode Command Line Tools（`xcode-select --install`）
- Windows: [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### 本地启动

```bash
# 克隆你的 fork
git clone https://github.com/<your-username>/Sift.git
cd Sift

# 安装前端依赖
npm install

# 启动开发模式（前端 + Rust 同时编译）
npm run tauri dev
```

### 构建生产版本

```bash
npm run tauri build
```

## 项目结构

```
src/              # Vue 3 前端
├── components/   # UI 组件
├── composables/  # Vue 组合式函数
├── services/     # Tauri 命令调用
├── stores/       # Pinia 状态管理
└── types/        # TypeScript 类型定义

src-tauri/        # Rust 后端
└── src/
    ├── commands/ # Tauri 命令
    ├── models/   # 数据模型
    └── utils/    # 工具函数
```

## 代码规范

- **前端**：TypeScript + Vue 3 Composition API，遵循项目 ESLint/Prettier 配置
- **后端**：Rust，遵循 `cargo fmt` + `cargo clippy` 标准
- **样式**：Tailwind CSS，暗色主题
- **命名**：变量/函数用 camelCase，组件/类用 PascalCase，常量用 UPPER_SNAKE_CASE

## 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <description>

feat(viewer): add pinch-to-zoom support
fix(scan): handle symlinked directories
docs(readme): update installation guide
```

常用 type：`feat` / `fix` / `docs` / `style` / `refactor` / `perf` / `test` / `chore`

## Pull Request 流程

1. Fork 仓库并创建你的分支（`git checkout -b feat/my-feature`）
2. 在你的分支上进行开发
3. 确保代码能正常编译：
   - 前端：`npm run build`
   - 后端：`cd src-tauri && cargo build`
4. 提交 PR 到 `main` 分支
5. 在 PR 描述中说明改动内容和动机

## Bug 报告

请使用 [Bug Report](https://github.com/sankigan/Sift/issues/new?template=bug_report.md) 模板提交 Issue，包含以下信息：

- 操作系统及版本
- Sift 版本
- 复现步骤
- 预期行为 vs 实际行为
- 如有可能，附上截图或日志

## Feature Request

欢迎通过 [Feature Request](https://github.com/sankigan/Sift/issues/new?template=feature_request.md) 模板提出新功能建议。

## License

提交贡献即表示你同意你的代码以 [MIT License](LICENSE) 授权。
