# Agent Cockpit 实习生开发手册 v2

## 0. 一句话目标

做一个 Android + iPhone App，让用户可以从手机远程控制自己电脑上的 agent session，例如 opencode、hermes、BoOS、shell。

系统结构固定为：

```text
手机 App
  ↓ WSS 443
自建 VPS Relay
  ↓ WSS 443
电脑 Agent Bridge
  ↓
tmux session
  ↓
opencode / hermes / shell
```

本项目不是远程桌面，不是网页 terminal，不是手机跑 agent。

## 1. 最重要的安全原则

本项目采用个人设备信任模型。

意思是：

```text
1. 手机和电脑都是用户自己的。
2. 已配对手机被视为可信控制端。
3. 不能让陌生手机加入。
4. 不能让 Relay 伪造手机命令。
5. 不能只靠 token 授权。
```

最终安全机制：

```text
手机生成私钥和公钥。
电脑保存手机公钥。
手机发往电脑的每条控制消息都必须用私钥签名。
电脑 Bridge 收到消息后必须验签。
验签失败，一律不执行。
```

Relay 只负责转发，不可信。

Relay 拿不到手机私钥，所以 Relay 不能伪造手机命令。

## 2. 强制技术栈

### 2.1 手机端

固定使用：

```text
Flutter
Dart
Android + iOS 共用一套代码
Ed25519 签名
```

禁止使用：

```text
React Native
网页壳
小程序
双端原生同时开发
```

### 2.2 Relay 端

固定使用：

```text
Rust
Tokio
Axum
WebSocket
SQLite
Caddy HTTPS 反向代理
```

Relay 只开放公网：

```text
443/tcp
```

禁止开放调试端口。

### 2.3 Bridge 端

固定使用：

```text
Rust
Tokio
Axum client / WebSocket client
SQLite
tmux
```

Bridge 运行在：

```text
macOS
Linux
Windows WSL2 Ubuntu
```

Windows 原生暂不支持。

## 3. 组件职责

## 3.1 手机 App 职责

手机 App 负责：

```text
1. 生成手机 Ed25519 密钥对
2. 保存手机私钥
3. 通过配对码把手机公钥交给电脑
4. 展示 session 列表
5. 发送 session 输入
6. 查看 session 输出
7. 上传文本、URL、小截图、小文件
8. 给每条控制消息签名
```

手机 App 禁止：

```text
1. 直接执行 shell
2. 保存电脑 SSH key
3. 直接连接电脑端口
4. 把私钥发给 Relay
5. 把私钥发给 Bridge
```

## 3.2 Relay 职责

Relay 负责：

```text
1. 接收手机 WebSocket 连接
2. 接收 Bridge WebSocket 连接
3. 记录设备在线状态
4. 转发消息
5. 暂存离线消息
6. 做简单限流
```

Relay 禁止：

```text
1. 执行 shell
2. 运行 opencode/hermes
3. 判断 agent 权限
4. 伪造手机消息
5. 修改消息 payload
6. 保存手机私钥
```

Relay 可以有 token，但 token 只用于连接层识别和限流。

token 不能作为电脑执行命令的授权依据。

## 3.3 Bridge 职责

Bridge 负责：

```text
1. 主动连接 Relay
2. 保存已配对手机公钥
3. 验证手机控制消息签名
4. 验证 nonce、timestamp、seq，防止重放攻击
5. 创建 tmux session
6. 向 tmux session 发送输入
7. 采集 tmux 输出
8. 写入 event log
9. 把输出返回手机
```

Bridge 是唯一允许接触 tmux / opencode / hermes 的组件。

## 4. 仓库结构

必须按下面结构建仓库：

