# Sift

> RAW + JPG Photo Culling Tool — One Click Away

Sift 是一个为 RAW + JPG 双格式拍摄的摄影师设计的跨平台筛图工具。快速浏览、标记精选、一键归档，让你专注于挑选最好的照片。

![Sift 应用截图](https://s41.ax1x.com/2026/04/09/peduoBd.png)

## 功能

### 核心工作流

- 🔍 **自动扫描配对** — 自动匹配同名的 RAW 和 JPG 文件
- 🖼️ **JPG 大图预览** — 只浏览 JPG 做决定，高性能渲染，支持缩放和平移
- ⭐ **标记精选** — 快捷键 F，标记你最喜欢的照片
- 🗑️ **联动删除** — 快捷键 X，删除 JPG 自动带走对应 RAW
- ⏭️ **跳过** — 空格键，跳过不做操作
- 🔄 **撤销** — Ctrl/Cmd+Z 撤销上一步操作

### 图片对比

- 🔲 **左右分屏对比** — 按 C 进入对比模式，将当前图锁定为基准图，左右分屏对比浏览
- 🔄 **交换基准图** — 点击分隔线中间的按钮，将当前浏览图设为新的基准图
- 🔗 **同步缩放平移** — 对比模式下两张图共享缩放和平移状态
- 🏷️ **面板标识** — 顶部栏显示「基准图」/「当前浏览」标签和文件名，缩略图条高亮基准图

### 归档与导出

- 📦 **一键归档** — 将照片按 RAW/JPG 分类到子文件夹
- 📤 **精选导出** — 只导出标记的照片到指定目录

### 浏览与导航

- 🎞️ **缩略图胶片条** — 底部缩略图导航栏，快速定位任意照片，带状态色条和主色占位
- 🏷️ **筛选图库** — 按状态（已标记/已删除/已跳过/未处理）分类浏览照片
- 📊 **EXIF 拍摄信息** — 查看相机型号、镜头、光圈/快门/ISO、焦距、拍摄时间、图片尺寸、文件大小

### 数据统计

- 📈 **实时进度条** — 底部状态栏显示当前进度，带 shimmer 动画
- 🍩 **完成总结** — 审阅完毕弹出 donut chart 统计卡片，展示标记率和分类统计
- 🔢 **滚动数字动画** — 状态计数变化时带动画过渡

## 技术栈

- **Tauri 2** — Rust 后端 + Web 前端，跨平台桌面应用
- **Vue 3** — Composition API + TypeScript
- **Pinia** — 状态管理
- **Tailwind CSS** — 暗色摄影工作站风格
- **Lucide Icons** — 一致的图标系统

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
git clone https://github.com/sankigan/Sift.git
cd Sift

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
| X / Delete / Backspace | 删除 🗑️ |
| Space | 跳过 ⏭️ |
| → | 跳过 ⏭️ |
| ← / A | 上一张 |
| Ctrl/Cmd+Z | 撤销 |
| I | 显示/隐藏 EXIF |
| C | 进入/退出对比模式 |
| Ctrl/Cmd+Enter | 打开归档对话框 |
| Esc | 退出对比模式 / 关闭筛选图库 |
| 滚轮 | 缩放 |
| 双击 | 切换 100%/适应 |
| 0 | 适应窗口 |
| 1 | 100% |
| +/- | 放大/缩小 |

## 项目结构

```
src/
├── components/
│   ├── actions/       # ActionBar 操作栏, Toast 通知
│   ├── archive/       # 归档对话框
│   ├── common/        # 通用组件（骨架图等）
│   ├── exif/          # EXIF 拍摄信息面板
│   ├── status/        # 状态栏, 筛选图库, 滚动数字
│   ├── summary/       # 完成总结卡片
│   ├── viewer/        # 图片查看器, 导航栏, 缩略图条
│   └── welcome/       # 文件夹选择器
├── composables/       # Vue 组合式函数
├── services/          # Tauri 命令调用
├── stores/            # Pinia 状态管理
├── styles/            # 全局样式
└── types/             # TypeScript 类型定义

src-tauri/
└── src/
    ├── commands/      # Rust 后端命令（扫描/缩略图/归档/EXIF等）
    ├── models/        # 数据模型（PhotoPair, ExifData等）
    └── utils/         # 工具函数（EXIF读取等）
```

## License

MIT
