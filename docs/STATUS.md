# Lyreniq 项目状态

- 更新日期：2026-11-24（Asia/Shanghai）
- 当前里程碑：M0 工程基础
- 当前任务：T02-a — 环境脚本、自动化测试与 CI 基线
- 最近通过本地验收的任务：T02-a；T02 父任务仍为 IN_PROGRESS
- 状态：T02-a VERIFIED（本地）；CI 实际运行在 push 后核实，不由配置存在推断
- 当前分支：main，跟踪 origin/main
- 起点 commit：0ba2701de101f39f6205b0836127969a773cf456（T01）
- 已支持平台：本机 Windows x64 最小窗口/工程基线；尚非可用调音器或安装包
- 已通过：T01 恢复检查及实际窗口；项目外运行 check-env/bootstrap；PowerShell 7/5.1 的 fmt/clippy/lint/typecheck、2 项 Rust IPC 和 3 项前端测试；release 构建；原生失败退出码传播
- 尚未验证：T02-b 的最终统一 dev 脚本窗口/按钮/退出；CI 实际结果需 push 后查验；音频、主题、文件、设置、安装与生产离线验收均未实现
- GitHub：https://github.com/Hcreativelf/Lyreniq.git；当前提交/推送 hash 由本轮报告提供，不写入提交自身
- 关键情况：最终桌面复验被物理 Esc 中断，已停止桌面控制；父任务不得标完成。项目许可证和正式分发标识待确定
- 唯一下一任务：T02-b — 统一 dev 脚本原生窗口复验与 T02 收尾

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档、Git 初始化、远端 hash 核实 | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED | 本机最小窗口与桥接；本轮再次复验通过 | 0ba2701de101f39f6205b0836127969a773cf456 / main |
| T02 | IN_PROGRESS | 拆分原因与验收见 DECISIONS D009 | T02-a + T02-b |
| T02-a | VERIFIED（本地） | 脚本、2+3 测试、release；CI 结果在 push 后核实 | 本轮提交后报告 hash / main |
| T02-b | NOT_STARTED | 最终真实窗口、按钮与退出证据待补齐；不以 mock/启动日志替代 | 唯一下一任务 |
| T03 | NOT_STARTED | T02-b 完成前不进入 | 无 |

用户允许额度可用时逐项自主推进；仍保持复验、独立验收、交接、提交并核实推送。公共工具链允许共享安装，项目配置与锁文件保留在仓库。
