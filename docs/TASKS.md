# 语灵听写 WhisperKey - 原子化任务清单（TASKS）

> 本清单是 AI 编码工具实施开发的**唯一执行路径**。所有任务按依赖顺序排列，每个任务为单次会话可完成的原子任务。
>
> 操作规范（每个 AI 会话开始前必读）：
> 1. 读取本文件，找到第一个 `[ ]` 未完成任务
> 2. 仅完成该任务，完成后将 `[ ]` 改为 `[x]`
> 3. 严禁跨任务实现
> 4. 完成后 `cargo check` / `pnpm build` 必须通过
> 5. 每完成一个 Phase 提交一次 git commit

---

## Phase 0：清理废弃模块

> 目标：删除多模态音频直接输入 LLM 的实验性代码，恢复干净的 ASR + LLM 双步架构。

- [x] **T001 [cleanup] 删除多模态音频 LLM 实验模块**
  - 📥 输入：TECH_SPEC.md §2.1
  - 🔧 做什么：删除 `src-tauri/src/llm/audio_llm.rs`、`doubao_audio.rs`、`gemini_audio.rs`、`openai_audio.rs`、`qwen_audio.rs`；从 `llm/mod.rs` 中移除对这些模块的 `pub mod` 声明和任何引用
  - 📤 产出：删除 5 个文件，修改 `llm/mod.rs`
  - ✅ 验收：`cargo check` 通过；`grep -r "audio_llm\|doubao_audio\|gemini_audio\|openai_audio\|qwen_audio" src-tauri/src/` 无结果
  - 🔗 依赖：无

- [x] **T002 [cleanup] 清理 ASR 模块（为重建做准备）**
  - 📥 输入：T001；TECH_SPEC.md §10.2
  - 🔧 做什么：确认 `asr/` 目录下文件状态；若旧 asr 文件已被删除则重新创建 `asr/mod.rs`、`asr/trait.rs` 占位；确保 `lib.rs` 中 `pub mod asr;` 声明存在
  - 📤 产出：`asr/` 目录结构就绪（mod.rs + trait.rs 骨架）
  - ✅ 验收：`cargo check` 通过
  - 🔗 依赖：T001

- [x] **T003 [cleanup] 清理 LLM 模块（为单 Provider 重构做准备）**
  - 📥 输入：T001；TECH_SPEC.md §3.1
  - 🔧 做什么：若旧 llm provider 文件已被删除则重新创建 `llm/trait.rs`、`llm/openai.rs`、`llm/mod.rs` 占位；从 `lib.rs` 确保声明正确；删除旧的 per-mode provider 分发逻辑残留
  - 📤 产出：精简后的 `llm/` 目录（trait.rs + openai.rs + prompts.rs + mod.rs 骨架）
  - ✅ 验收：`cargo check` 通过
  - 🔗 依赖：T001

---

## Phase 1：数据结构重构（Config v2 + TypeScript 类型）

> 目标：先改数据，后续所有后端/前端逻辑基于新的 v2 schema 构建。

- [x] **T004 [config] 重写 Config v2 Schema（Rust）**
  - 📥 输入：T003；TECH_SPEC.md §6.1
  - 🔧 做什么：重写 `src-tauri/src/config/schema.rs`：
    1. 删除 `ModesConfig`、`ModeAssignment`、`ProvidersConfig` 及所有 `*Credential` 结构体
    2. 新增 `LlmConfig { provider, api_key, base_url, model }`
    3. 扩展 `AsrConfig { provider, api_key, base_url, model, language }`（替换旧的仅有 default+language 的定义）
    4. `Config.version` 默认值改为 `2`
    5. 更新 `Default` impl 和所有默认值函数
    6. 更新单元测试（覆盖新字段、camelCase 序列化）
  - 📤 产出：`config/schema.rs`
  - ✅ 验收：`cargo test -- config::schema` 全绿；`serde_json::to_string_pretty(&Config::default())` 输出与 TECH_SPEC §6.1 示例一致
  - 🔗 依赖：T003

