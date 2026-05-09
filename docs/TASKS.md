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

## Phase 0：项目初始化与脚手架

- [x] **T001 [scaffold] 初始化 Tauri 2 项目**
  - 📥 输入：无
  - 🔧 做什么：用 `pnpm create tauri-app@latest whisperkey -- --manager pnpm --template vue-ts` 创建项目；指定 Tauri 2.1
  - 📤 产出：`whisperkey/` 完整脚手架；`package.json`、`Cargo.toml`、`tauri.conf.json`
  - ✅ 验收：`pnpm tauri dev` 能弹出默认窗口
  - 🔗 依赖：无

- [x] **T002 [scaffold] 锁定工具链与版本**
  - 📥 输入：T001
  - 🔧 做什么：创建 `rust-toolchain.toml`（channel="1.78.0"）；`package.json` engines 锁 pnpm>=9 node>=20；`.editorconfig`、`.gitignore`、`.gitattributes`
  - 📤 产出：上述 4 个文件
  - ✅ 验收：`rustup show` 显示 1.78.0；`pnpm install` 成功
  - 🔗 依赖：T001

- [x] **T003 [scaffold] 建立模块化目录结构**
  - 📥 输入：T002；TECH_SPEC.md §2
  - 🔧 做什么：在 `src-tauri/src` 下创建所有模块文件夹与 `mod.rs` 占位（hotkey/audio/asr/llm/inject/indicator/tray/config/crypto/license/history/pipeline/updater/log/util/ipc）；前端 `src/views`、`src/api` 同步建空文件
  - 📤 产出：完整目录树（空 mod.rs 仅 `pub mod x;`）
  - ✅ 验收：`cargo check` 通过；`pnpm build` 通过
  - 🔗 依赖：T002

- [x] **T004 [scaffold] 配置生产依赖**
  - 📥 输入：T003；TECH_SPEC.md §1.1
  - 🔧 做什么：`Cargo.toml` 添加 windows 0.58、cpal 0.15、hound 3.5、reqwest 0.12 (rustls-tls,multipart,json,gzip)、tokio (full)、serde、serde_json、thiserror、anyhow、tracing、tracing-subscriber、tracing-appender、rusqlite (bundled)、ring、base64、dirs、once_cell、tauri-plugin-clipboard-manager、tauri-plugin-autostart、tauri-plugin-single-instance、tauri-plugin-updater；前端 `pnpm add naive-ui pinia vue-router @tauri-apps/api`
  - 📤 产出：更新后的 Cargo.toml / package.json
  - ✅ 验收：`cargo build` 通过；前端依赖列入 lockfile
  - 🔗 依赖：T003

- [x] **T005 [scaffold] tauri.conf.json 基础配置**
  - 📥 输入：T004
  - 🔧 做什么：配置 productName=WhisperKey、identifier=app.whisperkey、main 窗口 860×600 默认隐藏、新增 indicator 窗口（按 §4.3 配置）；bundler.targets=["nsis"]；installer NSIS 选项
  - 📤 产出：tauri.conf.json
  - ✅ 验收：`pnpm tauri dev` 出主窗口；indicator 窗口可通过代码 show
  - 🔗 依赖：T004

---

## Phase 1：核心基础模块（配置/日志/加密/错误）

- [x] **T006 [error] 定义 AppError 与 AppResult**
  - 📥 输入：T005；TECH_SPEC.md §8.1
  - 🔧 做什么：在 `src-tauri/src/error.rs` 定义 `AppError`（thiserror enum，覆盖错误码表全部变体）+ `pub type AppResult<T> = Result<T, AppError>`；实现 `serde::Serialize` 以便 IPC 错误传递
  - 📤 产出：error.rs
  - ✅ 验收：单元测试覆盖每个错误变体序列化为 `{code, message}` JSON
  - 🔗 依赖：T005

- [x] **T007 [log] tracing 初始化**
  - 📥 输入：T006；TECH_SPEC.md §8.2
  - 🔧 做什么：实现 `log::init(level)` 滚动文件 + 控制台双输出；JSON 格式
  - 📤 产出：log/mod.rs
  - ✅ 验收：启动后 `%APPDATA%\WhisperKey\logs\app.YYYY-MM-DD.log` 有 INFO 行
  - 🔗 依赖：T006