```text
agent-cockpit/
  README.md

  docs/
    00_goal.md
    01_architecture.md
    02_security_model.md
    03_protocol.md
    04_test_plan.md
    05_deployment.md

  protocol/
    message.schema.json
    signed_message.schema.json
    event.schema.json

  relay/
    Cargo.toml
    src/
      main.rs
      config.rs
      db.rs
      ws.rs
      routes.rs
      auth.rs
      router.rs
      device_registry.rs
      message_store.rs

  bridge/
    Cargo.toml
    src/
      main.rs
      config.rs
      db.rs
      relay_client.rs
      crypto.rs
      verify.rs
      pairing.rs
      devices.rs
      session/
        mod.rs
        manager.rs
        model.rs
      adapters/
        mod.rs
        echo.rs
        tmux.rs
        shell.rs
        opencode.rs
        hermes.rs
      event_log/
        mod.rs
        store.rs
        model.rs

  app/
    pubspec.yaml
    lib/
      main.dart
      app.dart
      crypto/
        device_key.dart
        signer.dart
      api/
        relay_client.dart
        ws_client.dart
        models.dart
      screens/
        pair_screen.dart
        connection_screen.dart
        session_list_screen.dart
        session_detail_screen.dart
        settings_screen.dart
      widgets/
        event_view.dart
        output_view.dart
        status_badge.dart
      storage/
        secure_key_store.dart
        local_cache.dart

  scripts/
    phase_01_crypto_test.sh
    phase_02_pairing_test.sh
    phase_03_echo_loop_test.sh
    phase_04_event_log_test.sh
    phase_05_tmux_shell_test.sh
    phase_06_full_path_test.sh
```

禁止把所有代码写进一个文件。

## 5. 设备配对流程

配对必须由电脑端主动开启。

### 5.1 电脑端开启配对

命令：

```bash
agent-bridge pair
```

输出：

```text
Pairing code: 482913
Expires in 10 minutes.
Waiting for phone...
```

要求：

```text
1. 配对码 6 位数字
2. 10 分钟过期
3. 只能使用一次
4. 连续输错 5 次，暂停配对 10 分钟
5. 默认不开放配对
```

### 5.2 手机端配对

手机 App 做：

```text
1. 本地生成 Ed25519 keypair
2. 保存 private_key
3. 显示输入配对码页面
4. 用户输入配对码
5. 手机把 public_key + device_name + pairing_code 发给 Relay
6. Relay 转发给 Bridge
7. Bridge 检查 pairing_code
8. Bridge 保存 public_key
9. 配对成功
```

注意：

```text
private_key 永远不离开手机。
Bridge 只保存 public_key。
Relay 不保存 private_key。
```

## 6. 手机私钥保存要求

手机私钥必须保存到系统安全存储。

Flutter 层封装一个接口：

```dart
abstract class SecureKeyStore {
  Future<void> savePrivateKey(String privateKeyBase64);
  Future<String?> loadPrivateKey();
  Future<void> deletePrivateKey();
}
```

要求：

```text
1. 私钥不能写进普通 JSON 文件
2. 私钥不能打印到日志
3. 私钥不能上传 Relay
4. 私钥不能随 crash report 上传
5. App 重装后允许重新配对
```

## 7. 控制消息签名协议

手机发往 Bridge 的控制消息必须签名。

电脑发往手机的输出消息 v1 不强制签名，因为手机只展示数据，不执行本地危险操作。

## 7.1 SignedMessage 格式

所有手机 → Bridge 的控制消息统一格式：

```json
{
  "version": 1,
  "message_id": "msg_001",
  "device_id": "phone_abc",
  "type": "session.input",
  "timestamp_ms": 1781000000000,
  "nonce": "base64_random_16_bytes",
  "seq": 1042,
  "payload": {
    "session_id": "sess_001",
    "content": "帮我检查这个项目为什么卡住"
  },
  "payload_sha256": "hex_sha256_of_payload",
  "signature": "base64_ed25519_signature"
}
```

必填字段：

```text
version
message_id
device_id
type
timestamp_ms
nonce
seq
payload
payload_sha256
signature
```

缺一个字段，Bridge 必须拒绝。

## 7.2 签名内容

禁止直接签整个 JSON 字符串。

因为 JSON 字段顺序和空格可能变化。

统一签下面这个 signing string：

```text
v1
message_id=<message_id>
device_id=<device_id>
type=<type>
timestamp_ms=<timestamp_ms>
nonce=<nonce>
seq=<seq>
payload_sha256=<payload_sha256>
```

例子：

```text
v1
message_id=msg_001
device_id=phone_abc
type=session.input
timestamp_ms=1781000000000
nonce=X0Jz2N8cCj9YhWm4xQw==
seq=1042
payload_sha256=9f86d081884c7d659a2feaa0c55ad015...
```

