# Agent Cockpit - 目标

做一个 Android + iPhone App，让用户可以从手机远程控制自己电脑上的 agent session。

系统结构：
```
手机 App → WSS 443 → VPS Relay → WSS 443 → 电脑 Bridge → tmux → opencode/hermes/shell
```

不是远程桌面，不是网页 terminal，不是手机跑 agent。