- [x] **T005 [config] 实现 v1 → v2 配置迁移**
  - 📥 输入：T004；TECH_SPEC.md §6.1 迁移规则
  - 🔧 做什么：在 `config/persist.rs` 中实现 `migrate_v1_to_v2(old_json)`：
    1. 检测 version=1
    2. 备份原文件 `config.json.bak.<ts>`
    3. 映射 `asr.default` → `asr.provider`
    4. 丢弃 `modes` 和 `providers` 字段
    5. 初始化 `llm` 为默认值
    6. 保留 hotkey/audio/ui/system/history/advanced
    7. 写入 version=2
  - 📤 产出：`config/persist.rs` 增量
  - ✅ 验收：单测：v1 JSON 字符串 → 迁移后 → v2 Config 反序列化成功；备份文件存在且内容为原 v1 JSON
  - 🔗 依赖：T004

- [x] **T006 [config] 简化 API Key 加密存取适配器**
  - 📥 输入：T004；TECH_SPEC.md §6.4
  - 🔧 做什么：更新 `config/secrets.rs`（若不存在则创建）：提供 `set_llm_key(key)` / `get_llm_key()` 和 `set_asr_key(key)` / `get_asr_key()`，自动 DPAPI 加密/解密 + base64 编解码；DPAPI 附加熵使用 `"WhisperKey-v2-Salt"`；删除旧的 `set_provider_key(provider, key)` 多厂商接口
  - 📤 产出：`config/secrets.rs`
  - ✅ 验收：set→save→reload→get roundtrip 单测通过
  - 🔗 依赖：T004

- [x] **T007 [types] 重写前端 TypeScript 类型定义**
  - 📥 输入：T004；TECH_SPEC.md §6.1
  - 🔧 做什么：重写 `src/types/index.ts`：
    1. 删除 `ModesConfig`、`ModeAssignment`、`ProvidersConfig`、`ProviderCredential` 及子类型
    2. 新增 `LlmConfig { provider, apiKey, baseUrl, model }`
    3. 重写 `AsrConfig { provider, apiKey, baseUrl, model, language }`
    4. 更新 `Config` 接口（删除 modes/providers，新增 llm）
    5. 更新 `HistoryItem.mode` 联合类型为 `'raw' | 'polish' | 'markdown' | 'humor' | 'venomous'`
    6. 更新 `LicenseStatus` → `{ activated: boolean, expiresAt?: number }`（删除 products 数组）
    7. 新增 `OutputMode = 'raw' | 'polish' | 'markdown' | 'humor' | 'venomous'`
  - 📤 产出：`src/types/index.ts`
  - ✅ 验收：`pnpm build` 类型检查通过（可能有前端组件报错，仅需确认类型文件本身无语法错误）
  - 🔗 依赖：T004

- [x] **T008 [history] 更新 SQLite 表结构迁移**
  - 📥 输入：T004；TECH_SPEC.md §6.2
  - 🔧 做什么：在 `history/migrations.rs` 新增 v2 迁移：`ALTER TABLE history` 的 CHECK 约束无法直接修改，执行 "创建新表 → 迁移数据 → 删旧表 → 重命名" 以更新 `mode` 字段 CHECK 约束（新增 humor/venomous）；或使用更宽松的 CHECK 策略
  - 📤 产出：`history/migrations.rs` 增量
  - ✅ 验收：单测：新 DB 创建时 CHECK 约束包含 5 种模式；旧 DB 迁移后数据不丢失
  - 🔗 依赖：T004

- [x] **T009 [license] 简化 License 数据结构**
  - 📥 输入：T004；TECH_SPEC.md §6.3
  - 🔧 做什么：更新 `license/` 下相关 struct：将 `products: Vec<String>` 替换为 `activated: bool`；更新 `LicenseStore::is_unlocked(product)` → `is_activated()`；更新 `verifier.rs` 验签逻辑（payload 不再包含 products）；更新 `activator.rs` 激活接口
  - 📤 产出：`license/` 下修改的多个文件
  - ✅ 验收：`cargo test -- license` 全绿；合法 license 返回 activated=true；篡改 license 验签失败
  - 🔗 依赖：T004

---

## Phase 2：后端核心逻辑重构

> 目标：ASR trait 实现、单 LLM Provider、Pipeline 状态机（ASR→LLM 双步）、快捷键、激活门控。

