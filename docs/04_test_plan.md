# 测试计划

## Phase 1: Crypto 本地测试
- Dart 签名 + Rust 验签
- 篡改检测、重放防护、seq 检查

## Phase 2: 配对流程测试
- 配对码生成、使用、过期
- 公钥注册

## Phase 3: Echo 闭环
- Relay + Bridge + Phone Simulator

## Phase 4: Event Log
- SQLite 持久化、seq 递增、重启保持

## Phase 5: tmux shell
- 创建、输入、输出采集、停止

## Phase 6: 全路径
- 签名 → Relay → Bridge → tmux → 输出回传
