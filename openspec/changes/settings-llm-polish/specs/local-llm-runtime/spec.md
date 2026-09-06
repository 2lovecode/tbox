## ADDED Requirements

### Requirement: Custom GGUF Download
系统 SHALL 允许用户通过填写下载 URL（及可选显示名）将任意 GGUF 下载到应用模型目录。下载 MUST 展示进度且可取消；只有完整且通过校验的文件方可标记为已安装并可启用。失败或取消后 MUST NOT 标记为已安装。

#### Scenario: Custom URL download succeeds
- **WHEN** 用户提交合法的 GGUF URL 且下载与校验成功
- **THEN** 该模型出现在本地模型列表中并显示为已安装，可选为本地启用模型

#### Scenario: Custom URL download fails
- **WHEN** URL 不可达、内容非完整 GGUF 或用户取消
- **THEN** 界面提示失败或已取消，且不得将该条目标为已安装

### Requirement: GPU Acceleration Preference
当主机存在可用 GPU 加速路径时，嵌入式推理引擎 SHALL 优先使用 GPU，而不是默认仅 CPU。优先级 MUST 为：macOS Apple Silicon 上 Metal；Windows / Linux 上优先 CUDA，若不可用则 Vulkan，再回退 CPU。系统 SHALL 向前端暴露当前实际使用的加速后端标识（如 metal / cuda / vulkan / cpu），以便设置或引擎状态可见。

#### Scenario: CUDA preferred on Windows or Linux when available
- **WHEN** 用户在 Windows 或 Linux 上使用已下载的本地模型，且运行时 CUDA 可用
- **THEN** 嵌入式引擎以 CUDA 加速加载/推理，状态显示后端为 cuda（或等价标识）

#### Scenario: Fallback when GPU unavailable
- **WHEN** 无可用 GPU 后端
- **THEN** 引擎回退 CPU 推理，状态显示为 cpu，主窗口仍可用

### Requirement: Provider Selection Secondary Dialog
在新建 LLM 配置流程中，用户选择提供方时，系统 SHALL 打开独立的提供方选择对话框（与编辑表单对话框分离）。该对话框 MUST 以可筛选的表格或等价网格展示全部预设（含图标或色块兜底、显示名、默认端点摘要），MUST 包含「自定义端点」选项。选中后 MUST 关闭选择对话框并回填编辑表单。编辑已有配置时提供方 MUST 仍保持只读。

#### Scenario: Open provider picker while creating
- **WHEN** 用户在新建配置中点击选择提供方
- **THEN** 打开独立提供方选择对话框，列表可筛选且含自定义端点

#### Scenario: Pick provider with icon
- **WHEN** 提供方选择对话框展示某一非 OAuth 预设
- **THEN** 该项显示图标（或首字母色块兜底）与名称，用户可点选并回到编辑表单

### Requirement: Compact Settings Page Chrome
设置页主内容区 MUST NOT 再展示与左侧导航重复的大号「设置」页眉块（含装饰性滑块图标与「管理应用偏好与 LLM 提供方」类副标题）。页面 SHALL 以左菜单 + 右内容为主视觉，可保留轻量返回控件。

#### Scenario: Settings page without redundant header
- **WHEN** 用户打开设置页 LLM 分区
- **THEN** 右侧不以大号「设置」标题块占据内容顶部，左菜单与 LLM 列表面板为主要结构

## MODIFIED Requirements

### Requirement: Curated Model Download
系统 SHALL 在设置页提供精选模型目录（约 1～2 个更新一代小 Instruct GGUF，例如 Qwen3 约 0.6B / 1.7B 量级 Q4_K_M，并标明推荐项与 Agent 适用性）。用户 MUST 能下载、看到进度、取消下载；只有校验完整的文件才可被标记为已安装并启用。下载位置 MUST 在应用数据目录（与现有 `~/.toolbox` 一类路径）。推荐目录 MUST NOT 再以 Qwen2.5 0.5B/1.5B 作为主推项（磁盘上已有旧文件仍可被发现，但不作为精选主推）。