- [x] **T010 [asr] 重建 AsrProvider trait + Registry**
  - 📥 输入：T002；TECH_SPEC.md §3、§10.2
  - 🔧 做什么：`asr/trait.rs` 定义 `#[async_trait] trait AsrProvider { async fn transcribe(wav: Vec<u8>, config: &AsrConfig) -> AppResult<AsrResponse>; fn name(&self) -> &'static str; }`；`asr/mod.rs` 实现 `AsrRegistry` 根据 `AsrConfig.provider` 字符串派发 provider 实例
  - 📤 产出：`asr/trait.rs`、`asr/mod.rs`
  - ✅ 验收：mock provider 注册并取出；`cargo check` 通过
  - 🔗 依赖：T002, T004

- [x] **T011 [asr] OpenAI Whisper ASR 实现**
  - 📥 输入：T010；TECH_SPEC.md §10.2
  - 🔧 做什么：`asr/openai.rs` 实现 `OpenAiAsr`：reqwest multipart POST `/v1/audio/transcriptions`；Authorization Bearer；超时 10s；401→E_ASR_AUTH / 429→E_ASR_QUOTA / 5xx 重试 1 次
  - 📤 产出：`asr/openai.rs`
  - ✅ 验收：wiremock 本地测试：200 OK 返回正确文本；401 返回 E_ASR_AUTH
  - 🔗 依赖：T010

- [x] **T012 [asr] 讯飞 + 火山引擎 ASR 实现**
  - 📥 输入：T010；TECH_SPEC.md §10.2
  - 🔧 做什么：`asr/xfyun.rs`（HmacSHA1 签名 + multipart 上传）、`asr/volcengine.rs`（X-Api-App-Key + X-Api-Access-Key header + base64 audio）；实现 AsrProvider trait
  - 📤 产出：`asr/xfyun.rs`、`asr/volcengine.rs`
  - ✅ 验收：各自 mock 通
  - 🔗 依赖：T010

- [x] **T013 [asr] 官方免费 ASR 中转 + 注册表完成**
  - 📥 输入：T011, T012；TECH_SPEC.md §10.2
  - 🔧 做什么：`asr/official.rs` 实现官方中转；`asr/mod.rs` 注册全部 4 个 provider（openai/xfyun/volcengine/official）
  - 📤 产出：`asr/official.rs`、`asr/mod.rs` 增量
  - ✅ 验收：mock 通；`AsrRegistry::get("openai")` / `AsrRegistry::get("xfyun")` 等均返回正确实现
  - 🔗 依赖：T011, T012

- [x] **T014 [llm] 重建 LlmProvider trait + Prompt 模板加载**
  - 📥 输入：T003, T004；TECH_SPEC.md §5、§10.1
  - 🔧 做什么：
    1. `llm/trait.rs` 定义 `#[async_trait] trait LlmProvider { async fn chat(system_prompt: &str, user_text: &str, config: &LlmConfig) -> AppResult<String>; fn name(&self) -> &'static str; }`
    2. `llm/prompts.rs` 实现从 `resources/prompts/{mode}.md` 加载模板文件，用 `{{TEXT}}` 占位符替换；暴露 `render_prompt(mode: &str, text: &str) -> AppResult<String>`
  - 📤 产出：`llm/trait.rs`、`llm/prompts.rs`
  - ✅ 验收：单测：模板渲染后包含原文且不含 `{{TEXT}}`；不存在的 mode 返回 error
  - 🔗 依赖：T003, T004

- [x] **T015 [llm] OpenAI 兼容协议 LLM 实现**
  - 📥 输入：T014；TECH_SPEC.md §10.1
  - 🔧 做什么：`llm/openai.rs` 实现 `OpenAiCompatibleLlm`：POST `{baseUrl}/chat/completions`；Authorization Bearer；body `{model, messages: [system, user], temperature: 0.2, max_tokens: 2048}`；超时 15s；401→E_LLM_AUTH / 429→E_LLM_QUOTA / 5xx 重试 1 次；响应解析 `choices[0].message.content`
  - 📤 产出：`llm/openai.rs`
  - ✅ 验收：wiremock 测试：200 OK 返回正确文本；错误码映射正确
  - 🔗 依赖：T014

- [x] **T016 [llm] Anthropic + Gemini + 文心一言 LLM 实现**
  - 📥 输入：T014；TECH_SPEC.md §10.1
  - 🔧 做什么：`llm/anthropic.rs`（x-api-key header + anthropic-version）；`llm/gemini.rs`（`{model}:generateContent?key=` + system_instruction）；`llm/ernie.rs`（OAuth2 access_token 换发 + 缓存 25min + `/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/`）
  - 📤 产出：`llm/anthropic.rs`、`llm/gemini.rs`、`llm/ernie.rs`
  - ✅ 验收：各自 mock 通
  - 🔗 依赖：T014