手机对这个 UTF-8 字符串做 Ed25519 签名。

Bridge 用对应 phone public key 验证签名。

## 7.3 payload_sha256 计算规则

payload 必须先转 canonical JSON。

v1 简化规则：

```text
1. payload 只能是一层 JSON object
2. key 按字母顺序排序
3. 不允许 undefined
4. 不允许 NaN
5. 不允许函数
6. 字符串使用 UTF-8
```

如果实习生不会做 canonical JSON，v1 可以临时用固定字段拼接。

例如 session.input 的 payload_hash 内容：

```text
session_id=<session_id>
content_sha256=<sha256(content)>
```

不允许各端自己随便 JSON.stringify 后直接 hash。

## 8. Bridge 验签流程

Bridge 收到手机消息后，必须按这个顺序处理：

```text
1. 检查 JSON 格式
2. 检查必填字段
3. 根据 device_id 查询 trusted_devices
4. 检查设备是否 active
5. 检查 timestamp_ms 是否在当前时间 ±5 分钟内
6. 检查 nonce 是否从未使用过
7. 检查 seq 是否大于该设备 last_seq
8. 重新计算 payload_sha256
9. 构造 signing string
10. 用 public_key 验证 signature
11. 验证通过后，写入 used_nonces
12. 更新 trusted_devices.last_seq
13. 执行业务逻辑
```

只要有一步失败，直接拒绝。

错误 code 固定：

```text
INVALID_JSON
MISSING_FIELD
DEVICE_NOT_TRUSTED
DEVICE_REVOKED
BAD_TIMESTAMP
REPLAY_DETECTED
BAD_SEQUENCE
BAD_PAYLOAD_HASH
BAD_SIGNATURE
INTERNAL_ERROR
```

## 9. 数据库设计

## 9.1 Bridge 数据库

Bridge 使用 SQLite。

必须创建这些表：

```sql
CREATE TABLE trusted_devices (
  device_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  platform TEXT NOT NULL,
  public_key_base64 TEXT NOT NULL,
  key_algorithm TEXT NOT NULL,
  status TEXT NOT NULL,
  last_seq INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  revoked_at TEXT
);

CREATE TABLE used_nonces (
  device_id TEXT NOT NULL,
  nonce TEXT NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  message_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (device_id, nonce)
);

CREATE TABLE sessions (
  session_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  title TEXT NOT NULL,
  workspace TEXT NOT NULL,
  tmux_name TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE events (
  event_id TEXT PRIMARY KEY,
  session_id TEXT,
  seq INTEGER NOT NULL,
  type TEXT NOT NULL,
  content TEXT,
  payload_json TEXT,
  created_at TEXT NOT NULL
);
```

## 9.2 Relay 数据库

Relay 使用 SQLite。

必须创建：

```sql
CREATE TABLE devices (
  device_id TEXT PRIMARY KEY,
  device_type TEXT NOT NULL,
  name TEXT NOT NULL,
  token_hash TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  last_seen_at TEXT
);

CREATE TABLE messages (
  message_id TEXT PRIMARY KEY,
  from_device_id TEXT NOT NULL,
  to_device_id TEXT NOT NULL,
  type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  delivered_at TEXT
);
```

Relay 只保存连接层 token hash。

Relay 不保存手机私钥。

Relay 不负责最终授权。

## 10. Session 类型

v1 支持这些 session kind：

```text
echo
shell
opencode
hermes
```

开发顺序必须是：

```text
echo → shell → opencode → hermes
```

禁止直接从 opencode/hermes 开始。

## 11. tmux 使用规则

所有真实 session 都必须跑在 tmux 里。

Bridge 创建 session 时执行类似逻辑：

```bash
tmux new-session -d -s <tmux_name> -c <workspace> <command>
```

发送输入：

```bash
tmux send-keys -t <tmux_name> "<input>" Enter
```

采集输出 v1 可以先用：

```bash
tmux capture-pane -pt <tmux_name> -S -200
```

但必须写在文档里：

```text
capture-pane 只作为 v1 输出采集方案。
它不是完美日志流。
后续可替换为 pipe-pane 或 control mode。
```

