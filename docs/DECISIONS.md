# Lyreniq 决策记录

## D001 — 软件命名与平台范围（2026-11-24）

- 决策：软件名为 Lyreniq；Universal Tuner 保留为需求项目名称和任务书文件名。第一里程碑仅为 Windows 桌面 MVP。
- 原因：用户本轮明确命名，任务书要求按小任务逐轮开发。
- 替代方案：沿用 Universal Tuner 作为软件显示名；与用户命名不符，不采用。
- 影响：README、规则、状态和交接使用 Lyreniq；未来包名、应用标识在相应工程任务确定，T00 不生成应用配置。
- 验证：本轮检查基础文档名称、T00 当前任务与 T01 唯一下一任务一致性。

## D002 — 主任务书位置与原件保留（2026-11-24）

- 决策：根目录任务书作为冻结的用户输入保留，逐字节复制到 docs，后续以 `docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 为维护入口。
- 原因：满足 T00 将任务书放 docs 和保留原始文件两项要求，同时避免两个可变主文档。
- 替代方案：移动或改写原件；本轮采用保留原件，便于追溯。
- 影响：T00 两份 SHA-256 应相同；后续经授权的主文档修订需记录决策，根目录冻结原件不跟随更新。
- 原件基准：91,308 字节，1,213 行；SHA-256 `C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF`。
- 验证：提交前比较两份文件 SHA-256，并核实 Git 中保留原始字节。

## D003 — Git 初始化与远端状态（2026-11-24）

- 决策：在当前项目目录建立本地 main 分支；使用已配置的 Git 提交身份，不修改全局 Git 配置。目标仓库未确定前不配置猜测的 remote。
- 原因：起点没有 Git；用户授权推送，但没有提供目标 URL。gh 不可用，连接器按两个项目名称均未检索到匹配仓库。
- 替代方案：自行猜测仓库或新建公开仓库；没有足够依据，不采用。
- 影响：T00 可完成本地验收和提交，GitHub 交付单独记为待配置；取得 URL 后检查远端历史、认证和权限再 push 并核实。
- 验证：`git init -b main` 成功；`git remote -v` 无输出；`gh auth status` 返回命令不可用。

## D004 — 文本与忽略规则（2026-11-24）

- 决策：新增文本默认 UTF-8/LF，PowerShell 脚本预设 CRLF；两份原始任务书在 Git 中不进行换行转换。仅这两份任务书允许原有 Markdown 行末空格。
- 原因：本机 Git 的 core.autocrlf=true；原任务书第 3–5 行的两个空格是 Markdown 硬换行，保留原始字节及排版。
- 替代方案：清理原任务书或对所有 Markdown 放宽空白检查；均不采用。
- 影响：.gitattributes 对两份任务书设置 `-text whitespace=-blank-at-eol`；其他文件维持 Git 空白检查。缓存、构建、临时音频、用户数据、密钥和 IDE 私有配置忽略；明确保留测试夹具音频和锁文件。
- 验证：提交前运行 diff 空白检查和代表性 git check-ignore 检查。

## D005 — 用户授权的任务推进与共享安装（2026-11-24）

- 决策：用户在 T00 后明确允许额度可用时自主推进下一任务，仍保持每个任务的复验、独立验收、交接、commit/push 边界；用户已同意安装 T01 必需组件，并允许共享系统/用户级环境。
- 原因：后续直接用户指令优先于原任务书每轮结束即停止的默认规则。
- 影响：无需每项重复询问是否开始；遇到需要用户操作或不可自动处理的问题时说明。无法读取实际额度余额，不承诺额度消耗或耗尽时点。
- 验证：本轮先复验 T00，再进入 T01；共享工具链实测见 ENVIRONMENT。

## D006 — T01 版本与配置基线（2026-11-24）

- 决策：复用 Node 24.21.0/WebView2 157，安装共享 Rust 1.100.0、pnpm 11.25.0、MSVC 2022 与 SDK；具体版本见 ENVIRONMENT。直接依赖精确约束并提交两份锁文件。
- 原因：官方稳定版本查询后在本机验证，避免使用 Codex 内置 pnpm 的不同 Node 版本。pnpm 11 非认证配置统一放 workspace YAML，store 在项目 .local，仅批准 esbuild 构建脚本。
- 替代方案：强制把 SDK/全部工具迁入项目，或忽略 engine/build-script 错误；均不采用。
- 影响：项目可通过声明的共享系统前置条件重建；缓存和二进制工具不提交。MSVC 安装器曾返回 3010，安装后工具已可用且 vswhere 无重启标记，未自动重启。
- 验证：版本输出、前端 build、frozen-lockfile 安装、cargo check --workspace --locked 已通过；原生窗口结果另见 TESTING。

## D007 — 最小原生窗口边界（2026-11-24）

- 决策：T01 只建一个桌面 crate 与 React 页面，通过单个只读 runtime_label 命令返回真实版本/平台；不创建空 DSP/audio crate，不做调音 UI。
- 原因：先验证真实 Windows 原生链路。桌面调用封装于 services/desktop.ts，生产 CSP 保持本地资源限制，开发 CSP 额外允许本机 Vite/React Refresh，capability 不开放插件权限。
- 影响：应用标识暂用 com.lyreniq.desktop（开发标识，不表示拥有该域名）；bundle 暂关闭，签名/分发标识在发布任务复核。图标来自本仓库自有 L 字 SVG，由 Tauri CLI 生成 Windows ICO/PNG；不是最终品牌设计。
- 验证：构建和原生 IPC smoke test 见 TESTING；其他平台、音频、UI 设计系统、安装包均未实现。

## D008 — T02 自动化与 Windows 测试清单（2026-11-24）

- 决策：项目内统一脚本兼容 PowerShell 7/Windows PowerShell 5.1，明确检查工具而不自动安装系统组件；bootstrap 冻结两份锁文件，其他脚本传递失败退出码并记录本地日志。
- 原因：T01 的 Codex 内置 pnpm/Node 差异和从任意目录恢复需求需要统一入口。新增 Biome recommended 规则、Vitest 连接状态测试和 Tauri mock IPC 注册/拒绝测试；不增加产品功能。
- Windows 修复：Rust 测试首次返回 0xc0000139；dumpbin 确认引用 Common Controls v6 API，与 [Tauri 官方问题 #13419](https://github.com/tauri-apps/tauri/issues/13419) 相符。最终将 manifest 作为单一项目文件，为应用及测试链接，移除重复的默认 manifest，避免重复资源错误；不依赖内部 __TAURI_WORKSPACE__ 环境变量或关闭测试。
- 验证：2 项真实 mock IPC 测试、3 项前端行为测试、fmt/clippy/lint/typecheck 在两种 PowerShell 下通过；最终 release exe 构建通过。模拟原生命令 exit 7 被正确识别为失败。
- CI：官方 Actions 使用经 ls-remote 核实的固定提交，权限 contents:read，Windows runner 使用同样脚本；实际 CI 结果在 push 后核实。

## D009 — T02 分成可独立验收的两部分（2026-11-24，当时状态）

- 决策：T02-a 交付环境脚本、自动化测试、release 构建与 Windows CI 基线；T02-b 完成最终统一 dev 脚本的真实原生窗口/IPC、按钮和退出复验。T02 父任务保持 IN_PROGRESS，唯一下一任务为 T02-b。
- 原因：T01 恢复窗口已实测通过；在 T02 实施后的最终窗口复验时，computer-use 报告物理 Esc 停止。遵守停止桌面控制，继续保存和验证不依赖 UI 的已授权工作；不把窗口未验写成通过。
- 影响：T02-a 是已通过本地验收的独立工程基线，可独立提交/推送并核实 CI；T02-b 完成前不进入 T03。不以 mock 测试代替真实 GUI。
- 验证：当时 T02-b 的 GUI 证据待补齐；后续后台验收已完成，见 D010 和 TESTING。

## D010 — 用户使用桌面期间的后台 UI 验收（2026-11-24）

- 决策：用户要求后台 UI 测试，默认不使用会抢焦点或注入系统输入的桌面工具。T02-b 采用隐藏的真实 Tauri/WebView2 实例，Playwright 通过 loopback CDP 运行交互和截图，正式应用配置不变。
- 原因：用户仍在使用桌面，先前 Esc 会停止桌面控制；协议测试可验证真实渲染和 IPC，同时避免与用户输入争用。
- 隔离：专用 target/ui-smoke、独立 .local profile/临时配置、随机本机端口；只对测试实例授予读取窗口状态和关闭权限。脚本验证 hidden/unfocused 并检查正常退出和端口释放。没有把浏览器 mock 当作原生结果。
- 验证：PowerShell 7/5.1（含项目目录外）后台 smoke 通过，真实按钮 IPC 响应核对、1000x700 渲染截图、native is_visible/is_focused=false、退出 0/端口释放均有证据。因此 T02-b 和父任务 T02 的工程 smoke 验收完成。
- 范围：标题栏实体点击、DPI 矩阵、系统弹窗、音频设备留在相应后续任务；后台原生 UI 测试当前未纳入云端 CI。T02-a 的 Windows CI 已实际成功，记录见 TESTING。

## 尚未决定

- 项目许可证、正式分发应用标识、Windows 支持范围、安装包签名与分发策略。
- 不添加 LICENSE 冒充授权；不把本机 smoke test 当作所有 Windows 版本、设备或安装环境已通过。
