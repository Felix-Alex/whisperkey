# 语灵听写 WhisperKey

Windows 11 桌面端全局语音输入工具。按下快捷键开始录音，再按一次停止，语音经云端 ASR 识别和 AI 优化后，直接注入当前光标位置。
(当前LLM服务商只测试了deepseek，ASR服务商仅测试了火山引擎语音，未测试的服务商，不确认能用，但是对于AI初学者友好，这两个服务商都近乎免费。）

DeepSeek开放平台：https://platform.deepseek.com/api_keys （获取API Key，充值10块钱够用几个月了）

火山引擎：(https://console.volcengine.com/auth/login/) （在新版页面，在豆包语音模型选择**流式语音模型版2.0 小时版**，一般都有20小时免费试用 ，用完了再新建一个应用又可以免费用，基本实现全免费。如果不知道如何获取X-Api-Key，直接问豆包）

***软件是免费的，激活码：N9fA2d***

安装、链接API等任何问题，欢迎与我联系。

**一定要尝试markdown格式的输入，你一句话，它就帮你生成专业的markdown格式的提示词~**

## 系统要求

| 项 | 最低要求 |
|---|---|
| 操作系统 | Windows 10 22H2+ / Windows 11 |
| 架构 | x64 / ARM64 |
| 内存 | 4 GB |
| 磁盘 | 100 MB |
| 网络 | 需联网调用 ASR / LLM API |

## 安装

1. 下载 `WhisperKey_Setup_<version>_x64.exe`
2. 双击安装（无需管理员权限，安装到 `%LOCALAPPDATA%\Programs\WhisperKey`）
3. 首次启动后，右键托盘图标进入"设置"
4. 配置 LLM 和 ASR 服务商 API Key
5. 开始使用

## 软件截图
<img width="1076" height="789" alt="image" src="https://github.com/user-attachments/assets/13b7b77b-43f7-4d37-bcca-3fda1cd54c91" />
<img width="1076" height="789" alt="image" src="https://github.com/user-attachments/assets/3400205a-39b5-4f86-8d4c-5b067f1714b6" />
<img width="1076" height="789" alt="image" src="https://github.com/user-attachments/assets/3001c571-6fe5-4b50-8f6a-ac3b1c69c963" />
<img width="1076" height="789" alt="image" src="https://github.com/user-attachments/assets/a7e97562-3c11-4901-bcc4-d4c7796fac6e" />

## 五种输出模式

| 模式 | 快捷键后 | 说明 |
|------|---------|------|
| **原话模式** | ASR → 直接输出 | 语音转文字，不经过 AI，终生免费 |
| **优化模式** | ASR → LLM 润色 | 去除口语词，优化为书面语言 |
| **速问模式** | ASR → LLM 回答 | 语音提问，AI 快速给出答案 |
| **Markdown 模式** | ASR → LLM 生成 | 口述需求转为结构化 AI 提示词 |
| **自定义模式** | ASR → 自定义 Prompt | 使用自设 prompt 处理语音结果 |

原话模式终生免费，其余四种模式需激活后使用。

## 快捷键

默认快捷键：`Alt + J`

可在"设置 → 快捷键"中自定义，支持 2-3 键组合（必须包含至少一个修饰键 Ctrl/Alt/Shift/Win）。

## 服务商配置

### LLM（AI 后处理）
- OpenAI / DeepSeek / 通义千问 / 豆包 — OpenAI 兼容协议
- Anthropic Claude / Google Gemini / 百度文心一言

### ASR（语音识别）
- OpenAI Whisper / 讯飞极速听写 / 火山引擎语音 / WhisperKey 官方中转

在"设置 → 服务商"中填入 API Key 并保存，可使用"测试连接"按钮验证。

## 历史记录

所有录音识别结果自动保存，可在"设置 → 历史"中查看、搜索、复制、删除。默认保留 7 天。

## 激活

- 在"设置 → 激活"中输入 6 位激活码
- 未激活状态下，仅原话模式可用
- 激活后解锁全部五种输出模式

## 常见问题

**Q: 按下快捷键无反应？**
A: 检查快捷键是否被其他软件占用，尝试更换组合键。

**Q: 录音后无文字输出？**
A: 检查麦克风权限（Windows 设置 → 隐私 → 麦克风），确认 ASR API Key 有效。

**Q: LLM 模式返回错误？**
A: 检查 LLM API Key 和配额。非原话模式需要激活。

**Q: 如何卸载？**
A: 通过 Windows 设置 → 应用，或运行安装目录下的 `uninstall.exe`。卸载时可选保留配置和历史。

**Q: SmartScreen 弹出"未知发布者"？**
A: V1.0 使用自签名，点击"更多信息 → 仍要运行"即可。后续版本将使用 EV 代码签名证书消除此提示。

## 技术栈

Tauri 2 + Vue 3 + Naive UI + Rust

## 开发

```bash
pnpm install
pnpm tauri dev       # 开发模式
pnpm tauri build     # 生产构建
cargo test           # 运行测试
```

## License

专有软件。原话模式终生免费，其余模式需激活使用。
