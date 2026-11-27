# Lyreniq 项目状态

- 更新日期：2026-11-27（Asia/Shanghai）
- 现行计划：v2.0（四层架构 + P1–P8 垂直闭环，见 D018 与 `docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md`）
- 当前阶段：P2 — 流式解码与离线文件分析引擎
- 状态：VERIFIED（WAV/FLAC/MP3 流式解码、离线单音分析、取消/进度、统计与 CSV 导出全通过；本轮 commit/push 在最终报告给出，下轮回填 SHA）
- 最近通过：P2；P1 与既有基线 T00–T07 保持完整通过
- 当前分支：main，跟踪 origin/main
- 起点 commit：908ecc72e5e33e425d23b059db8395905784ee88（P1，已推送核实）
- 本轮新增：
  - `crates/audio-decode`：锁定 `symphonia = "=0.5.5"`（仅启用 `wav/pcm/adpcm/flac/mp3/opt-simd`），流式逐包解码、错扩展名内容探测回退、有损 MP3 峰值限幅清洗、单声道选择与双声道反相抵消检测、畸形/截断/缺失文件安全错误处理
  - `crates/tuner-engine`：平台无关离线 `FileAnalyzer`（恒定内存滑窗、精确帧中心时间戳、单调进度、原子取消、`AnalysisSummary` 诚实统计不把静音计入调准）与 `export_timeline_csv`（含 `#` 配置元数据头、无私有路径泄漏、静音留空、支持中文文件名）
  - `tests/fixtures/`：100% 合成、无版权风险的三格式测试夹具（`tone_440_48k.wav`、`tone_440_44k1.flac`、`tone_440_48k.mp3` 及生成说明 `README.md`）
- 性能（Release）：全部 5 项 `tuner-engine` 测试（含完整 60 秒 44.1 kHz 流式分析 ~1,200 帧）总耗时 **0.06 秒**
- 验证：59 项 Rust 测试（47 核心 + 2 文档 + 3 解码 + 5 引擎 + 2 桌面）+ 26 项前端 TS 测试 + fmt/clippy/lint/typecheck/PS AST 全通过
- 未实现：P3（CPAL 麦克风 / WASAPI Loopback / 参考音 / 虚拟实时流 + `LiveSession`）、P4–P8
- 未验证：真实乐器实录、真机麦克风/声卡设备、Windows 原生 DPI/标题栏、生产离线与安装包
- 唯一下一阶段：**P3 — 音频 I/O 与实时管线**

| 阶段/ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00–T07 | VERIFIED（历史基线） | 工程脚本、桌面壳、音律数学、640 帧合成基线 | 562cfad..0db1504 / main |
| P1 | VERIFIED | v2.0 计划、DSP 2.0 47+2 核心测试、Release 基准 | 908ecc72e5e33e425d23b059db8395905784ee88 / main |
| P2 | VERIFIED | `audio-decode` + `tuner-engine` 离线分析与 CSV、WAV/FLAC/MP3 夹具、59 Rust + 26 TS | 本轮提交后报告 hash / main |
| P3 | NOT_STARTED | 唯一下一阶段：`audio-io`（CPAL 麦克风 + WASAPI Loopback + 参考音 + 虚拟流）+ `LiveSession` | 无 |
| P4 | NOT_STARTED | 极薄 `src-tauri` 桥接、Tauri Channel、本地设置持久化与契约测试 | 无 |
| P5 | NOT_STARTED | 实时调音与频谱工作台 UI 重构及真实后端接入 | 无 |
| P6 | NOT_STARTED | 文件分析工作台 UI（导入/时间轴/统计/CSV 导出） | 无 |
| P7 | NOT_STARTED | 参考音与设置页面完整落地 | 无 |
| P8 | NOT_STARTED | 稳定性压测、离线隐私审查与 Windows 安装包发布 | 无 |