- [x] **T017 [llm] LlmRegistry 单 Provider 派发**
  - 📥 输入：T015, T016；TECH_SPEC.md §10.1
  - 🔧 做什么：`llm/mod.rs` 实现 `LlmRegistry::resolve(config: &LlmConfig) -> Arc<dyn LlmProvider>`：根据 `config.provider` 字符串选择实现（"openai"/"deepseek"/"qwen"/"doubao" 均走 OpenAiCompatibleLlm，仅 baseUrl + model 不同）；其他走各自实现；provider 初始化时从 config 读取 api_key（DPAPI 解密后内存中持有）
  - 📤 产出：`llm/mod.rs`
  - ✅ 验收：单测验证不同 provider 字符串派发到正确实现；`cargo test -- llm` 全绿
  - 🔗 依赖：T015, T016

- [x] **T018 [pipeline] 重写状态机：ASR + LLM 双步走**
  - 📥 输入：T010, T017；TECH_SPEC.md §7.1
  - 🔧 做什么：重写 `pipeline/state_machine.rs`：更新 `PipelineState` 枚举为 Idle → Recording → AsrTranscribing → LlmProcessing（仅非 raw 模式） → Injecting → Idle；raw 模式从 AsrTranscribing 直接跳转到 Injecting（跳过 LlmProcessing）；状态切换通过 `tokio::sync::watch` 广播
  - 📤 产出：`pipeline/state_machine.rs`
  - ✅ 验收：单测覆盖：raw 模式路径（无 LlmProcessing）、polish 模式路径（经过 LlmProcessing）、ASR 失败路径、LLM 失败路径
  - 🔗 依赖：T010, T017

- [x] **T019 [pipeline] 运行编排器：全链路集成**
  - 📥 输入：T018；TECH_SPEC.md §3.1、§11.2
  - 🔧 做什么：`pipeline/mod.rs` 实现 `Pipeline::run()`：
    1. 订阅 hotkey 事件
    2. 按状态机驱动 audio→asr→[llm]→inject→history 全链路
    3. raw 模式直接跳过 LLM 阶段
    4. 非 raw 模式在 LlmProcessing 前校验 `license.is_activated()`，未激活返回 `E_LICENSE_REQUIRED`
    5. 模式选择：从当前用户选择的模式（存储在 AppState 中）读取
  - 📤 产出：`pipeline/mod.rs`
  - ✅ 验收：手动 e2e mock：raw 模式全链路不调用 LLM；polish 模式调用 LLM；未激活时非 raw 模式被拦截
  - 🔗 依赖：T018

- [x] **T020 [hotkey] 重写快捷键约束与保存逻辑**
  - 📥 输入：T004；TECH_SPEC.md §8
  - 🔧 做什么：更新 `hotkey/registrar.rs` 和 `hotkey/mod.rs`：
    1. `HotkeyConfig` 校验：必须 2-3 键组合、至少 1 个修饰键（Ctrl/Alt/Shift/Win）
    2. 新增 `validate(&self) -> AppResult<()>` 方法
    3. register 仅在 IPC 命令调用时执行（不是自动触发）
    4. 注册成功后永不重置
    5. 快捷键冲突时返回 `E_HOTKEY_REGISTER`
    6. 非法组合返回 `E_HOTKEY_INVALID`
  - 📤 产出：`hotkey/mod.rs`、`hotkey/registrar.rs` 增量
  - ✅ 验收：单测：Ctrl+Shift+Space 通过校验；单键 Space 失败；无修饰键组合失败；4 键组合失败
  - 🔗 依赖：T004

---

## Phase 3：Prompt 文件与 IPC 命令层

> 目标：补齐 5 种模式的 prompt 文件，暴露全部 Tauri command 给前端。

