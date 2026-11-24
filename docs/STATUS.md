# Lyreniq 项目状态

- 更新日期：2026-11-24（Asia/Shanghai）
- 当前里程碑：M0 工程基础已通过；下一项为 M1 UI 基础
- 当前任务：T02-b — 后台原生 UI 复验与 T02 收尾
- 最近通过验收的任务：T02-b；T02 父任务 VERIFIED
- 当前分支：main，跟踪 origin/main
- 起点 commit：def1765d43745d9b310fbeba511ff257ee140f9a（T02-a，已推送）
- 上轮 CI：T02-a Windows checks 实际 success，[运行记录](https://github.com/Hcreativelf/Lyreniq/actions/runs/45338108012)
- 已通过：项目外环境检查、冻结依赖安装、脚本语法/lint、Rust fmt/clippy、2 项 Rust IPC 和 3 项前端测试、release 构建；隐藏的真实 Tauri/WebView2 页面、Rust IPC、重连按钮、正常关闭与端口释放
- 后台 UI 实测：1000×700；原生 visible=false、focused=false；未注入系统鼠标键盘；[截图](evidence/T02-b/native-window.png)和[结果](evidence/T02-b/result.json)
- 尚未验证：标题栏实体交互、DPI 矩阵、设备权限与音频；调音、主题、文件、设置、安装及生产离线功能尚未实现
- 当前提交/push 和本轮 CI：本文件写于提交前，最终 hash、远端核实与 CI 实际结果由交付报告提供，下一轮按 SHA 回填；不由 CI 配置存在推断通过
- GitHub：https://github.com/Hcreativelf/Lyreniq.git
- 关键约束：用户同时使用桌面，默认后台 UI 测试，不前置窗口或注入系统输入；许可证与正式分发标识待确定
- 唯一下一任务：T03 — 现代 UI 设计系统与工作区壳（本轮未开始）

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档、Git 初始化、远端 hash 核实 | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED | 最小原生窗口与真实 Rust 桥接 | 0ba2701de101f39f6205b0836127969a773cf456 / main |
| T02 | VERIFIED | T02-a 工程回归及 CI + T02-b 后台原生 smoke | 两个独立子任务 |
| T02-a | VERIFIED | 脚本、2+3 测试、release、Windows CI success | def1765d43745d9b310fbeba511ff257ee140f9a / main |
| T02-b | VERIFIED（本地） | 真实隐藏窗口、按钮 IPC、截图、正常退出；见 TESTING | 本轮提交后报告 hash / main |
| T03 | NOT_STARTED | 下一项；开始前先复验 T02-b | 无 |

用户允许额度可用时逐项自主推进；每项仍须复验、独立验收、交接、提交并核实推送。公共工具链允许共享安装，项目配置与锁文件保留在仓库。
