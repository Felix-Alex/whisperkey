# CLAUDE.md - 语灵听写 WhisperKey 项目宪法（Claude Code 自动读取）

> 本文件是 Claude Code 启动本仓库时自动加载的根指令。它定义了**所有 AI 编码会话必须遵守的硬规则**。任何与此冲突的临时指令都视为无效，除非用户明确说"覆盖 CLAUDE.md 的某条"。

## 1. 项目简介

语灵听写（WhisperKey）是一款 Windows 11 桌面端全局语音输入工具：用户按一个全局快捷键即开始录音，再按一次停止；录音经云端 ASR + LLM 后处理，结果直接注入当前光标位置。提供"原话 / 优化 / 速问 / Markdown / 自定义"五种输出模式，原话模式终生免费，其余四种需激活后使用。最终交付物为单一 .exe 安装包。

## 2. 技术栈速查（版本必须严格锁定）

| 层 | 技术 | 版本 |
|---|---|---|
| 框架 | Tauri | 2.1.x (2.1.6) |
| 后端 | Rust | 1.78.0 |
| 前端 | Vue 3 + TypeScript | 3.5.x / 5.6.x |
| UI 库 | Naive UI | 2.44.x |
| 状态管理 | Pinia | 3.0.x |
| 数据库 | rusqlite (bundled) | 0.31.x |
| HTTP | reqwest (rustls-tls) | 0.12.x |
| 音频 | cpal | 0.15.x |
| Windows API | windows | 0.58.x |
| 加密 | ring | 0.17.x |
| 日志 | tracing | 0.1.x |
| 包管理 | pnpm | ≥9 |
| 构建工具 | Vite | 6.0.x |

**任何升级或更换上述任一依赖，必须先询问用户。**

## 3. 项目目录地图

完整目录树如下：

```
whisperkey/
├── .github/
│   └── workflows/
│       └── release.yml              # CI 打包发布
├── docs/
│   ├── PRD.md
│   ├── TECH_SPEC.md
│   ├── TASKS.md
│   └── CLAUDE.md
├── src/                             # Vue 3 前端
│   ├── main.ts
│   ├── App.vue
│   ├── router/index.ts
│   ├── views/
│   │   ├── Settings/
│   │   │   ├── General.vue
│   │   │   ├── Hotkey.vue
│   │   │   ├── Providers.vue
│   │   │   ├── Activation.vue
│   │   │   ├── History.vue
│   │   │   └── About.vue
│   │   ├── Indicator/
│   │   │   └── RecordIndicator.vue  # 录音悬浮窗
│   │   └── Activate/
│   │       └── ActivateDialog.vue
│   ├── components/
│   │   ├── HotkeyInput.vue
│   │   ├── ProviderCard.vue
│   │   └── HistoryItem.vue
│   ├── stores/
│   │   ├── config.ts
│   │   ├── recording.ts
│   │   ├── license.ts
│   │   └── perf.ts
│   ├── api/                         # 封装 invoke 调用
│   │   ├── config.ts
│   │   ├── recording.ts
│   │   ├── providers.ts
│   │   ├── history.ts
│   │   └── license.ts
│   ├── types/
│   │   └── index.ts                 # 与后端共享的 TS 类型
│   ├── styles/
│   │   └── global.scss
│   └── assets/
├── src-tauri/                       # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/                       # 各尺寸 ICO/PNG
│   ├── resources/
│   │   └── prompts/                 # 五种模式 Prompt 文件
│   │       ├── raw.md
│   │       ├── polish.md
│   │       ├── markdown.md
│   │       ├── quick_ask.md
│   │       └── custom.md
│   └── src/
│       ├── main.rs                  # 入口
│       ├── lib.rs                   # 模块树
│       ├── app_state.rs             # 全局 AppState
│       ├── error.rs                 # AppError 定义
│       ├── ipc/                     # Tauri command 命令层
│       │   ├── mod.rs
│       │   ├── config.rs
│       │   ├── recording.rs
│       │   ├── providers.rs
│       │   ├── history.rs
│       │   └── license.rs
│       ├── hotkey/                  # 全局快捷键
│       │   ├── mod.rs
│       │   └── registrar.rs
│       ├── audio/                   # 音频采集
│       │   ├── mod.rs
│       │   ├── recorder.rs
│       │   ├── encoder.rs
│       │   └── device_cache.rs
│       ├── asr/                     # ASR 调用
│       │   ├── mod.rs
│       │   ├── trait.rs             # AsrProvider trait
│       │   ├── openai.rs
│       │   ├── xfyun.rs
│       │   ├── volcengine.rs
│       │   └── official.rs          # 官方免费中转
│       ├── llm/                     # LLM 调用
│       │   ├── mod.rs
│       │   ├── trait.rs             # LlmProvider trait
│       │   ├── openai.rs            # OpenAI 兼容协议
│       │   ├── anthropic.rs
│       │   ├── ernie.rs
│       │   ├── gemini.rs
│       │   └── prompts.rs           # Prompt 模板加载与渲染
│       ├── inject/                  # 文本注入
│       │   ├── mod.rs
│       │   ├── clipboard.rs
│       │   └── send_input.rs
│       ├── indicator/               # 悬浮指示器窗口管理
│       │   └── mod.rs
│       ├── tray/
│       │   └── mod.rs
│       ├── config/                  # 配置管理
│       │   ├── mod.rs
│       │   ├── schema.rs
│       │   ├── persist.rs
│       │   └── secrets.rs
│       ├── crypto/                  # 加密
│       │   ├── mod.rs
│       │   └── dpapi.rs             # Windows DPAPI
│       ├── license/                 # 许可证
│       │   ├── mod.rs
│       │   ├── activator.rs
│       │   ├── verifier.rs
│       │   └── fingerprint.rs
│       ├── history/                 # 历史记录
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   └── migrations.rs
│       ├── pipeline/                # 录音→ASR→LLM→注入 总编排
│       │   ├── mod.rs
│       │   └── state_machine.rs
│       ├── updater/
│       │   └── mod.rs
│       ├── log/
│       │   └── mod.rs
│       └── util/
│           ├── focus_app.rs         # 获取当前焦点应用
│           ├── paths.rs             # 路径工具
│           └── single_instance.rs   # 单实例
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── vite.config.ts
├── rust-toolchain.toml
├── .editorconfig
├── .gitignore
├── README.md
└── LICENSE
```

