# Lyreniq 项目状态

- 更新日期：2026-11-24（Asia/Shanghai）
- 当前里程碑：M0 工程基础
- 当前任务：T01 — Windows 工具链与最小窗口
- 最近通过验收的任务：T01（本地验收）
- 状态：VERIFIED（提交/推送结果以 Git 日志和交付报告为准）
- 当前分支：main
- 起点 commit：562cfadb1b818b1f04b19d05c114b9cba211c5fd
- 已支持平台：仅本机 Windows x64 最小窗口；不是完整调音软件或安装包
- 已通过检查：T00 基线复验及补推；版本输出、frozen-lockfile 安装、cargo fmt、cargo check --workspace --locked、TypeScript/Vite build、真实 Tauri 窗口和 runtime_label IPC/按钮
- 尚未验证：单元测试/lint/CI（T02 建立）；音频、文件、设置、主题、安装和生产离线验收（后续任务）
- GitHub：https://github.com/Hcreativelf/Lyreniq.git；main 已跟踪 origin/main
- 关键待办：完成当前提交/推送；MSVC 安装器曾返回 3010，后续 vswhere 无重启标记，未重启即通过编译/窗口；项目许可证待确定
- 下一项任务：T02 — 环境脚本与回归基线

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档与 Git 复验通过；推送并核实远端 main hash 相同 | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED（本地） | 锁文件、构建及真实窗口/IPC 验收通过；截图见 docs/evidence/T01 | main；起点为 T00 提交；本轮 hash 提交后报告 |
| T02 | NOT_STARTED | 唯一下一任务；尚未建立脚本或 CI | 无 |

用户已允许额度可用时逐项自主推进，且允许必需工具链共享安装。仍按任务边界复验、验收、交接、提交并核实远端，不跳步。
