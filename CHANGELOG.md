# Changelog

## 1.0.0 (2026-05-14)

### 核心功能
- 全局语音输入：按下快捷键开始录音，再按停止
- 五种输出模式：原话 / 优化 / 速问 / Markdown / 自定义
- 文本自动注入光标位置（剪贴板 + Ctrl+V 主方案，SendInput 回退）
- 录音悬浮窗指示器（录音中 / 处理中状态）

### 服务商接入
- LLM：OpenAI、DeepSeek、通义千问、豆包、Anthropic Claude、Google Gemini、百度文心一言
- ASR：OpenAI Whisper、讯飞极速听写、火山引擎语音、WhisperKey 官方中转
- API Key 使用 Windows DPAPI 加密存储

### 许可与激活
- 原话模式终生免费
- 6 位激活码解锁全部模式
- 设备指纹绑定（MachineGuid），防拷贝

### 其他
- 历史记录 SQLite 存储，支持搜索/过滤/分页
- 快捷键自定义（2-3 键组合，至少 1 修饰键）
- 录音时长限制 / 静音自动停止
- 单实例运行，启动最小化到托盘
- 暗色主题 UI（Naive UI）
- NSIS 安装包 ≤ 25MB
