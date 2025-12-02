#!/bin/bash

# 脚本名称: cargo_clean.sh
# 功能: 在当前目录下执行 cargo clean 命令，清理 Rust 项目的编译产物

echo "正在清理 Rust 项目构建缓存..."

if command -v cargo >/dev/null 2>&1; then
    cargo clean
    echo "清理完成。"
else
    echo "错误: 未找到 cargo 命令，请确保已安装 Rust 工具链。"
    exit 1
fi