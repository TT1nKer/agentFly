# 部署指南

## Relay 部署

1. 编译：`cargo build --release -p agent-relay`
2. 部署到 VPS
3. 配置 Caddy 反向代理 HTTPS
4. 只开放 443/tcp

## Bridge 部署

1. 编译：`cargo build --release -p agent-bridge`
2. 安装 tmux
3. 运行 `agent-bridge run`

## App 构建

1. `cd app && flutter build apk`
2. `flutter build ios`
3. 分发到手机
