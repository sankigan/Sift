#!/bin/bash
# Sift - 移除 macOS 隔离属性
# 双击此文件即可修复 "Sift 已损坏" 或无法打开的问题
# 适用于未签名的开发版本

APP_PATH="/Applications/Sift.app"

if [ ! -d "$APP_PATH" ]; then
  echo "❌ 未找到 $APP_PATH"
  echo "请确认 Sift.app 已安装到 /Applications 目录"
  echo ""
  read -n 1 -s -r -p "按任意键退出..."
  exit 1
fi

echo "🔧 正在移除 Sift.app 的隔离属性..."
xattr -cr "$APP_PATH"

if [ $? -eq 0 ]; then
  echo "✅ 完成！现在可以正常打开 Sift 了。"
else
  echo "❌ 操作失败，请尝试手动执行："
  echo "   sudo xattr -cr /Applications/Sift.app"
fi

echo ""
read -n 1 -s -r -p "按任意键退出..."
