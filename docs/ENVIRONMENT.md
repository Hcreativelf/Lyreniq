# Windows 开发环境

## 已选择的 T01 基线（2026-11-24）

| 工具/依赖 | 实际版本 | 约束或记录位置 |
|---|---|---|
| Node.js | 24.21.0 | .node-version、根 package.json engines |
| pnpm | 11.25.0 | packageManager、engines |
| Rust / Cargo | 1.100.0 / 1.100.0 | rust-toolchain.toml；x86_64-pc-windows-msvc；rustfmt/clippy |
| MSVC Build Tools 2022 | 17.14.45 / installationVersion 17.14.37906.3 | 共享系统安装；VC 工具目录 14.44.35207 |
| Windows SDK | 10.0.26100.0 | 共享系统安装；组件清单 .vsconfig |
| WebView2 Runtime | 157.0.4341.60 | 复用系统现有运行时，未重复安装 |
| PowerShell | 7.6.9 / Windows PowerShell 5.1 | T02 脚本兼容两者，已实测 |
| Git | 2.55.0.windows.5 | 本机实际版本 |
| Tauri crate / tauri-build | 2.12.7 / 2.7.6 | src-tauri/Cargo.toml 精确约束、Cargo.lock |
| Tauri CLI / JS API | 2.12.6 / 2.12.5 | desktop/package.json 精确约束、pnpm-lock.yaml |
| React / React DOM | 19.3.1 | desktop/package.json、pnpm-lock.yaml |
| TypeScript / Vite / React 插件 | 5.9.3 / 7.3.8 / 5.2.0 | desktop/package.json、pnpm-lock.yaml |
| Vitest / Biome | 4.1.12 / 2.6.8 | T02 测试与格式/lint，保持 Vite 7 主版本 |
| Testing Library React / user-event | 16.3.3 / 14.6.13 | T02 连接状态与用户重试测试 |
| jest-dom / jsdom | 7.0.1 / 27.4.0 | 与 Vitest 4 相容的 matcher 和 DOM 环境 |
| playwright-core | 1.65.1 | 后台直连已有 WebView2，不下载浏览器 |
| lucide-react | 1.75.0 | T03 本地图标；React 19 peer 相容；ISC/Feather MIT 声明见 docs/licenses |

本机注册表报告 ProductName=Windows 10 Pro、DisplayVersion=26H2、build=26300.9974，架构 AMD64。仅以这台 Windows x64 机器为当前验证环境，不据此声称已覆盖所有 Windows 10/11 版本。

## 安装与存放

用户已授权安装 T01 必需组件，并允许公共环境共享安装。Rust 在标准用户目录 `.cargo`/`.rustup`，pnpm 在标准 npm 用户级目录，MSVC/SDK 在标准系统目录；不强制迁移已有全局工具链。

项目保留全部版本约束、两份依赖锁文件和 `.vsconfig` 组件清单。pnpm 项目逻辑路径为 `.local/pnpm-store`，本机通过 junction 集中到 D 盘缓存目录，见下节；node_modules、target、dist、Tauri 生成 schema 和 .local 均不提交。

## 本机集中缓存（2026-11-24）

用户持久偏好：当前 Windows 电脑的可迁移缓存集中到 D:\4-stuff\Cache。该路径不是账号云端记忆，不在没有此 D 盘的其他电脑照搬。项目配置、依赖安装目录、构建产物、日志、测试 profile、源码和凭据仍按用途存放。

| 工具 | 当前实际缓存位置 | 配置/兼容方式及验证 |
|---|---|---|
| pnpm 全局 store | D:\4-stuff\Cache\pnpm\store | 项目外 pnpm config get store-dir 已核实 |
| pnpm 其他缓存 | D:\4-stuff\Cache\pnpm\cache | pnpm config get cache-dir 已核实 |
| Lyreniq pnpm store | D:\4-stuff\Cache\pnpm\projects\Lyreniq | 原 .local/pnpm-store 为 junction；保留项目相对 storeDir；frozen 安装通过 |
| npm | D:\4-stuff\Cache\npm\cache | npm 用户级原生 cache 配置；cache verify 实际写入新路径；旧 %LOCALAPPDATA%/npm-cache 保留 junction |
| Cargo registry | D:\4-stuff\Cache\cargo\registry | %USERPROFILE%/.cargo/registry 保留 junction；核实官方目录用途与 CACHEDIR.TAG；10,845 个文件逐一 SHA-256 相同 |
| Vite | D:\4-stuff\Cache\vite\Lyreniq | 原生 cacheDir；真实后台启动后 deps 实际生成 |
| Vitest | D:\4-stuff\Cache\vitest\Lyreniq | 原生 cacheDir；测试后 vitest 缓存实际生成 |

