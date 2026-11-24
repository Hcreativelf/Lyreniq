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
| PowerShell | 7.6.9 | 本机实际版本；统一脚本在 T02 建立 |
| Git | 2.55.0.windows.5 | 本机实际版本 |
| Tauri crate / tauri-build | 2.12.7 / 2.7.6 | src-tauri/Cargo.toml 精确约束、Cargo.lock |
| Tauri CLI / JS API | 2.12.6 / 2.12.5 | desktop/package.json 精确约束、pnpm-lock.yaml |
| React / React DOM | 19.3.1 | desktop/package.json、pnpm-lock.yaml |
| TypeScript / Vite / React 插件 | 5.9.3 / 7.3.8 / 5.2.0 | desktop/package.json、pnpm-lock.yaml |

本机注册表报告 ProductName=Windows 10 Pro、DisplayVersion=26H2、build=26300.9974，架构 AMD64。仅以这台 Windows x64 机器为当前验证环境，不据此声称已覆盖所有 Windows 10/11 版本。

## 安装与存放

用户已授权安装 T01 必需组件，并允许公共环境共享安装。Rust 在标准用户目录 `.cargo`/`.rustup`，pnpm 在标准 npm 用户级目录，MSVC/SDK 在标准系统目录；不强制迁移已有全局工具链。

项目保留全部版本约束、两份依赖锁文件和 `.vsconfig` 组件清单。pnpm store 位于 `.local/pnpm-store`；node_modules、target、dist、Tauri 生成 schema 和 .local 均不提交。

安装器保存在 `.local/downloads`（忽略）。Rustup 来自 Rust 官方，并与官方 SHA-256 文件核对，哈希为 `38027D51D69A5D05493627A0634DBEA52D6CD793C8F072D2373D3F4163EEC7DB`。Build Tools 安装器来自 Microsoft 官方，运行前 Authenticode 状态为 Valid、签名主体 Microsoft Corporation。

MSVC 安装命令只指定 `.vsconfig` 的两个组件，使用 `--quiet --wait --norestart --nocache`；安装器返回 3010，未自动重启。安装后 vswhere 报告 isComplete=true、isLaunchable=true、isRebootRequired=false。当前已通过编译检查；后续是否需要重启以实际错误或系统提示为准。

Rust 安装参数为 `-y --profile minimal --default-host x86_64-pc-windows-msvc --default-toolchain 1.100.0 --component rustfmt,clippy`。新终端会加载安装后的用户 PATH；不修改已有 PowerShell profile。

首次依赖下载允许联网，正式应用不应依赖网络；T01 开发模式使用本机 Vite 服务器，不代表生产离线验收完成。

## 运行方式

从项目根目录运行。新终端中确认 node/pnpm/rustc 的版本与上表一致，然后执行 README 中的安装、构建与启动命令。

Codex 的内置 pnpm 包含另一 Node 版本（24.22.3）。本项目本轮采用共享 pnpm 搭配用户现有 Node 24.21.0；在当前 Codex shell 需要时按以下方式临时调整进程 PATH，不修改系统 PATH：

```powershell
$taskNodeDirectory = Split-Path -Parent (Get-Command node -CommandType Application).Source
$env:PATH = "$taskNodeDirectory;$(Join-Path $env:APPDATA 'npm');$(Join-Path $env:USERPROFILE '.cargo\bin');$env:PATH"
node --version
pnpm --version
rustc --version
```

pnpm 11 使用 `pnpm-workspace.yaml` 的 storeDir/engineStrict/saveExact/allowBuilds 配置；仅批准 esbuild 构建脚本。初次安装曾因旧式配置和未批准脚本失败，修正后已重新安装成功并通过 frozen-lockfile 检查。初次失败产生过默认共享 store，不删除或假设其仅属于本项目。

若系统 PowerShell profile 的 conda 启动报错，可使用 `pwsh -NoProfile`；本项目未改动用户 profile。

## 官方依据

- [Tauri Windows 前置要求](https://v2.tauri.app/start/prerequisites/)
- [Tauri Vite 配置](https://v2.tauri.app/start/frontend/vite/)
- [Rustup 安装及目录配置](https://rust-lang.github.io/rustup/installation/index.html)
- [Microsoft Build Tools 组件 ID](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022)
- [pnpm 配置](https://pnpm.io/settings)

首次选择版本后必须通过本机实测才作为基线；不要在下一轮无理由升级。