#### Scenario: Download then enable
- **WHEN** 用户从设置页下载推荐模型且下载成功
- **THEN** 该模型显示为已安装，可被选为本地启用模型

#### Scenario: Failed download not installed
- **WHEN** 下载失败或用户取消
- **THEN** 该模型不得显示为已安装；不得留下可被启用的半截文件状态

#### Scenario: Offline chat after download
- **WHEN** 推荐模型已启用且嵌入式引擎可运行，用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM）

#### Scenario: Catalog prefers newer small Instruct
- **WHEN** 用户查看精选本地模型目录
- **THEN** 列表主推更新一代小 Instruct（约 ≤2B），而非 Qwen2.5 0.5B/1.5B 作为推荐项

### Requirement: Sidecar Lifecycle
系统 SHALL 在选中 `local` 且已有启用模型时，通过内置的进程内嵌入式推理引擎（成熟 Rust 推理库编译进应用，如 llama.cpp 绑定）加载该模型并服务对话，退出应用时释放引擎资源（MUST 包含释放模型权重，即 `llama_free_model`）。引擎 MUST NOT 依赖任何外部进程、外部端口或用户手动安装的运行时。推理 MUST 在专用线程执行，引擎崩溃或模型加载失败 MUST NOT 导致主窗口退出。释放 MUST 在进程退出前的应用级清理阶段完成，早于 llama.cpp/ggml 的 C++ 静态析构。当 GPU 加速可用时，加载与推理 MUST 遵循「GPU Acceleration Preference」；仅在无 GPU 后端时以 CPU 为可接受路径。

#### Scenario: Bind loopback only
- **WHEN** 嵌入式引擎因本地对话被启动
- **THEN** 不监听任何网络端口，推理完全在进程内完成

#### Scenario: Engine crash surfaces error
- **WHEN** 引擎线程 panic 或模型加载失败
- **THEN** 对话界面提示本地引擎不可用，设置中可请求重试，主窗口仍保持可用

#### Scenario: Offline chat with embedded engine
- **WHEN** 推荐模型已下载且用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM，也不依赖本机 Ollama）

#### Scenario: Clean exit after model load
- **WHEN** 用户在本地对话（已加载模型）后退出应用
- **THEN** 引擎释放模型权重与上下文，进程正常退出，不因 ggml Metal residency-set 断言（`GGML_ASSERT([rsets->data count] == 0)`）崩溃

### Requirement: Provider Presets Aligned With CC Switch
系统 SHALL 在设置中提供与 CC Switch（Claude 预设目录）对齐的提供商预设列表，并额外包含 TBox 自有的 `local`、`ollama` 与 `custom`。预设 MUST 包含显示名、默认 base URL、默认模型 id、默认协议，以及用于 UI 的图标标识与可选主题色；列表 MUST 注明所依据的上游快照标识（日期或 commit），并在本变更中刷新至实现时的最新上游快照。标记为需要 OAuth 的预设 MUST 在 UI 中可见且标明暂不支持，MUST NOT 被保存为可发起 Agent 推理的有效后端。

#### Scenario: Select non-OAuth preset
- **WHEN** 用户选择某一非 OAuth 预设（例如 DeepSeek 或智谱）
- **THEN** 表单填入该预设的默认 base URL、模型与协议，用户仍可修改后保存

#### Scenario: OAuth preset not usable
- **WHEN** 用户选择标记为需要 OAuth 的预设（例如 GitHub Copilot）
- **THEN** 界面标明暂不支持 OAuth，且不得将该配置作为可用 LLM 后端用于对话

#### Scenario: Local and Ollama remain available
- **WHEN** 用户打开提供商列表
- **THEN** 列表中包含 `local`（内置引擎）与 `ollama`，与 CC Switch 对齐预设并存

#### Scenario: Preset shows icon metadata
- **WHEN** 前端渲染提供方选择列表中的某一预设
- **THEN** 可获得该预设的图标标识（及可选颜色）用于展示；缺失时使用兜底展示