## 12. Event Log 规则

Bridge 必须保存 event log。

不能只把输出转发给手机。

所有重要动作都要写 event：

```text
session.created
session.started
session.stopped
session.failed
user.input
agent.output
agent.error
device.paired
device.revoked
system.notice
```

event 必须有 seq。

同一个 session 的 seq 递增。

手机断线重连后，通过 last_seq 拉取历史事件。

## 13. Phase 开发计划

## Phase 1：Crypto 本地测试

目标：

```text
证明手机签名、Bridge 验签逻辑正确。
```

暂时不接 Relay，不接 tmux，不接 App UI。

要做：

```text
1. 写 Dart 生成 Ed25519 keypair
2. 写 Dart 对 signing string 签名
3. 写 Rust 读取 public_key、message、signature
4. Rust 验签通过
5. 篡改 payload 后验签失败
6. 重放 nonce 被拒绝
7. seq 倒退被拒绝
```

验收脚本：

```bash
./scripts/phase_01_crypto_test.sh
```

必须输出：

```text
valid signature: PASS
tampered payload rejected: PASS
bad signature rejected: PASS
replay nonce rejected: PASS
bad seq rejected: PASS
PHASE 1 PASS
```

失败判定：

```text
1. 如果篡改 payload 后仍然通过，失败
2. 如果旧 nonce 可重复使用，失败
3. 如果 seq 倒退仍然执行，失败
4. 如果 token 可以绕过签名，失败
```

## Phase 2：配对流程测试

目标：

```text
证明只有电脑开启配对后，手机公钥才能加入 trusted_devices。
```

要做：

```text
1. Bridge 实现 agent-bridge pair
2. Bridge 生成 6 位配对码
3. Phone simulator 生成 keypair
4. Phone simulator 发送 public_key + pairing_code
5. Bridge 保存 public_key
6. 配对码使用后失效
7. 配对码过期后失效
```

验收脚本：

```bash
./scripts/phase_02_pairing_test.sh
```

必须输出：

```text
pair code generated: PASS
phone public key registered: PASS
pair code single-use: PASS
expired pair code rejected: PASS
wrong pair code rejected: PASS
PHASE 2 PASS
```

失败判定：

```text
1. 不开配对也能加手机，失败
2. 配对码能重复使用，失败
3. Bridge 没有保存 public_key，失败
4. Relay 保存 private_key，失败
```

## Phase 3：Relay + Bridge + Phone Simulator Echo 闭环

目标：

```text
证明全链路能转发签名消息。
```

链路：

```text
phone simulator
  ↓
relay
  ↓
bridge
  ↓
echo adapter
  ↓
relay
  ↓
phone simulator
```

要做：

```text
1. Relay 实现 /health
2. Relay 实现 WebSocket
3. Bridge 主动连接 Relay
4. Phone simulator 连接 Relay
5. Phone simulator 发送已签名 echo.ping
6. Relay 转发
7. Bridge 验签
8. Echo adapter 返回 echo.pong
```

验收脚本：

```bash
./scripts/phase_03_echo_loop_test.sh
```

必须输出：

```text
relay health: PASS
bridge online: PASS
phone connected: PASS
signed echo.ping sent: PASS
bridge signature verify: PASS
echo.pong received: PASS
PHASE 3 PASS
```

失败判定：

```text
1. Relay 自己验签后直接执行，失败
2. Bridge 不验签，失败
3. 无签名消息能触发 echo，失败
4. Relay 修改 payload 后 Bridge 仍然接受，失败
```

## Phase 4：Bridge 本地 Event Log

目标：

```text
证明 Bridge 能保存完整事件。
```

要做：

```text
1. Bridge 写 SQLite
2. Echo input 写 user.input event
3. Echo output 写 agent.output event
4. Bridge 重启后 events 仍在
5. 根据 last_seq 拉取增量
```

验收脚本：

```bash
./scripts/phase_04_event_log_test.sh
```

必须输出：

```text
event written: PASS
event seq increasing: PASS
restart keeps events: PASS
fetch after seq: PASS
PHASE 4 PASS
```

失败判定：

```text
1. event 只保存在内存，失败
2. 没有 seq，失败
3. 重启后丢事件，失败
```

