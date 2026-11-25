# Lyreniq 项目状态

- 更新日期：2026-11-25（Asia/Shanghai）
- 当前里程碑：M2 DSP 核心
- 当前任务：T05 — 音律与 DSP 数据模型
- 状态：VERIFIED（本地）；本轮 commit/push 与 CI 在最终报告给出，下一轮按 SHA 回填
- 最近通过验收：T05；T00–T04 保持通过
- 当前分支：main，跟踪 origin/main
- 起点 commit：784149b18a36ec51f3a0938460e98b9719cb0768（T04，已推送）
- 上轮 CI：[T04 Windows checks](https://github.com/Hcreativelf/Lyreniq/actions/runs/45405643741) 按 SHA 核实 success
- 新增能力：零外部依赖 tuner-core，音律/A4/目标映射、校验数据模型和有界正弦/谐波 PCM 夹具；尚未链接桌面
- 已通过：17 核心集成 + 1 文档测试；既有 2 桌面 Rust + 26 TS；项目外五脚本回归、release、16 工作区/44 Demo 后台组合
- 证据：[TUNING_CORE](TUNING_CORE.md)、[T05 结果](evidence/T05/result.json)、TESTING；原 UI/Demo 未改动
- 设备条件：用户说明本机没有麦克风，未做枚举/采集；当前纯核心/合成测试不受影响；T09 应验证实际设备列表与无可用输入分支
- 环境：工具版本、集中缓存及链接沿用；Cargo.lock 仅增加本地核心包，无外部依赖升级
- 尚未实现：YIN、FFT、真实音频输入/分析、核心 UI 桥接、文件分析、参考音输出、设置持久化和安装包
- 尚未验证：实际检测精度/设备、原生 DPI/跨屏/标题栏、生产断网和安装；既有比例模拟不代替系统实测
- GitHub：https://github.com/Hcreativelf/Lyreniq.git；项目许可证/正式分发标识/签名待定
- 唯一下一任务：T06 — 基础 YIN 单音算法（本轮未开始）

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档/Git/远端 | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED | 最小窗口/桥接 | 0ba2701de101f39f6205b0836127969a773cf456 / main |
| T02 | VERIFIED | 工程基线/CI/后台 UI | 两个独立子任务 |
| T02-a | VERIFIED | 脚本/测试/release/CI | def1765d43745d9b310fbeba511ff257ee140f9a / main |
| T02-b | VERIFIED | 后台窗口/IPC/退出/CI | 1355bd94121e4bd8dec8b82e15dee765f8d4106e / main |
| T03 | VERIFIED | 主题/导航/回归/CI | 3c6779919ff169f874d074baac3a7578673a8f76 / main |
| T04 | VERIFIED | 调音显示/显式 Demo/回归/CI | 784149b18a36ec51f3a0938460e98b9719cb0768 / main |
| T05 | VERIFIED（本地） | 核心 17+1 测试、全项目回归 | 本轮提交后报告 hash / main |
| T06 | NOT_STARTED | 唯一下一项，先复验 T05 | 无 |

任务仍须逐项复验、独立验收、更新交接和 Git 核实。UI 默认后台，不前置窗口或注入系统输入。Demo/合成信号不是实际采集或检测精度结果。
