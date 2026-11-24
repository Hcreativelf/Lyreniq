# Lyreniq 验证记录

## T02-b 后台原生 UI 验收（2026-11-24）

起点：`def1765d43745d9b310fbeba511ff257ee140f9a`（T02-a）。开始时工作区干净、远端一致；先复跑 check-env/bootstrap/test/build。上轮 [Windows checks run 45338108012](https://github.com/Hcreativelf/Lyreniq/actions/runs/45338108012) 的实际 conclusion=success，工程检查与 release 构建全部通过。

用户正在使用桌面，本轮采用隐藏的真实 Tauri/WebView2 实例，通过 loopback CDP 操作页面；没有使用桌面控制工具、前置窗口或注入系统鼠标键盘。测试配置、profile 和构建目录隔离，不修改生产窗口配置或 ACL。

| 命令/检查 | 实际结果 |
|---|---|
| scripts/test-ui.ps1（PowerShell 7） | 最终方案全部 5 项检查通过，退出 0 |
| 在 D:\ 经 Windows PowerShell 5.1 调用 scripts/test-ui.ps1 | 最终格式化脚本全部 5 项检查通过，退出 0；8.345 秒 |
| 真实页面与 Rust | 页面显示 Lyreniq；实际 runtime_label 返回 Lyreniq 0.1.0 · windows |
| 后台状态 | 实际原生 is_visible=false、is_focused=false；按钮与截图后重复断言通过 |
| 重连按钮 | CDP 点击页面按钮；观察真实 POST ipc.localhost/runtime_label，状态 200、tauri-response=ok，响应值与 Rust 版本一致 |
| 渲染器截图 | 1000×700、无页面溢出；已人工查看生成 PNG；不包含用户桌面 |
| 正常关闭 | 测试专用窗口 close IPC；dev 退出 0；Vite 与 CDP 端口释放；不冒充标题栏实体点击 |
| 最终 bootstrap.ps1 | 退出 0；pnpm frozen-lockfile、cargo fetch --locked 通过 |
| 最终 test.ps1 | 退出 0；PS/MJS 解析、MJS Biome recommended、Rust fmt/clippy、2 项 Rust IPC、前端 lint/types 和 3 项 Vitest 通过 |
| 最终 build.ps1 | 退出 0；target/release/lyreniq-desktop.exe 生成；安装包留给 T24 |

证据：[后台渲染截图](evidence/T02-b/native-window.png)、[脱敏结果](evidence/T02-b/result.json)。原始最终运行位于 .local/ui-smoke/2026-11-24T06-49-07-123Z，独立日志和 latest.json 保留在被忽略的 .local 内。playwright-core 1.65.1 精确锁定，直接连接已安装 WebView2，没有下载另一个浏览器。

中间一次观测方案因尝试替换 Tauri 只读 invoke 属性而超时，结果保留为 failed；最终改为监听真实 IPC 网络响应，随后两次完整验证均通过，没有更改保护或降低返回值断言。

T02-b 与 T02 父任务的工程 smoke 验收完成。标题栏实体交互、DPI 矩阵、系统权限、设备与生产离线不在本次结果内。现有 GitHub CI 只执行工程检查和 release 构建，后台原生 UI 目前仅本机实测。本轮提交后的 CI 实际状态由交付报告和对应 SHA 的 Actions 核实，下一轮回填。

## T02-a 验收与当时的 T02-b 待验（历史快照，2026-11-24）

以下保留当时的真实状态；T02-b 的后续完成证据及 T02-a 的实际 CI 成功结果见上节。

起点：`0ba2701de101f39f6205b0836127969a773cf456`。工作区起初干净，fetch 后与远端一致；先复跑 T01 的 frozen 安装、fmt、cargo check、前端构建，并实际观察原生窗口及真实 runtime_label 成功，再实施 T02。

| 命令/检查 | 实际结果 |
|---|---|
| 在 D:\ 调用 scripts/check-env.ps1 | PowerShell 7 和 Windows PowerShell 5.1 均退出 0；工具版本、MSVC、SDK、WebView2 通过 |
| 在 D:\ 调用 scripts/bootstrap.ps1 | 0；pnpm install --frozen-lockfile、cargo fetch --locked --target x86_64-pc-windows-msvc 通过 |
| scripts/test.ps1 | PowerShell 7、5.1 均退出 0；PowerShell 解析、Rust fmt/clippy、前端 Biome/typecheck 通过 |
| Rust IPC 测试 | 2 passed：注册命令可经 IPC 调用、未注册命令被拒绝；main/doc test 为 0 项，不计入 2 项 |
| 前端测试 | 3 passed：等待真实响应才显示成功、失败后可重试、后续连接失败清除旧成功读数 |
| Invoke-Checked 的受控 exit 7 演练 | 成功捕获为失败；没有被 Out-Host/日志管线吞掉退出码 |
| scripts/build.ps1 | 0；最终 Windows 清单配置在 PowerShell 5.1（项目目录外）通过；产物 target/release/lyreniq-desktop.exe；跳过安装包 |
| scripts/dev.ps1 | 在 D:\ 经 PowerShell 5.1 启动 Vite 与 cargo run --locked，原生 exe 已运行；最终可视窗口/按钮/退出复验未完成，见下 |
| 本地日志 | .local/logs/bootstrap.log、test.log、build.log、dev.log；命令、退出码与 stdout，stderr 同时显示于控制台；均忽略 |
| Windows CI | 配置已建立；本记录写于 push 前，实际运行状态由 push 后交付报告及 GitHub Actions 记录核实 |

首次 Rust 测试在运行前报 STATUS_ENTRYPOINT_NOT_FOUND（0xc0000139）；链接诊断确认 Common Controls v6 API。第一次直接追加清单还造成重复 manifest（LNK1123）；最终使用单一 windows.manifest，为应用和测试共同链接，随后 Rust 测试与 release 构建通过。没有降低断言、跳过测试或安装替代 DLL。

新增依赖固定为 Vitest 4.1.12、Biome 2.6.8、Testing Library React 16.3.3/user-event 14.6.13、jest-dom 7.0.1、jsdom 27.4.0；保持 T01 的 Vite/React 主版本。Biome 使用 recommended preset，不关闭 lint 规避问题。

**T02-b 尚未验收**：最终统一 dev 脚本的窗口检查期间，桌面工具报告物理 Esc 停止。本轮已停止桌面控制并终止测试用开发进程。脚本启动输出、mock IPC 和此前 T01 的窗口证据均不能替代这一最终 GUI 复验，因此 T02 父任务仍为 IN_PROGRESS。

T02-b 恢复时先核实 Git/文件并重跑 check-env/bootstrap/test/build，再启动 dev.ps1，实际检查窗口、Rust 返回值、按钮和退出；必要时保存 docs/evidence/T02-b。通过后才将 T02 父任务标为 VERIFIED，并进入 T03。

## T01 验收（2026-11-24）

起点为 T00 提交 `562cfadb1b818b1f04b19d05c114b9cba211c5fd`。实施前重跑 T00 的文档/哈希断言、15 项忽略与 6 项可跟踪例外，检查 Git 工作区、历史、diff 与提交空白，全部符合预期；上一轮没有应用，应用构建/GUI 回归不适用。

| 命令/检查 | 结果 | 证据 |
|---|---|---|
| `node --version`、`pnpm --version`、`rustc --version` | 均 0 | v24.21.0 / 11.25.0 / rustc 1.100.0；环境选择见 ENVIRONMENT |
| `pnpm install --frozen-lockfile` | 0 | Already up to date；两项目 workspace；仅 esbuild 构建脚本被批准 |
| `cargo fmt --all -- --check` | 0 | Rust 格式检查通过 |
| `cargo check --workspace --locked` | 0 | lyreniq-desktop 编译检查通过；最终配置复验耗时 0.68s |
| `pnpm build` | 0 | TypeScript 无错误；Vite 7.3.8 构建 32 个模块；最终 459ms |
| `pnpm dev` | 原生窗口/IPC 通过；退出 0 | 原生 exe=target/debug/lyreniq-desktop.exe；页面显示 Lyreniq 0.1.0 · windows；重新验证按钮调用后仍成功 |
| Windows 原生窗口证据 | 已保存 | [截图](evidence/T01/native-window.jpg)、[观测记录](evidence/T01/native-window.json) |
| T00 GitHub 补推及核实 | 0，远端 hash 一致 | `git push -u origin main`、`git ls-remote --heads origin main` 返回 T00 hash |
| 提交前远端同步 | 0 | `git fetch origin`、`git rev-list --left-right --count HEAD...origin/main` 输出 0/0 |

首次 pnpm 安装因旧式配置/脚本未授权失败，修正 workspace YAML 和共享 pnpm 选择后重装成功；重复键错误也已修复，未把这些失败记成通过。MSVC 安装器返回 3010，后续 vswhere isComplete/isLaunchable=true、isRebootRequired=false，未重启即完成实际编译和窗口验收。

窗口正常渲染并完成按钮验证；后续 inventory 显示窗口已退出，开发 CLI 返回 0。尝试代理 Alt+F4 时因检测到用户输入被保护机制拦截，未把这一动作计作代理自动关闭测试通过。首次遮挡的截图没有用作证据；已前置窗口后保存纯 Lyreniq 截图。

T01 尚无单元测试套件、lint 或 CI；不声称 cargo test、Vitest、安装包构建、真实音频或生产离线运行通过。T02 将建立相应工程回归基线。

下一轮先选择 ENVIRONMENT 中的共享工具链，在项目根目录复跑：

```powershell
node --version
pnpm --version
rustc --version
pnpm install --frozen-lockfile
cargo fmt --all -- --check
cargo check --workspace --locked
pnpm build
pnpm dev
git diff --check
git show --format= --check HEAD
```

smoke test 必须观察真实原生窗口和 Rust 返回值，不能只看到 Vite ready 就判断桌面验收通过。核对两份锁文件仍被跟踪，以及 target/node_modules/dist/.local/gen 未被提交。原任务书 SHA-256 仍与 D002 基准一致。

## T00 验收范围（历史快照，2026-11-24）

本轮只有文档和 Git 初始化。原目录没有上一轮成果、测试或应用；上一轮应用回归不适用。任务书中的算法、UI、设备与发布矩阵是未来目标，本轮不把它们记作通过。

| 检查 | 当前结果 | 证据 |
|---|---|---|
| 原目录、文件与 Git 盘点 | 通过 | 原目录只有任务书；Git 初始不存在；见 HANDOFF |
| 完整阅读主任务书 | 完成 | 按行段读取全部 1,213 行 |
| Git 初始化 | 通过 | `git init -b main` 返回 0 |
| 原件保留与 docs 副本哈希 | 通过 | SHA-256 均与 DECISIONS D002 基准相同 |
| 基础文档、链接、任务 ID 一致性 | 通过；PowerShell 退出 0 | 11 个文件、8 个本地链接；当前 T00，唯一下一任务 T01；没有提前创建应用工程 |
| Git 空白及 staged 内容审查 | 通过；退出 0 | `git diff --check`、`git diff --cached --check`；仅 11 个本轮文件 |
| 忽略缓存、音频、用户数据和密钥 | 通过 | 15/15 忽略；6 项 fixtures/锁文件/环境示例未忽略；分别退出 0/1（均符合预期） |
| Git 暂存区任务书原始字节 | 通过 | 两个 indexed blob 与 `git hash-object --no-filters` 相同；见 HANDOFF |
| 常见凭据模式扫描与文本审查 | 通过 | 私钥/GitHub token/AWS access key 模式无匹配（rg 退出 1，预期）；人工检查 staged 文档 |
| GitHub remote / gh 认证 | 待配置 / CLI 不可用 | 无 remote；`gh auth status` 返回 1，未安装 gh |
| 应用测试、构建、GUI、设备、安装 | 未执行；不适用 | 没有应用、清单、测试或脚本 |

## 复验入口

从项目根目录运行；PowerShell 使用 `-NoProfile` 可避开本机已有 profile 的 conda 启动错误。

```powershell
git rev-parse --show-toplevel
git status --short --branch
git branch --show-current
git log -5 --oneline
git diff
git diff --cached
git remote -v
git diff --check
git show --format= --check HEAD
Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256
Get-FileHash -LiteralPath docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256
```

检查 AGENTS、README、STATUS、HANDOFF、DECISIONS、TESTING 的本地 Markdown 链接均指向现有文件；核实软件名 Lyreniq、T00 结果和唯一下一任务 T01，并与实际文件和 Git 日志对照。T00 两份任务书哈希相同；未来主文档获批修订后以决策记录为准。

本轮文档内联断言，可在 T01 开始实施前复跑（不安装工具、不创建文件）：

```powershell
$ErrorActionPreference = 'Stop'
$taskExpected = @('AGENTS.md','README.md','.gitignore','.gitattributes','.editorconfig','UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md','docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md','docs/STATUS.md','docs/HANDOFF.md','docs/DECISIONS.md','docs/TESTING.md')
foreach ($taskFile in $taskExpected) {
    if (-not (Test-Path -LiteralPath $taskFile -PathType Leaf)) { throw "Missing file: $taskFile" }
}
$taskFiles = @(& rg --files --hidden -g '!.git')
if ($LASTEXITCODE -ne 0 -or $taskFiles.Count -ne $taskExpected.Count) { throw 'Unexpected project file inventory' }
$taskLinks = 0
foreach ($taskFile in @('AGENTS.md','README.md','docs/STATUS.md','docs/HANDOFF.md','docs/DECISIONS.md','docs/TESTING.md')) {
    $taskText = Get-Content -LiteralPath $taskFile -Raw -Encoding UTF8
    foreach ($taskMatch in [regex]::Matches($taskText,'\[[^\]]+\]\(([^)]+)\)')) {
        $taskLink = $taskMatch.Groups[1].Value
        if ($taskLink -match '^(https?://|#)') { continue }
        $taskLinkPath = Join-Path (Split-Path -Parent (Join-Path (Get-Location) $taskFile)) $taskLink
        if (-not (Test-Path -LiteralPath $taskLinkPath -PathType Leaf)) { throw "Broken link in ${taskFile}: $taskLink" }
        $taskLinks++
    }
}
foreach ($taskFile in @('README.md','docs/STATUS.md','docs/HANDOFF.md')) {
    $taskText = Get-Content -LiteralPath $taskFile -Raw -Encoding UTF8
    foreach ($taskToken in @('Lyreniq','T00','T01')) {
        if (-not $taskText.Contains($taskToken)) { throw "Missing ${taskToken}: $taskFile" }
    }
}
foreach ($taskPath in @('Cargo.toml','package.json','apps','crates','scripts','.github')) {
    if (Test-Path -LiteralPath $taskPath) { throw "T00 scope exceeded: $taskPath" }
}
$taskOriginalHash = (Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256).Hash
$taskDocHash = (Get-FileHash -LiteralPath docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256).Hash
if ($taskOriginalHash -ne 'C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF' -or $taskDocHash -ne $taskOriginalHash) { throw 'Plan bytes changed' }
Write-Output "PASS: $($taskFiles.Count) project files, $taskLinks local links, task/name consistency, T00 scope, both plan SHA-256 values"
```

该断言专用于 T00 基线。T01 开始新增工程后，文件数量/范围断言和副本哈希条件需按实际任务调整，不能误报后续合法变更失败。

忽略规则可用以下无须创建文件的命令复验：

```powershell
git check-ignore --no-index -- target/debug/probe.obj node_modules/probe.js apps/desktop/dist/probe.js .local/cache/probe .venv/probe .env .env.production debug.log recordings/probe.wav probe.wav user-data/settings.json settings.json installer.msi signing.key .vscode/settings.json
git check-ignore --no-index -- .env.example Cargo.lock pnpm-lock.yaml tests/fixtures/tone.wav tests/fixtures/tone.mp3 tests/fixtures/tone.flac
```

第一条期望逐个输出全部路径且退出码为 0；第二条期望无输出且退出码为 1（均未被忽略，属于预期结果）。正式夹具仅允许小型合成或明确授权音频，不能因存在例外就提交个人录音。

最终提交结果以 `git log` 和交付报告核实；GitHub push 状态独立记录，不能由本地 Git 成功推断。常见凭据模式扫描及人工 staged diff 审查用于本轮文本核验，不代表未来文件免于审查。