- [x] **T008 [util] AppPaths 工具**
  - 📥 输入：T007
  - 🔧 做什么：实现 `AppPaths { config, license, history_db, logs_dir, prompts_dir }` 由 dirs crate 计算；首次访问自动 mkdir
  - 📤 产出：util/paths.rs
  - ✅ 验收：单测验证路径以 `%APPDATA%\WhisperKey\` 开头
  - 🔗 依赖：T007

- [x] **T009 [crypto] DPAPI 包装**
  - 📥 输入：T008；TECH_SPEC.md §6.4
  - 🔧 做什么：`crypto/dpapi.rs` 暴露 `encrypt(plain: &[u8]) -> Vec<u8>` / `decrypt(cipher: &[u8]) -> Vec<u8>`；附加熵固定 "WhisperKey-v1-Salt"；以 base64 字符串接口包装一层 `encrypt_str`/`decrypt_str`
  - 📤 产出：crypto/dpapi.rs
  - ✅ 验收：roundtrip 单测通过；密文非空且 != 明文
  - 🔗 依赖：T008

- [x] **T010 [config] config.json schema 类型**
  - 📥 输入：T009；TECH_SPEC.md §6.1
  - 🔧 做什么：`config/schema.rs` 定义全部 struct（serde derive）；提供 `Default` 实现 = 默认配置
  - 📤 产出：config/schema.rs
  - ✅ 验收：`serde_json::to_string_pretty(&Config::default())` 与 §6.1 示例字段一致
  - 🔗 依赖：T009

- [x] **T011 [config] 加载/保存/迁移**
  - 📥 输入：T010
  - 🔧 做什么：`config/persist.rs` 实现 `load() -> Config`（不存在则写默认；version 不符则迁移并备份原文件至 `config.json.bak.<ts>`）；`save(c: &Config)`（写到 `.tmp` 再原子 rename）；启用 `parking_lot::RwLock` 提供 `ConfigStore` 单例
  - 📤 产出：config/mod.rs、config/persist.rs
  - ✅ 验收：单测：写入→读取一致；坏 JSON → 自动重建；并发 100 次写无文件损坏
  - 🔗 依赖：T010

- [x] **T012 [config] API Key 加密存取适配器**
  - 📥 输入：T009, T011
  - 🔧 做什么：在 ConfigStore 上层提供 `set_provider_key(provider, key)`（自动 DPAPI 加密 + base64 + 持久化）/ `get_provider_key(provider) -> String`（解密返回明文，仅内存中流转）
  - 📤 产出：config/secrets.rs
  - ✅ 验收：set→save→reload→get roundtrip
  - 🔗 依赖：T009, T011

---

## Phase 2：录音与快捷键

- [x] **T013 [hotkey] HotkeyConfig 数据模型 + 转换**
  - 📥 输入：T011
  - 🔧 做什么：`hotkey/mod.rs` 定义 `HotkeyConfig { modifiers: Vec<Modifier>, key: KeyCode }`；实现 `to_winapi() -> (HOT_KEY_MODIFIERS, u32 vk)`；实现 `from_string("Ctrl+Shift+Space")` 解析与 `Display`
  - 📤 产出：hotkey/mod.rs
  - ✅ 验收：8 组测试覆盖正常/异常字符串
  - 🔗 依赖：T011

- [x] **T014 [hotkey] RegisterHotKey + 消息泵线程**
  - 📥 输入：T013；TECH_SPEC.md §4.1
  - 🔧 做什么：`hotkey/registrar.rs` 实现 `start(cfg) -> HotkeyHandle`：独立线程跑 `RegisterHotKey + GetMessage`；通过 `tokio::sync::broadcast` 广播 `HotkeyEvent::{Triggered, RegisterFailed}`；handle drop 时通过 PostThreadMessage(WM_QUIT) 优雅退出
  - 📤 产出：hotkey/registrar.rs
  - ✅ 验收：手动测试：Ctrl+Shift+Space 在记事本顶部按下时控制台打印 Triggered 事件；快捷键被占用时返回 RegisterFailed
  - 🔗 依赖：T013

- [x] **T015 [audio] cpal 录音器**
  - 📥 输入：T011
  - 🔧 做什么：`audio/recorder.rs` 实现 `Recorder` 结构体；start() 启动 16kHz/16bit/mono 输入流，把 frame 推入环形缓冲；stop() 返回 `Vec<i16> samples`；提供 `current_rms()` 用于电平
  - 📤 产出：audio/recorder.rs
  - ✅ 验收：录音 3 秒 → samples.len() ≈ 48000；RMS 对静音 ≈ 0、对说话 > 500
  - 🔗 依赖：T011

- [x] **T016 [audio] WAV 编码**
  - 📥 输入：T015
  - 🔧 做什么：`audio/encoder.rs` 用 hound 把 i16 samples 编码为 WAV bytes（16kHz/16bit/mono header）
  - 📤 产出：audio/encoder.rs
  - ✅ 验收：生成的 WAV 用 Windows 媒体播放器能播放
  - 🔗 依赖：T015

- [x] **T017 [audio] 静音自动停止与最大时长**
  - 📥 输入：T015；config.audio.silenceTimeoutMs / maxDurationSec
  - 🔧 做什么：在 Recorder 内开启监控任务：若启用静音停止，连续 N 毫秒 RMS<阈值 则发出 `StopReason::Silence`；总时长达到 maxDurationSec 发出 `StopReason::MaxDuration`
  - 📤 产出：audio/recorder.rs 增量
  - ✅ 验收：单测模拟样本流，覆盖 Silence/MaxDuration/Manual 三种停止原因
  - 🔗 依赖：T015

---

## Phase 3：ASR 与 LLM 集成

- [x] **T018 [asr] AsrProvider trait + Registry**
  - 📥 输入：T012
  - 🔧 做什么：`asr/trait.rs` 定义 `#[async_trait] trait AsrProvider { async fn transcribe(req) -> AppResult<AsrResponse>; fn name(&self) -> &'static str; }`；`asr/mod.rs` 提供 `AsrRegistry::get(name) -> Arc<dyn AsrProvider>` 通过名字派发
  - 📤 产出：asr/trait.rs、asr/mod.rs
  - ✅ 验收：编译通过；mock provider 可注册并取出
  - 🔗 依赖：T012

