# Lyreniq 交接记录

## 当前任务与起点

- 日期：2026-11-24（Asia/Shanghai）；任务：T01 — Windows 工具链与最小窗口。
- 目录：D:\2-workspace\Lyreniq；分支：main，跟踪 origin/main。
- 起点 commit：562cfadb1b818b1f04b19d05c114b9cba211c5fd（T00）。开始时工作区干净，T00 文档、哈希、忽略规则与 Git 提交复验通过。
- 用户后续授权：额度可用时逐项自主推进；每项仍复验、验收、交接、提交/推送。必需工具链允许系统/用户级共享安装，具体安装已获授权。
- 本项仅建立 Windows 最小工程；未做调音 UI、DSP、麦克风、文件分析、其他平台或安装包。

## 完成内容与文件

- Cargo/pnpm workspace、单个 apps/desktop Rust/Tauri + React/TS/Vite 工程、只读 runtime_label 原生命令和重新连接按钮。
- 版本约束：.node-version、rust-toolchain.toml、package.json、两份锁文件；MSVC/SDK 组件清单 .vsconfig。
- 项目内 pnpm store 和忽略规则；自有 SVG/Windows 图标；生产/开发 CSP 和无插件权限 capability。
- AGENTS、README、STATUS、DECISIONS、TESTING、ENVIRONMENT 已同步；两份原任务书未修改。
- 原生窗口证据：docs/evidence/T01/native-window.jpg、native-window.json。

## 实际验收与命令

命令在项目根目录运行；当前 Codex shell 使用 ENVIRONMENT 里的临时 PATH 块选择用户 Node、共享 pnpm 和 Rust。没有修改 PowerShell profile。

| 命令/操作 | 退出码/结果 |
|---|---|
| T00 文档/哈希 PowerShell 断言、忽略规则复验 | 0；11 个初始文件、8 个链接、15 项忽略和 6 项例外均正确 |
| git status/log/diff/cached/remote、git show --format= --check HEAD | 通过；起点干净 |
| node --version、pnpm --version、rustc --version | 0；24.21.0 / 11.25.0 / 1.100.0 |
| 官方安装器下载、Rustup SHA-256、Microsoft 数字签名验证 | 通过；安装位置与精确命令见 ENVIRONMENT |
| MSVC/SDK 安装 | 安装器 3010；包装命令 0；未重启；后续 vswhere 显示安装完整/可启动且无重启标记 |
| rustup-init.exe -y --profile minimal --default-host x86_64-pc-windows-msvc --default-toolchain 1.100.0 --component rustfmt,clippy | 0 |
| npm install --global pnpm@11.25.0 --fetch-retries=0 --fetch-timeout=30000 | 0 |
| pnpm install --force（修正配置后的最终一次） | 0；store 位于项目 .local/pnpm-store/v11 |
| pnpm install --frozen-lockfile | 0 |
| cargo fmt --all -- --check | 0 |
| cargo check --workspace --locked | 0；最终配置复验 0.68s |
| pnpm build | 0；TypeScript + Vite 32 模块，最终 459ms |
| pnpm dev | 实际原生窗口和 Rust IPC/按钮通过；窗口退出后 CLI 0 |
| git fetch origin、git rev-list --left-right --count HEAD...origin/main | 0；提交前 0/0 |
| git diff --check | 0 |

图标通过 `pnpm --filter @lyreniq/desktop tauri icon src-tauri/icons/icon.svg --output ../../.local/generated-icons` 生成，只复制 ICO/PNG 到工程；其他临时输出在忽略目录内，没有创建其他平台工程。

首次安装因沙箱 DNS 失败，官方联网操作通过权限流程完成。初次 pnpm 安装/重复 YAML 键曾失败，修正 pnpm 11 配置后重新安装和 frozen 检查均成功，未隐藏失败。

原生窗口以 computer-use 观测：Lyreniq 标题、桌面连接已建立、Lyreniq 0.1.0 · windows；点击重新验证仍成功。已保存纯窗口截图。后续窗口已退出且开发 CLI 为 0；代理 Alt+F4 曾因用户输入保护被拦截，不声称该自动关闭动作完成。

## Git 连接与提交

- 用户已提供并授权 https://github.com/Hcreativelf/Lyreniq.git（SSH 备选 git@github.com:Hcreativelf/Lyreniq.git）。使用 HTTPS，未改仓库可见性。
- 只读 ls-remote 确认最初无分支后配置 origin，`git push -u origin main` 成功。
- T00 远端 main 已核实为 562cfadb1b818b1f04b19d05c114b9cba211c5fd，与本地相同。
- T01 本地验收已通过，提交标题 `build: bootstrap Windows Tauri workspace`。本记录在提交前写入；本轮最终 hash、push 和 clean 状态以 Git 输出和交付报告为准，下次可回填，不写入提交自身 hash。
- 正常 push 后必须比较 `git ls-remote --heads origin main` 与本地 HEAD；禁止强推和覆盖其他提交。

## 未验证与风险

- 尚无单元测试、lint、统一 PowerShell 脚本或 CI，属于 T02；未执行或声称 cargo test/Vitest/CI 通过。
- 未实现或验证任何音频、产品 UI、深浅主题、DPI 矩阵、生产离线运行或安装包，不把最小窗口称为可用调音器。
- MSVC 安装器曾请求重启；当前工具和窗口已实际通过，无须为本项自动重启。若后续遇到系统组件问题，请用户协助重启后复验。
- 共享系统依赖需要在新机器安装，不能通过复制项目目录替代；项目许可证及正式分发标识仍待确定。

## 下一轮必须复验

1. 读取 AGENTS、README、docs 下主任务书、STATUS、HANDOFF、DECISIONS、TESTING、ENVIRONMENT，核对 T01 最终 commit。
2. 检查 git 根目录/分支/status/log/diff/cached/remote；fetch origin 并检查 ahead/behind，保留用户修改。
3. 使用 ENVIRONMENT 的工具链：node/pnpm/rustc --version；pnpm install --frozen-lockfile；cargo fmt --all -- --check；cargo check --workspace --locked；pnpm build。
4. pnpm dev 实际窗口 smoke test：观察原生窗口和真实 runtime_label 返回值，必要时重新连接；仅 Vite ready 不代表通过。
5. git diff --check、git show --format= --check HEAD；核对两份锁文件、原任务书哈希和缓存忽略规则。

唯一下一任务：**T02 — 环境脚本与回归基线**。先复验 T01，再建立项目内脚本、最小测试/lint 与 Windows CI，不增加产品功能。
