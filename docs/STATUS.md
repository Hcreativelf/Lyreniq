# Lyreniq 项目状态

- 更新日期：2026-11-25（Asia/Shanghai）
- 当前里程碑：M2 DSP 核心
- 当前任务：T07 — YIN 鲁棒性与运行性能
- 状态：VERIFIED（本地固定夹具/单机预算）；本轮 commit/push/CI 在最终报告给出，下一轮按 SHA 回填
- 最近通过：T07；T00–T06 保持通过
- 当前分支：main，跟踪 origin/main
- 起点 commit：58830d6536561526af734ee55ecb201a6984225b（T06，已推送）
- 上轮 CI：[T06 Windows checks](https://github.com/Hcreativelf/Lyreniq/actions/runs/45583704093) 按 SHA 核实 success
- 新增：提前搜索/差分累加优化、泛音周期验证、四次谷值拟合、固定噪声夹具；T06 参考保留；原始检测与三帧中值展示分离
- 精度：130 纯音窗口、40 泛音窗口通过 ≤1 cent；20/5000 Hz 已测边界修复；固定 20/10 dB 接受帧通过 5/10 cents，0 dB 返回无音高
- 覆盖率：有已知目标的 280 窗口中 T07 接受 250（89.2857%），其余 30 为 0 dB；泛音子集八度/粗大错误 0/40；不是所有乐器或连续频段保证
- 性能：i5-14600KF Release 最坏已测 p95=3.5552 ms，检测核心满足 33.3 ms 预算；完整实时管线尚未实现/实测
- 验证：4 核心单元+33 集成+2 文档、2 桌面 Rust、26 TS、项目外五脚本/release/后台 16 工作区与 44 Demo；新基准脚本实际通过 Windows PowerShell 5.1
- 证据：[YIN_VALIDATION](YIN_VALIDATION.md)、[T07 摘要](evidence/T07/result.json)、TESTING
- 环境：零外部核心依赖，两份锁文件未变；沿用集中缓存，无新工具或权限
- 设备：用户报告本机无麦克风，本轮未枚举/采集；合成验证不受影响，真实输入及无设备分支留给 T09
- UI：仍为 T04 显式 Demo，尚未链接核心，不能测量真实乐器
- 未实现：FFT 频谱、设备采集、实时 worker/IPC、真实 UI 接入、文件分析、参考音、设置持久化、安装包
- 未验证：更多噪声/音色/非谐和乐器、真实设备时钟与稳定性、完整管线预算、原生 DPI/标题栏、生产离线和安装
- GitHub：https://github.com/Hcreativelf/Lyreniq.git；许可证/正式分发标识/签名待定
- 唯一下一任务：T08 — FFT 频谱核心（本轮未开始）

| ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00 | VERIFIED | 文档/Git/远端 | 562cfadb1b818b1f04b19d05c114b9cba211c5fd / main |
| T01 | VERIFIED | 最小窗口/桥接 | 0ba2701de101f39f6205b0836127969a773cf456 / main |
| T02-a | VERIFIED | 脚本/测试/release/CI | def1765d43745d9b310fbeba511ff257ee140f9a / main |
| T02-b / T02 | VERIFIED | 后台窗口/IPC/退出/CI | 1355bd94121e4bd8dec8b82e15dee765f8d4106e / main |
| T03 | VERIFIED | 主题/导航/回归/CI | 3c6779919ff169f874d074baac3a7578673a8f76 / main |
| T04 | VERIFIED | 调音显示/Demo/回归/CI | 784149b18a36ec51f3a0938460e98b9719cb0768 / main |
| T05 | VERIFIED | 音律/模型/17+1 核心/回归/CI | af37a77a126d4b90e0efede36c1b83b315ffca31 / main |
| T06 | VERIFIED | 基础 YIN/27+2 核心/回归/远端/CI | 58830d6536561526af734ee55ecb201a6984225b / main |
| T07 | VERIFIED（本地固定范围） | 37+2 核心、误差/覆盖率/CPU/分位耗时、完整回归 | 本轮提交后报告 hash / main |
| T08 | NOT_STARTED | 唯一下一项，先复验 T07 | 无 |

每项独立验收、更新交接并核实 Git。后台 UI 不前置窗口或注入系统输入；合成音准、Demo 与真实设备证据分开记录。