- [x] **T019 [asr] OpenAI Whisper 实现**
  - 📥 输入：T018；TECH_SPEC.md §5.1
  - 🔧 做什么：`asr/openai.rs` 实现 OpenAiAsr：reqwest multipart；超时 10s；401/429/500 错误映射 AppError；重试 1 次（仅 5xx/网络）
  - 📤 产出：asr/openai.rs
  - ✅ 验收：用 wiremock 起本地 mock，转写 200 OK 返回正确文本；401 返回 E_ASR_AUTH
  - 🔗 依赖：T018

- [x] **T020 [asr] 讯飞 + 火山实现**
  - 📥 输入：T018；TECH_SPEC.md §5.9-5.10
  - 🔧 做什么：`asr/xfyun.rs`、`asr/volcengine.rs`；签名算法封装到 util
  - 📤 产出：xfyun.rs、volcengine.rs
  - ✅ 验收：mock 返回响应解析正确
  - 🔗 依赖：T018

- [x] **T021 [asr] 官方免费 ASR 中转**
  - 📥 输入：T018；TECH_SPEC.md §5.11
  - 🔧 做什么：`asr/official.rs` 走 `https://api.whisperkey.app/v1/transcribe`，自动带设备指纹 header
  - 📤 产出：asr/official.rs
  - ✅ 验收：mock 通；429 quota 返回 E_ASR_QUOTA
  - 🔗 依赖：T018, T035（fingerprint，可后填）