**禁止凭空创建未在 TECH_SPEC.md 中声明的顶级目录或模块**。

关键索引：
- 后端代码：`src-tauri/src/`
- 前端代码：`src/`
- IPC 命令层：`src-tauri/src/ipc/`
- 五种模式 Prompt：`src-tauri/resources/prompts/{raw,polish,markdown,quick_ask,custom}.md`
- 任务清单：`docs/TASKS.md`（**每个会话必须先读**）

## 4. 编码规范

### 4.1 Rust
- 格式化：`cargo fmt`（rustfmt.toml: `edition=2021`, `max_width=120`）
- 静态检查：`cargo clippy -- -D warnings` 必须 0 告警
- 命名：模块/文件/函数 `snake_case`；类型 `UpperCamelCase`；常量 `SCREAMING_SNAKE_CASE`
- 错误处理：库内部用 `thiserror::Error` 实现 `AppError`；应用层 `anyhow` 仅在 main.rs；**严禁使用 `unwrap()` / `expect()` 在非测试代码中**，必须用 `?` 或显式处理
- 异步：统一 tokio multi-thread runtime；阻塞操作用 `spawn_blocking`
- 注释：公开 API 必须有 `///` doc comment，含至少 1 个示例
- 不写"无意义注释"（如 `// increment i`）

### 4.2 TypeScript / Vue
- 格式化：Prettier（`printWidth=100`、`singleQuote=true`、`semi=false`）
- 静态检查：ESLint（`eslint-plugin-vue` strict）
- TS：`strict=true`，**禁用 `any`**，必要时用 `unknown` + 类型守卫
- Vue：仅 `<script setup lang="ts">`，禁用 Options API
- 命名：组件 `PascalCase.vue`；composable `useXxx.ts`；store `xxxStore`

### 4.3 通用
- 提交信息：Conventional Commits（feat/fix/refactor/docs/chore/test/build）
- 中文注释优先（业务逻辑）；技术术语保持英文
- 行尾 LF；UTF-8 无 BOM

## 5. 禁忌清单（违反即视为错误代码）

