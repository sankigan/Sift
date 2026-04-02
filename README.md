# Sift

> RAW+JPG Photo Culling Tool — One Click Away

Sift 是一个为 RAW+JPG 双格式拍摄的摄影师设计的跨平台筛图工具。

## 功能

- 🔍 **自动扫描配对** — 自动匹配同名的 RAW 和 JPG 文件
- 🖼️ **JPG 大图预览** — 只浏览 JPG 做决定，高性能 Canvas 渲染
- ⭐ **标记精选** — 快捷键 F，标记你最喜欢的照片
- 🗑️ **联动删除** — 快捷键 X，删除 JPG 自动带走对应 RAW（走系统回收站）
- ⏭️ **跳过** — 右方向键/空格，跳过不做操作
- 📦 **一键归档** — 将幸存照片按 RAW/JPG 分类到子文件夹
- 📤 **精选导出** — 只导出标记的照片到指定目录
- 🔄 **撤销** — Ctrl/Cmd+Z 撤销上一步操作
- 📊 **EXIF 信息** — 查看拍摄参数（ISO/光圈/快门/焦距）

## 技术栈

- **Tauri 2** — Rust 后端 + Web 前端，跨平台桌面应用
- **Vue 3** — Composition API + TypeScript
- **Pinia** — 状态管理
- **Tailwind CSS** — 暗色摄影工作站风格
- **@vueuse/motion** — Spring 物理动画

## 支持的 RAW 格式

Canon (CR2/CR3) · Nikon (NEF/NRW) · Sony (ARW/SR2/SRF) · Fujifilm (RAF) · Olympus (ORF) · Panasonic (RW2) · Pentax (PEF) · Leica (RWL) · Hasselblad (3FR) · Phase One (IIQ) · Samsung (SRW) · Sigma (X3F) · Adobe (DNG)

## 快速开始

### 前置要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (最新 stable)
- macOS: Xcode Command Line Tools (`xcode-select --install`)
- Windows: [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### 安装

```bash
# 安装 Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
cd ~/Desktop/sift

# 安装前端依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 构建

```bash
# 构建生产版本
npm run tauri build
```

## 快捷键

| 快捷键 | 操作 |
|--------|------|
| F | 标记精选 ⭐ |
| X / Delete | 删除 🗑️ |
| → / Space | 跳过 ⏭️ |
| ← / A | 上一张 |
| Ctrl/Cmd+Z | 撤销 |
| I | 显示/隐藏 EXIF |
| 滚轮 | 缩放 |
| 双击 | 切换 100%/适应 |
| 0 | 适应窗口 |
| 1 | 100% |
| +/- | 放大/缩小 |

## License

MIT
