#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Sift Release Script
# 用法: ./scripts/release.sh <version>
#   例: ./scripts/release.sh 1.1.0
#   或: npm run release -- 1.1.0
# ============================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info()  { echo -e "${CYAN}[info]${NC} $*"; }
ok()    { echo -e "${GREEN}[ok]${NC} $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $*"; }
error() { echo -e "${RED}[error]${NC} $*" >&2; exit 1; }

# ---- 1. 参数校验 ----
VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
  echo ""
  echo -e "  ${CYAN}Sift Release Script${NC}"
  echo ""
  echo "  用法: $0 <version>"
  echo "  例如: $0 1.2.0"
  echo ""
  exit 1
fi

# 去掉可能的 v 前缀
VERSION="${VERSION#v}"

# semver 格式校验
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
  error "版本号格式不合法: '$VERSION' (需要 semver 格式，如 1.2.0 或 1.0.0-beta.1)"
fi

TAG="v${VERSION}"

# ---- 2. 项目根目录 ----
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

info "项目根目录: $ROOT_DIR"

# ---- 3. 检查 tag 是否已存在 ----
if git rev-parse "$TAG" >/dev/null 2>&1; then
  error "Tag '$TAG' 已存在！请使用一个新版本号。"
fi

# ---- 4. 检查工作区（允许有暂存的改动，比如 CHANGELOG） ----
if [[ -n "$(git diff HEAD --name-only 2>/dev/null)" ]] || [[ -n "$(git diff --cached --name-only 2>/dev/null)" ]]; then
  warn "检测到未提交的改动，这些改动会被包含在发布 commit 中："
  echo ""
  git status --short
  echo ""
  read -rp "$(echo -e "${YELLOW}继续？(y/n): ${NC}")" CONTINUE
  if [[ "$CONTINUE" != "y" && "$CONTINUE" != "Y" ]]; then
    info "已取消。"
    exit 0
  fi
fi

# ---- 5. 更新版本号 ----
info "更新版本号为 ${VERSION} ..."

# package.json
if command -v node >/dev/null 2>&1; then
  node -e "
    const fs = require('fs');
    const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
    pkg.version = '${VERSION}';
    fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
  "
else
  # fallback: sed
  sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${VERSION}\"/" package.json
fi
ok "package.json -> ${VERSION}"

# tauri.conf.json
if command -v node >/dev/null 2>&1; then
  node -e "
    const fs = require('fs');
    const conf = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
    conf.version = '${VERSION}';
    fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(conf, null, 2) + '\n');
  "
else
  sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${VERSION}\"/" src-tauri/tauri.conf.json
fi
ok "src-tauri/tauri.conf.json -> ${VERSION}"

# Cargo.toml (只改 [package] 下的第一个 version)
sed -i '' '0,/^version = ".*"/s/^version = ".*"/version = "'"${VERSION}"'"/' src-tauri/Cargo.toml
ok "src-tauri/Cargo.toml -> ${VERSION}"

# ---- 6. 同步 Cargo.lock ----
info "同步 Cargo.lock ..."
(cd src-tauri && cargo check --quiet 2>/dev/null) || warn "cargo check 失败，Cargo.lock 可能未更新"
ok "Cargo.lock 已同步"

# ---- 7. 显示变更摘要 ----
echo ""
echo -e "${CYAN}════════════════════════════════════════${NC}"
echo -e "${CYAN}  发布摘要${NC}"
echo -e "${CYAN}════════════════════════════════════════${NC}"
echo ""
echo -e "  版本:  ${GREEN}${VERSION}${NC}"
echo -e "  Tag:   ${GREEN}${TAG}${NC}"
echo -e "  分支:  $(git branch --show-current)"
echo ""
echo -e "  ${YELLOW}变更文件:${NC}"
git add -A
git diff --cached --stat
echo ""
echo -e "${CYAN}════════════════════════════════════════${NC}"
echo ""

# ---- 8. 确认推送 ----
read -rp "$(echo -e "${YELLOW}确认发布 ${TAG}？(y/n): ${NC}")" CONFIRM
if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
  warn "已取消。改动已暂存但未提交，你可以手动处理。"
  exit 0
fi

# ---- 9. Commit + Tag + Push ----
BRANCH="$(git branch --show-current)"

git commit -m "chore(release): ${TAG}"
ok "已提交: chore(release): ${TAG}"

git tag "$TAG"
ok "已创建 tag: ${TAG}"

git push origin "$BRANCH" --tags
ok "已推送到 origin/${BRANCH}"

echo ""
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}  🎉 ${TAG} 发布完成！${NC}"
echo -e "${GREEN}  CI 将自动开始构建...${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo ""