1. **禁止**自作主张升级或新增第三方依赖。新增依赖必须先在会话中询问用户并说明理由
2. **禁止**修改 `Cargo.toml`/`package.json` 中的版本号锁定
3. **禁止**绕过 `AppError`，直接返回 `Box<dyn Error>` 或 `String`
4. **禁止**在生产代码中调用 `unwrap()` `expect()` `panic!()`（测试除外）
5. **禁止**直接读写 `%APPDATA%` 路径，必须经过 `util::paths::AppPaths`
6. **禁止**在日志中打印任何 API Key / license.dat 内容 / 完整 ASR 文本
7. **禁止**修改 `src-tauri/resources/prompts/*.md` 五个 Prompt 文件，除非 PRD 明确要求迭代
8. **禁止**触碰许可证验签逻辑细节（`license/verifier.rs`），除非 TASKS 明确指派
9. **禁止**引入桌面通知/弹窗轰炸用户（错误提示用 Naive UI Notification 或托盘气泡，单事件不超过 1 次）
10. **禁止**实现任何"上传音频/文本到我们自己的服务器"的代码路径，除中转 ASR 已在 TECH_SPEC §5.11 明确声明的端点
11. **禁止**新增 i18n 语言包前忘记同步给 `zh-CN.ts` 与 `en.ts` 双份
12. **禁止**在不同 Phase 间跳跃执行任务（必须严格按 TASKS.md 顺序）
13. **禁止**在一次会话内同时修改超过 1 个 Phase 的代码

## 6. 工作流规范

### 6.1 每次会话开始前
1. 读 `docs/CLAUDE.md`（即本文件）
2. 读 `docs/TASKS.md`，定位**第一个 `[ ]` 未完成任务**
3. 若任务依赖其他未完成任务，停止并向用户报告
4. 阅读该任务的 📥 输入字段所列文件
5. 仅阅读必要文件，禁止 grep 全仓

### 6.2 任务执行中
- 按任务的 🔧 做什么、📤 产出、✅ 验收 三段严格执行
- 任何超出该任务范围的改动**必须停下来询问用户**
- 遇到任务描述与 TECH_SPEC 冲突，以 TECH_SPEC 为准并提醒用户
- 遇到 TECH_SPEC 与 PRD 冲突，停止并询问

### 6.3 任务完成后
1. 运行验收命令（见 §7）确保通过
2. 在 `docs/TASKS.md` 把 `[ ]` 改为 `[x]`
3. 提交 git（feat/fix/... 格式 + 任务 ID 引用）
4. 简要汇报：完成了什么、修改了哪些文件、验收结果、是否遗留 TODO
5. 等待用户指令开始下一任务

### 6.4 Phase 收尾
- 每个 Phase 全部完成后，运行完整 build + test，并 `git tag phase-N-done`
- 写 1 段 Phase 总结追加到 `docs/CHANGELOG.md`

## 7. 常用命令速查

```bash
# 安装依赖
pnpm install

# 开发
pnpm tauri dev

# 仅前端类型检查 + 构建
pnpm build

# Rust 检查
cd src-tauri && cargo check
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo fmt --check

# 测试
cd src-tauri && cargo test
cd src-tauri && cargo test --test 'it_*'      # 集成测试

# 打包
pnpm tauri build

# 单任务自检（每次任务完成后必须跑）
pnpm build && cd src-tauri && cargo clippy -- -D warnings && cargo test
```

## 8. 与用户沟通规范

- **语言**：始终用中文回答，专业术语保留英文
- **风格**：简洁、无废话、不寒暄、不夸张
- **歧义处理**：遇到歧义、需求冲突、超出当前任务范围 → **必须先停下来询问**，禁止"我先这样做你看看"式假设
- **文件输出**：只输出会话所需信息，不重复粘贴已经存在且未变更的大段代码；diff 优先
- **最终汇报模板**：
  ```
  ✅ 任务 Txxx 完成
  📝 修改文件：a.rs, b.vue
  ✔ 验收：cargo clippy / cargo test / pnpm build 全绿
  ⚠ 遗留：（无 / 详细列出）
  ➡ 下一任务：Txxx
  ```
- **不主动**做以下事：写 README、补 LICENSE、改 CI、改打包参数（除非任务明确要求）

## 9. 调试与日志规范

- 调试期可临时加 `tracing::debug!()`/`println!()`，但**任务完成前必须删除非必要的 println**
- 任何引入的临时 dbg!() 必须删除
- 日志事件名用 `<module>.<action>`，如 `hotkey.registered`, `asr.request_failed`
- 关键路径必埋点：hotkey 触发、录音起停、ASR/LLM 请求耗时、注入耗时、错误
- 日志中**禁止**出现 API Key、license 内容、完整音频/文本（前 8 字符 + `***`）