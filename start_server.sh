#!/bin/bash

echo "🚀 启动修仙宗门模拟器 Web 服务器..."
echo ""

cd "$(dirname "$0")"

cargo run --release -- --web