npm 旧缓存的 62 个文件已逐一 SHA-256 核实后迁移；原 node_modules/.vite 内的 21 个旧缓存文件迁到 Vite 目录的 legacy-node_modules-vite 子目录并保留 junction，哈希一致。机器迁移日志保留在 .local/logs/T03-cache-migration.jsonl 等本地日志，不提交。

Vite/Vitest 通过忽略的 `.local/cache-paths.json` 读取 `{ "cacheRoot": "D:/4-stuff/Cache" }`，也支持进程环境变量 LYRENIQ_CACHE_ROOT；由 cache-paths.ts 传给官方 cacheDir。没有本机配置的 clone/CI 自动使用项目 .local/cache/vite 与 .local/cache/vitest。机器专属路径没有写入共享 Vite 配置。

Cargo 官方支持整体 CARGO_HOME，但其中同时包含程序、配置和凭据；本轮只迁移确认属于缓存的 registry，保留其他内容的位置。registry 内部结构不是稳定 API，升级 Cargo 后需复核 junction 与缓存行为。Rustup 工具链不是缓存，未归入缓存目录；未来安装工具链时再检查其下载缓存。Vite 的临时配置加载文件不属于此 cacheDir 控制范围，不将整个 node_modules 迁移为缓存。

依据：[npm cache 配置](https://docs.npmjs.com/cli/v11/using-npm/config/#cache)、[pnpm 配置](https://pnpm.io/settings)、[Vite cacheDir](https://vite.dev/config/shared-options.html#cachedir)、[Cargo Home 目录说明](https://doc.rust-lang.org/cargo/guide/cargo-home.html)。

安装器保存在 `.local/downloads`（忽略）。Rustup 来自 Rust 官方，并与官方 SHA-256 文件核对，哈希为 `38027D51D69A5D05493627A0634DBEA52D6CD793C8F072D2373D3F4163EEC7DB`。Build Tools 安装器来自 Microsoft 官方，运行前 Authenticode 状态为 Valid、签名主体 Microsoft Corporation。

MSVC 安装命令只指定 `.vsconfig` 的两个组件，使用 `--quiet --wait --norestart --nocache`；安装器返回 3010，未自动重启。安装后 vswhere 报告 isComplete=true、isLaunchable=true、isRebootRequired=false。当前已通过编译检查；后续是否需要重启以实际错误或系统提示为准。

Rust 安装参数为 `-y --profile minimal --default-host x86_64-pc-windows-msvc --default-toolchain 1.100.0 --component rustfmt,clippy`。新终端会加载安装后的用户 PATH；不修改已有 PowerShell profile。

首次依赖下载允许联网，正式应用不应依赖网络；T01 开发模式使用本机 Vite 服务器，不代表生产离线验收完成。

## 运行方式

优先运行 README 中的 scripts/check-env.ps1、bootstrap.ps1、test.ps1、build.ps1、dev.ps1。支持 PowerShell 7 和 Windows PowerShell 5.1；脚本内容采用 ASCII，避免 5.1 对无 BOM UTF-8 脚本的解码问题。脚本从自身位置定位项目，恢复调用前的工作目录和进程 PATH，不修改系统设置。

check-env 只检查已安装的锁定版本、MSVC、SDK 及 WebView2；Rust 检查先查询已安装工具链，再使用 rustup run，避免 rustc 代理为缺失版本自动下载。缺失工具链须按本文件安装后重跑；bootstrap 仅安装项目依赖，不安装系统工具。

Codex 的内置 pnpm 包含另一 Node 版本（24.22.3）。本项目本轮采用共享 pnpm 搭配用户现有 Node 24.21.0；在当前 Codex shell 需要时按以下方式临时调整进程 PATH，不修改系统 PATH：

```powershell
$taskNodeDirectory = Split-Path -Parent (Get-Command node -CommandType Application).Source
$env:PATH = "$taskNodeDirectory;$(Join-Path $env:APPDATA 'npm');$(Join-Path $env:USERPROFILE '.cargo\bin');$env:PATH"
node --version
pnpm --version
rustc --version
```

pnpm 11 使用 `pnpm-workspace.yaml` 的 storeDir/engineStrict/saveExact/allowBuilds 配置；仅批准 esbuild 构建脚本。初次安装曾因旧式配置和未批准脚本失败，修正后已重新安装成功并通过 frozen-lockfile 检查。初次失败产生过默认共享 store，不删除或假设其仅属于本项目。

T02 脚本自动选择共享 pnpm、现有 Node 与 Cargo 用户安装，检查严格版本，不需要手工使用上述 PATH 块。bootstrap 使用 `pnpm install --frozen-lockfile` 与 `cargo fetch --locked --target x86_64-pc-windows-msvc`。每条原生命令的非零退出码会使脚本退出 1；已用受控 exit 7 验证失败传播。

test/build/bootstrap/dev 的日志写入 `.local/logs/<任务>.log`，包含命令、退出码及 stdout 测试摘要；原生 stderr 同时显示在控制台。日志可能含本机路径，保持本地且不提交。

build 当前使用 `tauri build --no-bundle --ci -- --locked`，生成 release exe；安装包策略留待 T24。Windows 应用和 Rust 测试共同链接项目 windows.manifest，确保 Common Controls v6 可用，避免测试程序在运行前报 STATUS_ENTRYPOINT_NOT_FOUND。未提高应用执行权限。

Windows CI 使用 windows-2022、固定提交的官方 checkout/setup-node、项目 Node/Rust/pnpm 版本与同一套脚本。无 GUI runner 显式使用 -SkipWebView2；窗口、设备、离线与安装验收仍需本机执行。初次 push 后的实际运行状态必须单独核实，不由配置文件存在推断通过。

## 后台原生 UI 测试

`pwsh -NoProfile -File scripts/test-ui.ps1`（也支持 powershell.exe）从任意目录运行。它通过统一 dev.ps1 的 BackgroundTestConfig 参数创建一个真实但 visible=false/focus=false 的窗口，窗口配置、构建目录、应用数据均单独隔离。启动阶段会编译自己的 target/ui-smoke，首次耗时较长；后续复用缓存。

测试配置仅写入 .local/ui-smoke，使用随机 loopback 端口、独立 appDirectoriesOverride 和测试专用的窗口状态读取/关闭权限。生产 tauri.conf.json 和 capability 文件不改动，测试不开放远程公网调试，也不使用桌面控制工具。脚本实际读取原生 is_visible/is_focused，点击和截图后再次检查。

Playwright 连接真实 WebView2，监听按钮触发的 Rust IPC 响应并核对返回值；直接从渲染器截取页面，而非抓取用户桌面。正常关闭使用仅对测试实例授权的窗口 close 命令，并核对 dev 进程退出 0、Vite/CDP 端口释放。失败时只终止该测试自己创建的进程树。

结果及独立 profile 留在 .local/ui-smoke，每轮结果汇总到 latest.json；脱敏截图和结果可复制到 docs/evidence。T03 已扩展到原生 1024×700/1440×900、两主题四页面的 16 组合，以及页面内键盘/焦点、文字 token 对比度和 CDP 渲染比例 125%/150%。比例模拟不等于 Windows 原生 DPI 切换；标题栏实体点击、跨屏 DPI、设备权限或完整生产离线行为仍未验证。GitHub CI 当前不运行这项原生 UI 测试。

依据：[Tauri 窗口配置](https://v2.tauri.app/reference/config/)、[Playwright CDP 接口](https://playwright.dev/docs/api/class-browsertype#browser-type-connect-over-cdp)、[Microsoft WebView2 调试参数](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags)。

T04 复用上述工具链和缓存，没有新增依赖或系统组件。后台脚本进一步检查两主题/尺寸下的 44 个 Demo 组合、页面内 Home/方向键选择、重试、退出焦点恢复与导航清理；与 T03 的 16 个工作区组合分开记录。Demo 只展示固定 TS 数据，不使用设备权限，不等于真实音频采集、算法或精度测试。

若系统 PowerShell profile 的 conda 启动报错，可使用 `pwsh -NoProfile`；本项目未改动用户 profile。

## 官方依据

- [Tauri Windows 前置要求](https://v2.tauri.app/start/prerequisites/)
- [Tauri Vite 配置](https://v2.tauri.app/start/frontend/vite/)
- [Rustup 安装及目录配置](https://rust-lang.github.io/rustup/installation/index.html)
- [Microsoft Build Tools 组件 ID](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022)
- [pnpm 配置](https://pnpm.io/settings)

首次选择版本后必须通过本机实测才作为基线；不要在下一轮无理由升级。