- [x] **T022 [llm] LlmProvider trait + Prompts**
  - 📥 输入：T012；TECH_SPEC.md §5.12
  - 🔧 做什么：`llm/trait.rs` 定义 trait（输入 mode + raw_text，输出 String）；`llm/prompts.rs` 加载 resources/prompts 三个 .md 文件并 `{{TEXT}}` 模板替换
  - 📤 产出：trait.rs、prompts.rs、resources/prompts/raw.md / polish.md / markdown.md
  - ✅ 验收：渲染后字符串包含原文且不含 `{{TEXT}}`
  - 🔗 依赖：T012

- [x] **T023 [llm] OpenAI 兼容协议实现（OpenAI/DeepSeek/Qwen/Doubao）**
  - 📥 输入：T022；TECH_SPEC.md §5.2/5.4/5.5/5.7
  - 🔧 做什么：实现 `OpenAiCompatibleLlm { base_url, api_key, model }`；DeepSeek/Qwen/Doubao 复用此实现仅传不同 base_url + model
  - 📤 产出：llm/openai.rs（含工厂方法）、llm/deepseek.rs、llm/qwen.rs、llm/doubao.rs（薄包装）
  - ✅ 验收：4 套 mock 各通过
  - 🔗 依赖：T022

- [x] **T024 [llm] Anthropic 实现**
  - 📥 输入：T022；TECH_SPEC.md §5.3
  - 🔧 做什么：`llm/anthropic.rs`；header `x-api-key`、`anthropic-version`；响应字段 `content[0].text`
  - 📤 产出：anthropic.rs
  - ✅ 验收：mock 通
  - 🔗 依赖：T022

- [x] **T025 [llm] 文心 + Gemini 实现**
  - 📥 输入：T022；§5.6/5.8
  - 🔧 做什么：`llm/ernie.rs`（含 access_token 缓存 25min）；`llm/gemini.rs`
  - 📤 产出：ernie.rs、gemini.rs
  - ✅ 验收：mock 通
  - 🔗 依赖：T022

- [x] **T026 [llm] LlmRegistry**
  - 📥 输入：T023, T024, T025
  - 🔧 做什么：`llm/mod.rs` 实现 `LlmRegistry::resolve(provider, model) -> Arc<dyn LlmProvider>`；从 ConfigStore 读取 key + base_url 装配
  - 📤 产出：llm/mod.rs
  - ✅ 验收：根据 provider 字符串能取出正确实现
  - 🔗 依赖：T023, T024, T025

- [x] **T027 [llm] 模式 B 输出长度二次校验**
  - 📥 输入：T026
  - 🔧 做什么：在 polish 模式调用后校验 `output.chars().count() <= raw.chars().count() * 1.2 + 20`；超出则降级返回 raw_text 并 warn 日志
  - 📤 产出：llm/mod.rs 增量
  - ✅ 验收：单测：超长输出被拦截
  - 🔗 依赖：T026

---

## Phase 4：文本注入与全局编排

- [x] **T028 [inject] 剪贴板备份/恢复**
  - 📥 输入：T005
  - 🔧 做什么：`inject/clipboard.rs` 用 windows crate OpenClipboard/GetClipboardData/SetClipboardData 实现 backup() 返回 Vec<u8> 或 Unicode 字符串、restore()、set_text()
  - 📤 产出：inject/clipboard.rs
  - ✅ 验收：roundtrip 文本与图片（图片仅备份格式句柄）；不阻塞主线程
  - 🔗 依赖：T005

- [x] **T029 [inject] SendInput Ctrl+V**
  - 📥 输入：T028
  - 🔧 做什么：`inject/send_input.rs` 实现 `send_ctrl_v()`：构造 4 个 INPUT 结构体（Ctrl down, V down, V up, Ctrl up），KEYEVENTF_SCANCODE
  - 📤 产出：inject/send_input.rs
  - ✅ 验收：手动测试在记事本可粘贴
  - 🔗 依赖：T028

