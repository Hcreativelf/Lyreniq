# Lyreniq 项目状态

- 更新日期：2026-11-24（Asia/Shanghai）
- 当前里程碑：M1 UI 基础完成；下一项为 M2 DSP 核心
- 当前任务：T04 — 调音器 UI 与状态模型
- 状态：VERIFIED（本地）；本轮提交/push 与 CI 在最终报告给出，下一轮按 SHA 回填
- 最近通过验收：T04；T00–T03 保持通过
- 当前分支：main，跟踪 origin/main
- 起点 commit：3c6779919ff169f874d074baac3a7578673a8f76（T03，已推送）
- 上轮 CI：[T03 Windows checks](https://github.com/Hcreativelf/Lyreniq/actions/runs/45387529930) 按 SHA 核实 success
- 当前能力：四工作区/深浅主题、调音展示与六种信号状态、统一 TS 展示适配器、11 个主动启用且明确标注的 Demo 场景；默认没有测量值
- 已通过：项目外 Windows PowerShell 5.1 的环境/冻结安装/检查/2 Rust + 26 TS/release/后台 UI；16 工作区与 44 Demo 组合、±50/±75 指针/数值边界、清理/重试/焦点、退出与端口释放
- UI 证据：[UI_GUIDE](UI_GUIDE.md)、[T04 截图与结果](evidence/T04/result.json)；原生窗口保持 hidden/unfocused，未注入系统鼠标键盘
- 环境：没有新增依赖、系统组件或缓存设置；本机集中缓存链接复核正常，详情见 ENVIRONMENT
- 尚未实现：DSP/音律核心、真实音频输入、频谱、文件分析、参考音输出、设置持久化与安装包
- 尚未验证：真实音准/设备、原生 DPI/跨屏/标题栏、生产断网与安装；125%/150% 只是渲染器模拟
- GitHub：https://github.com/Hcreativelf/Lyreniq.git；自身许可证、正式分发标识与签名仍未决定
- 唯一下一任务：T05 — 音律与 DSP 数据模型（本轮未开始）

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档、Git 初始化、远端 hash | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED | 最小原生窗口与桥接 | 0ba2701de101f39f6205b0836127969a773cf456 / main |
| T02 | VERIFIED | 工程回归/CI 与后台 GUI | 两个独立子任务 |
| T02-a | VERIFIED | 脚本、测试、release、CI | def1765d43745d9b310fbeba511ff257ee140f9a / main |
| T02-b | VERIFIED | 后台 GUI、真实 IPC、关闭、CI | 1355bd94121e4bd8dec8b82e15dee765f8d4106e / main |
| T03 | VERIFIED | 主题/导航壳、工程回归、CI | 3c6779919ff169f874d074baac3a7578673a8f76 / main |
| T04 | VERIFIED（本地） | 26 TS、release、44 Demo 与原有回归 | 本轮提交后报告 hash / main |
| T05 | NOT_STARTED | 唯一下一任务；先复验 T04 | 无 |

用户允许逐项自主推进；每项仍须先复验、独立验收、交接及 Git 核实。UI 默认后台测试，不前置窗口或注入系统输入。Demo 不是实际麦克风或算法精度结果。
