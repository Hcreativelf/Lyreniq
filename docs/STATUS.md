# Lyreniq 项目状态

- 更新日期：2026-11-24（Asia/Shanghai）
- 当前里程碑：M1 UI 基础
- 当前任务：T03 — 现代 UI 设计系统与工作区壳
- 状态：VERIFIED（本地）；本轮提交/push 和 CI 按实际结果在最终报告提供，下一轮回填
- 最近通过验收：T03；T00–T02 保持通过
- 当前分支：main，跟踪 origin/main
- 起点 commit：1355bd94121e4bd8dec8b82e15dee765f8d4106e（T02-b，已推送）
- 上轮 CI：[T02-b Windows checks](https://github.com/Hcreativelf/Lyreniq/actions/runs/45345059150) 按 SHA 核实 success
- 当前能力：四工作区导航、深浅主题、统一基础组件、中文文案、真实桌面连接状态；Windows 原生最小 1024×700，默认 1200×700
- 已通过：项目外 PS 5.1 全套环境/冻结安装/检查/2 Rust + 6 TS/release/后台 UI；16 个真实窗口尺寸/主题/页面组合、键盘焦点、18 组文字对比度、正常退出及端口释放
- UI 证据：[UI_GUIDE](UI_GUIDE.md)、[截图与结果](evidence/T03/result.json)；后台窗口始终 hidden/unfocused
- 缓存：已按本机偏好集中当前发现的 pnpm/npm/Cargo registry/Vite/Vitest 缓存并验证；原路径链接及跨机器回退见 ENVIRONMENT
- 尚未实现：调音显示与 Demo、DSP、音频采集、文件分析、参考音、设置持久化及安装包
- 尚未验证：Windows 原生 DPI/跨屏、标题栏实体交互、设备、生产断网与安装；125%/150% 目前仅渲染器模拟
- GitHub：https://github.com/Hcreativelf/Lyreniq.git；自身许可证与正式分发标识待确定
- 唯一下一任务：T04 — 调音器 UI 与状态模型（本轮未开始）

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档、Git 初始化、远端 hash | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED | 最小原生窗口、真实 Rust 桥接 | 0ba2701de101f39f6205b0836127969a773cf456 / main |
| T02 | VERIFIED | T02-a 工程回归/CI + T02-b 后台原生 smoke | 两个独立子任务 |
| T02-a | VERIFIED | 脚本、测试、release、CI success | def1765d43745d9b310fbeba511ff257ee140f9a / main |
| T02-b | VERIFIED | 后台 GUI、按钮 IPC、关闭；CI success | 1355bd94121e4bd8dec8b82e15dee765f8d4106e / main |
| T03 | VERIFIED（本地） | 2 Rust + 6 TS、release、16 GUI 组合、焦点/对比度 | 本轮提交后报告 hash / main |
| T04 | NOT_STARTED | 唯一下一任务；先复验 T03 | 无 |

用户允许逐项自主推进；每项仍须复验、独立验收、交接和 Git 核实。用户同时使用桌面，默认后台 UI，不前置窗口或注入系统输入。
