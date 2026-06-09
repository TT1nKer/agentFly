# 通信协议 v1

## SignedMessage 格式

```json
{
  "version": 1,
  "message_id": "msg_001",
  "device_id": "phone_abc",
  "type": "session.input",
  "timestamp_ms": 1781000000000,
  "nonce": "base64_random_16_bytes",
  "seq": 1042,
  "payload": { "session_id": "sess_001", "content": "..." },
  "payload_sha256": "hex...",
  "signature": "base64..."
}
```

## 签名构造

```
v1
message_id=<id>
device_id=<id>
type=<type>
timestamp_ms=<ms>
nonce=<base64>
seq=<n>
payload_sha256=<hex>
```

## 消息类型

- `session.create` - 创建 session
- `session.input` - 发送输入
- `session.stop` - 停止 session
- `file.upload` - 上传文件
- `echo.ping` - Echo 测试

## 错误码

INVALID_JSON, MISSING_FIELD, DEVICE_NOT_TRUSTED, DEVICE_REVOKED, BAD_TIMESTAMP, REPLAY_DETECTED, BAD_SEQUENCE, BAD_PAYLOAD_HASH, BAD_SIGNATURE, INTERNAL_ERROR