- [x] **T021 [prompts] 创建 5 个 Prompt 模板文件**
  - 📥 输入：TECH_SPEC.md §5.1-§5.5
  - 🔧 做什么：在 `src-tauri/resources/prompts/` 下创建/更新：
    1. `raw.md` — 按 §5.1 完整内容
    2. `polish.md` — 按 §5.2 完整内容
    3. `markdown.md` — 按 §5.3 完整内容（含完整 system prompt）
    4. `humor.md` — 按 §5.4 完整内容（新建）
    5. `venomous.md` — 按 §5.5 完整内容（新建）
  - 📤 产出：5 个 .md 文件
  - ✅ 验收：每个文件可被 `std::fs::read_to_string` 读取，包含 `{{TEXT}}` 占位符；文件编码 UTF-8 无 BOM
  - 🔗 依赖：无

- [x] **T022 [ipc] 重写 Tauri Command 全集**
  - 📥 输入：T004, T009, T019, T020；TECH_SPEC.md §6.1
  - 🔧 做什么：在 `ipc/` 下重新声明全部 `#[tauri::command]`：
    - 配置：`cmd_config_get` / `cmd_config_set` / `cmd_config_save`（新增显式保存命令）
    - LLM：`cmd_llm_set_config` / `cmd_llm_test_connection`
    - ASR：`cmd_asr_set_config` / `cmd_asr_test_connection`
    - 快捷键：`cmd_hotkey_set` / `cmd_hotkey_get` / `cmd_hotkey_validate`（保存时调用 register）
    - 录音：`cmd_recording_toggle` / `cmd_recording_get_state` / `cmd_set_output_mode`
    - 许可：`cmd_license_status` / `cmd_license_activate`
    - 历史：`cmd_history_list` / `cmd_history_delete` / `cmd_history_clear`
    - 系统：`cmd_app_quit` / `cmd_app_open_logs_folder`
    - 删除旧 ipc 命令（如 `cmd_config_set_provider_key`、`cmd_provider_test_connection`）
  - 📤 产出：`ipc/config.rs`、`ipc/recording.rs`、`ipc/license.rs`、`ipc/providers.rs`（或重命名为 `ipc/llm.rs`）、`ipc/history.rs` 增量
  - ✅ 验收：`cargo check` 通过；所有命令在 `lib.rs` 中注册
  - 🔗 依赖：T004, T009, T019, T020

- [x] **T023 [ipc] 注册 AppState 与全局事件**
  - 📥 输入：T022
  - 🔧 做什么：更新 `app_state.rs`：管理 `ConfigStore`、`LicenseStore`、`LlmRegistry`、`AsrRegistry`、`PipelineHandle`、当前输出模式（`OutputMode`）；启动时初始化全部单例；更新 `lib.rs` 注入 AppState 到所有 command
  - 📤 产出：`app_state.rs`、`lib.rs`
  - ✅ 验收：`cargo check` 通过；`pnpm tauri dev` 启动无 panic
  - 🔗 依赖：T022

---

## Phase 4：前端 UI 重构

> 目标：所有 Vue 页面和组件按新架构重写，遵循 UI 视觉规范。

- [x] **T024 [ui] App.vue 外壳 + 路由重构**
  - 📥 输入：T023；TECH_SPEC.md §12
  - 🔧 做什么：更新 `src/App.vue`：Naive UI `n-config-provider` + `dark-theme` 跟随系统；左侧导航更新（设置页签：通用/快捷键/服务商/激活/历史/关于）；整体布局简约高端，充足 margin/padding 留白；避免强对比色调；更新 `router/index.ts` 路由定义
  - 📤 产出：`App.vue`、`router/index.ts`
  - ✅ 验收：`pnpm tauri dev` 主窗口正常渲染，导航可切换
  - 🔗 依赖：T023

- [x] **T025 [ui] General.vue 通用设置页**
  - 📥 输入：T024；TECH_SPEC.md §6.1
  - 🔧 做什么：重写 `views/Settings/General.vue`：
    1. 模式选择下拉（5 种模式：原话/优化/Markdown/幽默/毒舌），设为默认输出模式
    2. 开机自启开关
    3. 启动最小化开关
    4. 检查更新开关
    5. 录音最大时长滑块/输入
    6. 静音自动停止开关 + 时长
    7. 显式"保存"按钮，点击后调 `cmd_config_save`
  - 📤 产出：`views/Settings/General.vue`
  - ✅ 验收：所有开关/下拉可操作，保存后刷新页面值不丢失
  - 🔗 依赖：T024

