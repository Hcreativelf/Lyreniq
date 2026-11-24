# Lyreniq 交接记录

## 本轮任务与起点

- 日期：2026-11-24（Asia/Shanghai）；目录 D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点：0ba2701de101f39f6205b0836127969a773cf456（T01，已推送并核实）。起初工作区干净；本轮 fetch、Git 检查及 T01 的 frozen 安装、fmt/check/build、实际原生窗口/IPC 均复验通过。
- 当前完成部分：T02-a — 环境脚本、自动化测试、release 构建及 Windows CI 基线。
- T02 父任务仍 IN_PROGRESS；T02-b 的最终统一 dev 脚本可视窗口/按钮/退出验收待完成。
- 拆分原因：最后桌面检查时工具报告物理 Esc 停止；已停止所有桌面控制。只继续保存与验证不依赖 UI 的已授权工作，见 DECISIONS D009。

## 本轮交付

- scripts/common.ps1、check-env.ps1、bootstrap.ps1、dev.ps1、test.ps1、build.ps1；支持 PowerShell 7 与 Windows PowerShell 5.1，从脚本位置定位项目，恢复调用者 cwd/PATH。
- check-env 只检查锁定 Node/pnpm/Rust、MSVC、SDK、WebView2，不自动下载工具链。bootstrap 冻结安装两套依赖；test 覆盖格式/lint/类型/测试；build 当前生成 release exe 并明确跳过 T24 安装包。
- .local/logs 保存命令、退出码和 stdout；缓存/产物/日志保持忽略。模拟原生命令 exit 7 已验证会传播为失败。
- Biome recommended、Vitest/Testing Library；3 项连接状态/重试测试。Rust 2 项 mock IPC 测试使用与生产同一命令注册函数，检查注册/拒绝边界。
- Windows Common Controls 清单统一为 src-tauri/windows.manifest，修复测试入口缺失；源代码格式化，不增加调音或其他产品功能。
- .github/workflows/ci.yml：Windows 2022 runner，固定提交的官方 checkout/setup-node，锁定 Node/pnpm/Rust，contents:read，同一套 bootstrap/test/build；CI 显式跳过 WebView2 运行时检查，不声称测试 GUI。
- README、STATUS、TESTING、ENVIRONMENT、DECISIONS 已更新；原始任务书未改。

## 实际命令与结果

| 命令/检查 | 结果 |
|---|---|
| Git 根目录/status/log/diff/cached/remote、fetch origin | 通过；起点干净，与 origin/main 一致 |
| T01 pnpm install --frozen-lockfile、cargo fmt/check --locked、pnpm build | 均 0 |
| T01 pnpm dev 及 computer-use 观察 | 实际窗口显示桌面连接已建立、Lyreniq 0.1.0 · windows；恢复通过后才实施 T02 |
| 在 D:\ 调用 scripts/check-env.ps1 | PowerShell 7、5.1 均 0，检查全部工具和 WebView2；不安装 |
| 在 D:\ 调用 scripts/bootstrap.ps1 | 0；frozen pnpm 安装和 cargo fetch --locked --target x86_64-pc-windows-msvc 通过 |
| scripts/test.ps1 | PowerShell 7/5.1 均 0：脚本语法、cargo fmt/clippy -D warnings、2 项 Rust IPC、前端 lint/types、3 项 Vitest 通过 |
| Invoke-Checked cmd.exe /d /c exit 7 | 预期失败被捕获，验证命令退出码没有被日志管线吞掉 |
| scripts/build.ps1（含在 D:\ 经 powershell.exe -NoProfile -File 调用） | 最终清单配置通过；产物 target/release/lyreniq-desktop.exe，非安装包 |
| powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\dev.ps1 | Vite ready、cargo run --locked、原生 exe 已运行；最终可视窗口检查中断，不能记作最终 GUI 通过 |

真实失败及修复：首次 Rust 测试返回 0xc0000139/STATUS_ENTRYPOINT_NOT_FOUND，test.ps1 正确返回 1；直接加 manifest 又出现 LNK1123 重复资源。最终将 manifest 统一后，2 项 Rust 测试和 release 构建通过。初选的已弃用 matcher 已改用与 Vitest 4 相容的 jest-dom 7.0.1。没有降低断言、关闭 lint 或跳过失败测试。

最终日志：.local/logs/test.log、build.log、bootstrap.log、dev.log。test 日志已包含 2+3 项测试摘要与命令退出码；原生 stderr 仍可在控制台查看，日志含本机路径而不提交。

## Git 和 CI

- 授权远端：https://github.com/Hcreativelf/Lyreniq.git（用户创建的公开仓库，本轮未改可见性）。
- T02-a 是独立本地验收通过的子任务，提交标题 build: add reproducible scripts and CI baseline；T02-b 未通过，不标父任务完成。
- 本记录写在提交/push 前；最终 hash、push 以及 GitHub CI 实际结果由本轮最终报告提供，下一轮从 git log 和 Actions 按 SHA 回填。
- push 后核实 git ls-remote --heads origin main 等于本地 HEAD，再查看该 SHA 对应的 Windows checks；不得把 CI 配置存在视为 CI 通过。
- 不强推、不改写历史；如远端有新提交或推送失败，保留本地成果并如实处理。

## 未完成与恢复步骤

- 唯一下一任务：**T02-b — 统一 dev 脚本原生窗口复验与 T02 收尾**。在此之前不执行 T03。
- 下一轮先读取 AGENTS、README、主任务书、STATUS/HANDOFF、DECISIONS、TESTING、ENVIRONMENT；检查实际 Git/分支/remote，fetch 并核对 T02-a 最终提交和 CI 结果。
- 复跑 scripts/check-env.ps1、bootstrap.ps1、test.ps1、build.ps1，检查两份锁文件、原任务书 SHA-256 和缓存忽略规则。
- 启动 scripts/dev.ps1，实际观察原生窗口、runtime_label、重新验证按钮，再验证退出/清理并保存脱敏证据。只有启动日志和 mock 测试不足以通过这一项。
- 若桌面工具再次报告用户停止，停止桌面操作；保持 T02-b 待验。验证通过后更新父任务 T02 为 VERIFIED，再提交/推送并选择 T03。
- 当前未实现或验证音频/调音 UI/主题/DPI 矩阵/文件/设置/安装包/生产离线；项目许可证与正式分发标识未定。