## Phase 5：tmux shell session

目标：

```text
证明 Bridge 能控制 tmux shell。
```

要做：

```text
1. 创建 workspace
2. 创建 tmux session
3. 发送 echo hello
4. 捕获输出
5. 写入 event log
6. 停止 session
```

验收脚本：

```bash
./scripts/phase_05_tmux_shell_test.sh
```

必须输出：

```text
tmux installed: PASS
session created: PASS
input sent: PASS
output captured: PASS
event log written: PASS
session stopped: PASS
PHASE 5 PASS
```

失败判定：

```text
1. 不用 tmux，失败
2. 输出只在终端显示但 event log 没有，失败
3. session 停止后 tmux 还在，失败
4. workspace 路径不受限制，失败
```

## Phase 6：Relay 控制 tmux shell

目标：

```text
手机模拟器通过 Relay 控制电脑 tmux session。
```

要做：

```text
1. Phone simulator 发送已签名 session.create
2. Relay 转发给 Bridge
3. Bridge 验签
4. Bridge 创建 tmux shell session
5. Phone simulator 发送已签名 session.input
6. Bridge 把 input 写入 tmux
7. Bridge 捕获输出
8. Phone simulator 收到输出事件
```

验收脚本：

```bash
./scripts/phase_06_full_path_test.sh
```

必须输出：

```text
signed session.create: PASS
bridge verify create: PASS
tmux session created: PASS
signed session.input: PASS
tmux output captured: PASS
phone receives event: PASS
PHASE 6 PASS
```

失败判定：

```text
1. 无签名 session.create 能创建 session，失败
2. 假 device_id 能创建 session，失败
3. Relay 能绕过 Bridge 直接创建 session，失败
4. 断线后 session 死掉，失败
```

## Phase 7：Flutter App 只读版

目标：

```text
App 能看到 session 列表和输出。
```

要做页面：

```text
1. PairScreen
2. ConnectionScreen
3. SessionListScreen
4. SessionDetailScreen
5. EventView
```

只读版不允许发送输入。

验收标准：

```text
1. Android 能连接 Relay
2. iPhone 能连接 Relay
3. App 能显示 Bridge online/offline
4. App 能显示 session list
5. App 能显示 event log
6. 长输出 1000 行不卡死
```

失败判定：

```text
1. 页面写死假数据，失败
2. 只能连接 mock，失败
3. 断线没有提示，失败
4. 重连不能拉历史事件，失败
```

## Phase 8：Flutter App 输入版

目标：

```text
App 能发送已签名 session.input。
```

要做：

```text
1. SessionDetailScreen 增加输入框
2. 点击发送时生成 message_id
3. 生成 timestamp_ms
4. 生成 nonce
5. seq 自增
6. 计算 payload_sha256
7. 构造 signing string
8. 用手机私钥签名
9. 发给 Relay
10. 等 Bridge 输出
```

验收标准：

```text
1. 手机输入 hello
2. Relay 收到 signed message
3. Bridge 验签通过
4. tmux 收到 hello
5. App 显示输出
```

失败判定：

```text
1. App 发送未签名消息，失败
2. seq 不递增，失败
3. 网络失败后草稿丢失，失败
4. 连点发送导致 message_id 重复，失败
```

## Phase 9：opencode / hermes 黑盒接入

目标：

```text
在 tmux 中启动真实 agent。
```

要做：

```text
1. kind=opencode 启动 opencode
2. kind=hermes 启动 hermes
3. 手机输入转发给 tmux
4. 输出采集到 event log
5. App 能看到输出
```

注意：

```text
v1 把 opencode/hermes 当黑盒 TUI。
不要试图解析它的内部状态。
不要承诺能拦截 agent 内部每个操作。
```

验收标准：

```text
1. 能创建 opencode session
2. 能创建 hermes session
3. 手机能发送 prompt
4. agent 有响应
5. 输出返回 App
```

失败判定：

```text
1. agent 崩溃没有 session.failed event，失败
2. 输出完全无法采集，失败
3. 必须人工 attach 才能工作，失败
```

## Phase 10：设备管理

目标：

```text
电脑可以查看和撤销已配对手机。
```

Bridge CLI 必须实现：