- [x] **T030 [inject] SendInput Unicode 逐字符回退**
  - 📥 输入：T029
  - 🔧 做什么：实现 `send_unicode(text)`：遍历 utf-16 code units，KEYEVENTF_UNICODE 输入；处理代理对
  - 📤 产出：inject/send_input.rs 增量
  - ✅ 验收：在记事本能输入"你好 Hello 😀"（emoji 用代理对）
  - 🔗 依赖：T029

- [x] **T031 [inject] 主流程 + 自动回退**
  - 📥 输入：T028, T029, T030；TECH_SPEC.md §4.2
  - 🔧 做什么：`inject/mod.rs::inject(text)` 实现剪贴板序列号检测 + 三级回退；返回 InjectResult
  - 📤 产出：inject/mod.rs
  - ✅ 验收：mock 不同失败场景，回退路径正确
  - 🔗 依赖：T028, T029, T030

- [x] **T032 [util] 当前焦点应用获取**
  - 📥 输入：T005
  - 🔧 做什么：`util/focus_app.rs::current_focus_app() -> { exe_name, window_title }`：GetForegroundWindow → GetWindowThreadProcessId → QueryFullProcessImageNameW
  - 📤 产出：util/focus_app.rs
  - ✅ 验收：聚焦记事本时返回 notepad.exe
  - 🔗 依赖：T005

- [x] **T033 [indicator] 录音悬浮窗 Vue 视图**
  - 📥 输入：T005
  - 🔧 做什么：`src/views/Indicator/RecordIndicator.vue` 实现 200×40 圆角胶囊，左红点呼吸 + 12 根波形条 + 右模式徽标 + spinner；通过 invoke 订阅事件 `indicator://state` `indicator://level`
  - 📤 产出：RecordIndicator.vue + 路由
  - ✅ 验收：开发模式手动 emit 事件可看到变化
  - 🔗 依赖：T005

- [x] **T034 [indicator] Rust 端窗口控制**
  - 📥 输入：T033
  - 🔧 做什么：`indicator/mod.rs` 提供 show(mode) / set_state / update_level / hide；首次 show 时通过 GetWindowLongPtrW + SetWindowLongPtrW 添加 WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT；emit Tauri events 给前端
  - 📤 产出：indicator/mod.rs
  - ✅ 验收：show/hide 不抢焦点；其他应用键盘输入不丢
  - 🔗 依赖：T033

- [x] **T035 [license] 机器指纹**
  - 📥 输入：T009
  - 🔧 做什么：`license/fingerprint.rs::device_fingerprint() -> String`：读注册表 `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` + WMI Win32_BaseBoard.SerialNumber 前 8 位 → SHA256 → base64
  - 📤 产出：fingerprint.rs
  - ✅ 验收：同机两次返回相同值；不同机器不同
  - 🔗 依赖：T009

- [x] **T036 [license] 验签器**
  - 📥 输入：T035；resources/public_key.pem（先放占位 PEM，后续替换）
  - 🔧 做什么：`license/verifier.rs::verify(license: &License) -> AppResult<()>`：ring RSA-PSS-2048 验签 + 比对设备指纹 + 比对 expiresAt
  - 📤 产出：verifier.rs
  - ✅ 验收：单测：合法 license/篡改 license/错误指纹/过期 license 各返回正确结果
  - 🔗 依赖：T035

- [x] **T037 [license] 激活器**
  - 📥 输入：T036
  - 🔧 做什么：`license/activator.rs::activate(code: &str) -> AppResult<License>`：POST `https://api.whisperkey.app/v1/activate` { code, fingerprint } → 返回 License JSON → 本地校验 → 写入 license.dat
  - 📤 产出：activator.rs
  - ✅ 验收：mock 服务器返回有效 license 时落盘成功；无效激活码返回 E_LICENSE_INVALID
  - 🔗 依赖：T036

- [x] **T038 [license] LicenseStore 单例**
  - 📥 输入：T036, T037
  - 🔧 做什么：`license/mod.rs` 启动时加载 license.dat 验签；提供 `is_unlocked(product) -> bool`；变更后广播事件
  - 📤 产出：license/mod.rs
  - ✅ 验收：is_unlocked 返回正确
  - 🔗 依赖：T036, T037