- [x] **T026 [ui] Hotkey.vue 快捷键设置页 + HotkeyInput 组件**
  - 📥 输入：T024；TECH_SPEC.md §8
  - 🔧 做什么：
    1. 重写 `components/HotkeyInput.vue`：
       - 聚焦后捕获键盘事件（keydown + keyup）
       - **必须调用 `e.preventDefault()` 拦截浏览器默认行为**
       - 维护当前按下的修饰键集合 + 最后按下的非修饰键
       - 松开所有键后合成快捷键字符串（如 "Ctrl+Shift+Space"）
       - 限制 2-3 键，少于 2 或多于 3 键时红色提示
       - 无修饰键时红色提示"必须包含至少一个修饰键"
    2. 重写 `views/Settings/Hotkey.vue`：
       - 展示当前快捷键
       - HotkeyInput 组件
       - 显式"保存"按钮，点击后调用 `cmd_hotkey_validate` + `cmd_hotkey_set`
       - 冲突时红字提示"快捷键被占用，请重新设置"
       - 保存成功绿色提示"快捷键已生效"
  - 📤 产出：`components/HotkeyInput.vue`、`views/Settings/Hotkey.vue`
  - ✅ 验收：手动测试：按下 Ctrl+P 不触发浏览器打印；无修饰键时红色提示；保存后快捷键生效
  - 🔗 依赖：T024

- [x] **T027 [ui] Providers.vue 服务商页（全局 LLM + ASR）**
  - 📥 输入：T024；TECH_SPEC.md §6.1、§10.1、§10.2
  - 🔧 做什么：重写 `views/Settings/Providers.vue`：
    1. **全局 LLM 配置区**：Provider 下拉（OpenAI/DeepSeek/Qwen/Doubao/Anthropic/Gemini/Ernie）+ API Key 输入框（密码遮罩）+ Base URL 输入框 + Model 输入框 + "测试连接"按钮 + 状态指示（绿色已连接/红色失败/灰色未测试）
    2. **ASR 配置区**：Provider 下拉（OpenAI/Xfyun/Volcengine/Official）+ API Key 输入框 + Base URL 输入框 + Model 输入框 + Language 下拉 + "测试连接"按钮 + 状态指示
    3. 显式"保存"按钮，点击后调 `cmd_llm_set_config` + `cmd_asr_set_config` + `cmd_config_save`
    4. 未保存离开页面时提示"有未保存的更改"
  - 📤 产出：`views/Settings/Providers.vue`
  - ✅ 验收：填入有效 API Key → 测试连接 → 绿色 OK；填入无效 Key → 红色失败；保存后刷新不丢失
  - 🔗 依赖：T024

- [x] **T028 [ui] Activation.vue 激活页（极简版）**
  - 📥 输入：T024；TECH_SPEC.md §11
  - 🔧 做什么：重写 `views/Settings/Activation.vue`：
    1. 仅一个激活码输入框（**无 placeholder 提示词**）
    2. 仅一个"激活"按钮
    3. **彻底删除重置激活功能**及其相关代码
    4. 未激活时显示文案："未激活状态下，仅'原话模式'可用。"
    5. 已激活时显示文案："恭喜您，解锁了所有的输出模式。"
    6. 激活按钮调用 `cmd_license_activate`，成功后更新 `licenseStore`
  - 📤 产出：`views/Settings/Activation.vue`
  - ✅ 验收：未激活状态文案正确；输入有效激活码→成功→文案切换；无效激活码→红字错误提示
  - 🔗 依赖：T024

- [x] **T029 [ui] 模式门控拦截（前端 + License Store）**
  - 📥 输入：T025, T028；TECH_SPEC.md §11.2
  - 🔧 做什么：
    1. 更新 `stores/license.ts`：简化为 `{ activated: boolean }` 状态；启动时调 `cmd_license_status`；提供 `isActivated` getter
    2. 在模式选择 UI 中（General.vue 或全局）：
       - 未激活时，除 `raw` 外的 4 种模式均设置 `disabled` 属性
       - 每个 disabled 模式旁显示红色文字"需激活后使用"
       - 激活后红色文字消失，全量开放选择
  - 📤 产出：`stores/license.ts`、相关 Vue 组件增量
  - ✅ 验收：未激活时 polish/markdown/humor/venomous 选项灰色不可点击 + 红字提示；激活后全部可选
  - 🔗 依赖：T025, T028