```bash
agent-bridge devices list
agent-bridge devices revoke <device_id>
```

list 输出：

```text
Paired devices:
- iPhone 13      phone_abc      active    last_seen=...
- Android Phone  phone_def      active    last_seen=...
```

revoke 后：

```text
1. trusted_devices.status = revoked
2. 后续该 device_id 的消息全部拒绝
3. App 显示设备已撤销，需要重新配对
```

验收标准：

```text
1. list 能看到手机
2. revoke 后手机不能发命令
3. revoke 后旧签名消息不能执行
4. 重新配对后生成新 device_id
```

失败判定：

```text
1. revoke 只在 Relay 生效，Bridge 仍然接受，失败
2. revoked 手机还能控制 session，失败
3. 复用旧 public_key 自动恢复，失败
```

## Phase 11：小型数据入口

目标：

```text
手机可以把文本、URL、小图片、小文件发给电脑 workspace。
```

v1 限制：

```text
1. 文本最大 100KB
2. URL 最大 4KB
3. 图片最大 5MB
4. 文件最大 10MB
```

超过限制直接拒绝。

上传流程：

```text
1. App 生成 file.upload 请求
2. 请求必须签名
3. Relay 暂存文件
4. Bridge 拉取文件
5. Bridge 保存到 workspace/inbox/
6. Bridge 写 file.uploaded event
7. App 显示保存路径
```

验收标准：

```text
1. 分享 URL 到 App
2. 上传截图到 App
3. Bridge 保存到 workspace/inbox/
4. event log 有 file.uploaded
5. 文件路径不能逃出 workspace
```

失败判定：

```text
1. 大文件卡死 App，失败
2. 文件能写到 workspace 外，失败
3. Relay 永久保存用户文件，失败
```

## 14. 日志规则

所有日志必须包含：

```text
time
component
level
message_id
device_id
session_id
event_id
message
```

禁止打印：

```text
1. 手机私钥
2. 完整 token
3. API key
4. 用户文件全文
5. 浏览器 cookie
```

错误日志不能只写：

```text
error happened
```

必须写：

```text
BAD_SIGNATURE: device_id=phone_abc message_id=msg_001
```

## 15. 每次提交前必须跑

```bash
cargo test
flutter test
./scripts/phase_01_crypto_test.sh
./scripts/phase_02_pairing_test.sh
./scripts/phase_03_echo_loop_test.sh
./scripts/phase_04_event_log_test.sh
./scripts/phase_05_tmux_shell_test.sh
./scripts/phase_06_full_path_test.sh
```

任何一个失败都不能合并。

## 16. 禁止事项

实习生禁止：

```text
1. 用 token 代替签名
2. 让 Relay 执行 shell
3. 让 Relay 保存手机私钥
4. 跳过 crypto test
5. 跳过 pairing test
6. 跳过 echo adapter，直接接 opencode
7. 未验签就执行 session.create
8. 未验签就执行 session.input
9. 把私钥打印到日志
10. 自己发明签名算法
11. 自己随便设计 signing string
12. 直接签 JSON.stringify 的结果
13. 把电脑端口暴露到公网
14. 把 session 状态只存在内存
15. 没有 event log
```

## 17. 最终 v1 成功标准

v1 完成必须满足：

```text
1. Android App 可以配对
2. iPhone App 可以配对
3. 手机私钥不离开手机
4. Bridge 保存手机公钥
5. 手机发出的控制消息都有签名
6. Bridge 未验签不执行
7. 篡改 payload 会被拒绝
8. 重放旧消息会被拒绝
9. revoked 手机不能继续控制
10. Relay 无法伪造手机命令
11. 手机能创建 shell/opencode/hermes session
12. 手机能发送输入
13. 手机能看到输出
14. session 断线不死
15. event log 可回放
16. 全部 smoke test 通过
```

少一条都不算 v1 完成。

## 18. 核心原则

所有人记住这几句话：

```text
手机是控制端。
电脑是执行端。
Relay 是转发层，不可信。
Bridge 是最终验签者。
token 不是授权。
手机私钥签名才是授权。
tmux 是 session 容器。
SQLite event log 是事实来源。
每一步必须能脚本验证。
```