- [x] **T039 [history] SQLite 初始化与迁移**
  - 📥 输入：T008；TECH_SPEC.md §6.2
  - 🔧 做什么：`history/db.rs` 用 rusqlite 打开 history.db；执行 migrations
  - 📤 产出：db.rs、migrations.rs
  - ✅ 验收：表结构 `pragma table_info(history)` 与 §6.2 完全一致
  - 🔗 依赖：T008

- [x] **T040 [history] 增删查清接口**
  - 📥 输入：T039
  - 🔧 做什么：实现 add / list(filter, page, page_size=50) / delete / clear / purge_old(now - 7d)
  - 📤 产出：history/mod.rs
  - ✅ 验收：单测覆盖 5 个接口；purge_old 删旧留新
  - 🔗 依赖：T039

- [x] **T041 [pipeline] 状态机骨架**
  - 📥 输入：T014, T015, T018, T026, T031, T034, T040；TECH_SPEC.md §7
  - 🔧 做什么：`pipeline/state_machine.rs` 实现 PipelineState 枚举 + Tokio task 循环；事件输入（HotkeyTriggered/SilenceTimeout/MaxDurationReached/AsrDone/LlmDone/InjectDone）；状态变化通过 watch 广播
  - 📤 产出：state_machine.rs
  - ✅ 验收：单测：模拟事件序列覆盖完整成功路径与每条失败路径
  - 🔗 依赖：T014, T015, T018, T026, T031, T034, T040

- [x] **T042 [pipeline] 运行编排器**
  - 📥 输入：T041
  - 🔧 做什么：`pipeline/mod.rs::Pipeline::run()`：订阅 hotkey 事件；按状态机驱动 audio/asr/llm/inject/history/indicator 全链路；模式 B/C 调用前 license 校验，未解锁返回友好错误
  - 📤 产出：pipeline/mod.rs
  - ✅ 验收：手动 e2e：按下快捷键 → 录音 3s → 再按 → 文本注入到记事本 + 历史落库
  - 🔗 依赖：T041

---

## Phase 5：UI 与设置界面