- [x] **T030 [ui] History.vue 历史记录页**
  - 📥 输入：T024
  - 🔧 做什么：更新 `views/Settings/History.vue` 和 `components/HistoryItem.vue`：模式过滤更新为 5 种；分页 50 条；搜索框 + 日期过滤；HistoryItem 显示模式中文名 + 时间 + 摘要 + 复制/删除按钮
  - 📤 产出：`views/Settings/History.vue`、`components/HistoryItem.vue`
  - ✅ 验收：历史列表正确显示 5 种模式标签；搜索/过滤/分页正常
  - 🔗 依赖：T024

- [x] **T031 [ui] About.vue 关于页 + 全局样式**
  - 📥 输入：T024
  - 🔧 做什么：更新 `views/Settings/About.vue`（版本号、检查更新、链接）；更新 `styles/global.scss`：全局排版优化（增加 margin/padding 留白、简约风格、柔和色调）
  - 📤 产出：`views/Settings/About.vue`、`styles/global.scss`
  - ✅ 验收：整体 UI 符合"简约、高端、排版舒适"视觉规范
  - 🔗 依赖：T024

---

## Phase 5：集成测试与品质打磨

- [x] **T032 [test] Rust 单元测试全覆盖**
  - 📥 输入：T001–T023
  - 🔧 做什么：确保核心模块单元测试覆盖：
    - `config/schema.rs`：default、roundtrip、camelCase、v1→v2 迁移
    - `crypto/dpapi.rs`：encrypt/decrypt roundtrip
    - `license/verifier.rs`：合法/篡改/过期/指纹不符
    - `pipeline/state_machine.rs`：raw 路径 / polish 路径 / 各失败分支
    - `hotkey/mod.rs`：校验合法/非法组合
    - `llm/prompts.rs`：模板渲染
  - 📤 产出：各模块 `#[cfg(test)]` 增量
  - ✅ 验收：`cargo test` 全绿
  - 🔗 依赖：T001–T023

- [x] **T033 [test] 集成测试 12 条用例**
  - 📥 输入：T032；TECH_SPEC.md §16.2
  - 🔧 做什么：在 `src-tauri/tests/` 下实现 IT-01 至 IT-12；用 wiremock 启动 mock ASR/LLM 服务器
  - 📤 产出：`tests/it_*.rs`
  - ✅ 验收：`cargo test --test 'it_*'` 全绿
  - 🔗 依赖：T032

- [x] **T034 [perf] 启动性能优化 + 未使用代码清理**
  - 📥 输入：T033
  - 🔧 做什么：lazy load ASR/LLM registry；前端路由按需 import；确认无残留的旧架构代码引用；`cargo clippy -- -D warnings` 零告警
  - 📤 产出：调整若干文件
  - ✅ 验收：冷启动到托盘可见 ≤ 1.5s（SSD）；`cargo clippy` 零告警
  - 🔗 依赖：T033

---

## Phase 6：打包与分发

- [x] **T035 [build] 安装包配置**
  - 📥 输入：T034；TECH_SPEC.md §15
  - 🔧 做什么：`tauri.conf.json` bundle.windows.nsis 完善（installMode=perUser、自定义图标、桌面快捷方式可选、中文安装界面）；确认 ≤25MB
  - 📤 产出：`tauri.conf.json` + NSIS 模板
  - ✅ 验收：`pnpm tauri build` 产出 ≤25MB 单 .exe
  - 🔗 依赖：T034

- [x] **T036 [build] CI 流水线 + 代码签名占位**
  - 📥 输入：T035
  - 🔧 做什么：`.github/workflows/release.yml`（windows-latest、x64 + ARM64 matrix、release 触发）；`scripts/sign.ps1` 签名脚本占位
  - 📤 产出：`release.yml`、`sign.ps1`
  - ✅ 验收：tag push 后产物上传成功；签名脚本干跑通过
  - 🔗 依赖：T035

- [x] **T037 [doc] README + V1.0 发布检查**
  - 📥 输入：T036
  - 🔧 做什么：README（安装、快捷键、5 种模式介绍、激活、FAQ）；CHANGELOG.md 1.0.0；PRD V1.0 验收手测；打 tag v1.0.0
  - 📤 产出：README.md、CHANGELOG.md、git tag
  - ✅ 验收：新用户照文档可在 5 分钟内完成首次使用
  - 🔗 依赖：T036