- [x] **T043 [ipc] Tauri command 全集**
  - 📥 输入：T011, T038, T040, T042
  - 🔧 做什么：在 `ipc/` 下声明全部 #[tauri::command]：cmd_config_get/set、cmd_config_set_provider_key、cmd_provider_test_connection、cmd_recording_toggle、cmd_recording_get_state、cmd_history_list/delete/clear、cmd_license_status/activate/unbind、cmd_app_quit、cmd_app_open_logs_folder
  - 📤 产出：ipc/*.rs
  - ✅ 验收：tauri generate-types 后前端可看到全部接口
  - 🔗 依赖：T011, T038, T040, T042

- [x] **T044 [ui] 路由与外壳 App.vue**
  - 📥 输入：T043
  - 🔧 做什么：vue-router 定义 /settings/general /settings/hotkey /settings/providers /settings/activation /settings/history /settings/about / /indicator /activate；App.vue 主窗口左侧导航 + 右侧 router-view；indicator/activate 路由独立窗口
  - 📤 产出：router/index.ts、App.vue
  - ✅ 验收：导航跳转正常；indicator 路由不显示在主窗口
  - 🔗 依赖：T043

- [x] **T045 [ui] 通用设置页（General）**
  - 📥 输入：T044
  - 🔧 做什么：开关组件：开机自启 / 启动最小化 / 检查更新；下拉：默认模式 / 录音最大时长；静音停止开关 + 时长
  - 📤 产出：views/Settings/General.vue
  - ✅ 验收：勾选 → 保存到 config 并立即生效
  - 🔗 依赖：T044

- [x] **T046 [ui] 快捷键设置页**
  - 📥 输入：T044
  - 🔧 做什么：`HotkeyInput.vue` 组件：聚焦后捕获按键合成显示；保存时调 hotkey 模块重新 register；冲突时显示红字提示
  - 📤 产出：HotkeyInput.vue + views/Settings/Hotkey.vue
  - ✅ 验收：改键并测试触发新键有效
  - 🔗 依赖：T044

- [x] **T047 [ui] 模式 & 服务商页**
  - 📥 输入：T044, T038
  - 🔧 做什么：上半 ASR 区 + 下半 LLM 区；ProviderCard 组件（logo、状态、配置按钮、测试按钮）；模式分配 3 个下拉（原话/优化/Markdown）；模式 B/C 未解锁时下拉旁显示锁图标和"购买激活"链接
  - 📤 产出：ProviderCard.vue + Providers.vue
  - ✅ 验收：填入 OpenAI Key + 测试 → 显示绿色 OK
  - 🔗 依赖：T044, T038

- [x] **T048 [ui] 激活页 + 激活弹窗**
  - 📥 输入：T044, T037
  - 🔧 做什么：Activation.vue 显示当前状态 + 输入激活码；ActivateDialog.vue 单独窗口路由 /activate（用于托盘"激活"入口）
  - 📤 产出：Activation.vue、ActivateDialog.vue
  - ✅ 验收：成功激活后 license 状态实时更新
  - 🔗 依赖：T044, T037

- [x] **T049 [ui] 历史记录页**
  - 📥 输入：T044, T040
  - 🔧 做什么：搜索框 + 模式过滤 + 日期过滤；分页 50 条；HistoryItem 组件（时间/模式/摘要/复制/重新注入/删除按钮）；底部"立即清空"二次确认
  - 📤 产出：HistoryItem.vue + History.vue
  - ✅ 验收：测试增删查；重新注入按钮触发 inject 模块
  - 🔗 依赖：T044, T040

- [x] **T050 [ui] 关于页**
  - 📥 输入：T044
  - 🔧 做什么：版本号、检查更新按钮、用户协议、隐私政策、官网链接
  - 📤 产出：About.vue
  - ✅ 验收：检查更新调用更新模块
  - 🔗 依赖：T044

- [x] **T051 [ui] 主题与多语言基础**
  - 📥 输入：T044
  - 🔧 做什么：Naive UI 主题 provider 跟随系统；vue-i18n 中英文 messages，本期默认中文，英文留 placeholder
  - 📤 产出：i18n/zh-CN.ts、i18n/en.ts、styles/global.scss
  - ✅ 验收：切换系统暗色模式 UI 跟随
  - 🔗 依赖：T044

- [x] **T052 [tray] 系统托盘**
  - 📥 输入：T043；TECH_SPEC.md §3.11
  - 🔧 做什么：`tray/mod.rs` 创建 TrayIcon；菜单项：模式切换（3 个 RadioItem）+ 设置 + 历史 + 暂停/启用快捷键 + 开机自启 + 关于 + 退出；模式 B/C 未解锁灰色 + 跳转激活
  - 📤 产出：tray/mod.rs
  - ✅ 验收：右键菜单完整可用；切换模式后 indicator 徽标更新
  - 🔗 依赖：T043

- [x] **T053 [util] 单实例**
  - 📥 输入：T005
  - 🔧 做什么：tauri-plugin-single-instance 集成；第二次启动时聚焦已运行实例并退出
  - 📤 产出：main.rs 增量
  - ✅ 验收：双开测试只剩一个进程
  - 🔗 依赖：T005

---

## Phase 6：许可证商业化与品质打磨

- [x] **T054 [license] 联网续签静默任务**
  - 📥 输入：T038
  - 🔧 做什么：每 30 天后台任务调用 `/v1/license/refresh`；失败保留旧 license，记 warn
  - 📤 产出：license/refresh.rs
  - ✅ 验收：mock 测试通过；离线时不触发错误
  - 🔗 依赖：T038

- [x] **T055 [license] 模式 B/C 准入门控**
  - 📥 输入：T038, T042
  - 🔧 做什么：在 pipeline 调用 LLM 前判断；UI 设置面板 + 托盘菜单按解锁状态显隐；emit `license://changed` 事件给前端
  - 📤 产出：pipeline 增量、tray 增量
  - ✅ 验收：未激活时模式 B 调用直接返回 E_LICENSE_INVALID + 弹激活引导
  - 🔗 依赖：T038, T042

- [x] **T056 [config] 自动启动**
  - 📥 输入：T011
  - 🔧 做什么：tauri-plugin-autostart 集成；General 设置开关绑定
  - 📤 产出：main.rs 增量
  - ✅ 验收：勾选后注册表/启动文件夹生效，重启 Windows 后自动启动
  - 🔗 依赖：T011

- [x] **T057 [updater] 自动更新接入**
  - 📥 输入：T005；TECH_SPEC.md §10.4
  - 🔧 做什么：tauri-plugin-updater 配置 endpoint + minisign 公钥；启动 30s 后静默检查；通过托盘气泡通知
  - 📤 产出：tauri.conf.json 增量、updater/mod.rs
  - ✅ 验收：本地起 mock 清单服务器测试更新流程
  - 🔗 依赖：T005

- [ ] **T058 [perf] 启动性能优化**
  - 📥 输入：所有
  - 🔧 做什么：lazy load LLM/ASR registry；前端路由按需 import；移除未使用依赖
  - 📤 产出：调整若干文件
  - ✅ 验收：冷启动到托盘可见 ≤ 1.5s（SSD）
  - 🔗 依赖：所有

- [x] **T059 [robust] 全局未捕获异常处理**
  - 📥 输入：T007
  - 🔧 做什么：std::panic::set_hook 写日志 + 弹提示 + 写 crash dump 到 logs/crash/
  - 📤 产出：main.rs 增量
  - ✅ 验收：人为 panic!() 一次产生 crash dump 文件
  - 🔗 依赖：T007

- [ ] **T060 [test] 集成测试**
  - 📥 输入：所有；TECH_SPEC.md §11.2
  - 🔧 做什么：在 `tests/` 下实现 IT-01..IT-10；wiremock 启动 mock LLM/ASR 服务器
  - 📤 产出：tests/it_*.rs
  - ✅ 验收：`cargo test --test 'it_*'` 全绿
  - 🔗 依赖：所有

---

## Phase 7：打包与分发

- [ ] **T061 [build] 安装包配置**
  - 📥 输入：所有；TECH_SPEC.md §10
  - 🔧 做什么：tauri.conf.json bundle.windows.nsis 完善（installMode=perUser、license file、自定义图标、桌面快捷方式可选）；中文安装界面
  - 📤 产出：tauri.conf.json + nsis 模板
  - ✅ 验收：`pnpm tauri build` 产出 ≤25MB 单 .exe；安装-启动-卸载流程通畅
  - 🔗 依赖：所有

- [ ] **T062 [build] CI 流水线**
  - 📥 输入：T061
  - 🔧 做什么：`.github/workflows/release.yml`：windows-latest、x64 + ARM64 matrix；release 触发上传 .exe
  - 📤 产出：release.yml
  - ✅ 验收：tag push 后产物上传成功
  - 🔗 依赖：T061

- [ ] **T063 [build] 代码签名脚本（占位）**
  - 📥 输入：T061
  - 🔧 做什么：`scripts/sign.ps1` 接受证书路径，调用 signtool 对 .exe 签名；CI 中预留 secrets
  - 📤 产出：sign.ps1
  - ✅ 验收：脚本干跑通过；待真实证书替换
  - 🔗 依赖：T061

- [ ] **T064 [doc] README + 用户文档**
  - 📥 输入：所有
  - 🔧 做什么：README 含安装、首次使用、快捷键、模式介绍、激活、FAQ、隐私政策摘要
  - 📤 产出：README.md
  - ✅ 验收：新用户照文档可在 5 分钟内完成首次使用
  - 🔗 依赖：所有

- [ ] **T065 [release] 版本发布检查清单**
  - 📥 输入：所有
  - 🔧 做什么：CHANGELOG.md 1.0.0；执行 PRD §11 V1.0 验收手测；打 tag v1.0.0
  - 📤 产出：CHANGELOG.md + git tag
  - ✅ 验收：tag 推送 → CI 产物可下载安装运行
  - 🔗 依赖：T061..T064