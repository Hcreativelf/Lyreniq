# Lyreniq 交接记录

## 阶段与起点

- 2026-11-27（Asia/Shanghai），`D:\2-workspace\Lyreniq`；`main` 跟踪 `origin/main`。
- 起点 `908ecc72e5e33e425d23b059db8395905784ee88`（P1，已推送核实，工作区干净）。
- 本轮完成 **P2 — 流式解码与离线文件分析引擎**，新增 workspace 成员 `crates/audio-decode` 与 `crates/tuner-engine` 以及三格式合成测试夹具。

## 本轮交付

- **`crates/audio-decode`**（`#![forbid(unsafe_code)]`）：
  - 引入并锁定 `symphonia = "=0.5.5"`（仅启用 `wav/pcm/adpcm/flac/mp3/opt-simd`，MPL-2.0，见 D019）。
  - 实现 `AudioDecoder::open_path` / `open_bytes` 与逐包流式 `next_chunk()`：自动回退内容探测（修复错扩展名）、清洗/限幅解码样本至有限 `[-1, 1]`（保护有损 MP3 帧间峰值）、支持 `ChannelMode::MixMono` / `Channel(idx)`、双声道反相抵消告警（`phase_cancellation_warning`）与畸形/截断/缺失文件安全错误处理。
  - 新增 `examples/gen_fixtures.rs` 与 `tests/decode.rs`（3 项集成测试）。
- **`crates/tuner-engine`**（`#![forbid(unsafe_code)]`）：
  - 实现恒定内存滑窗 `FileAnalyzer::analyze_path` / `analyze_decoder`：复用 DSP 2.0 `MultiScalePitchDetector` / `YinDetector`、精确帧中心时间戳、单调进度报告、原子取消（`AtomicBool`）、`AnalysisSummary` 诚实统计（绝不把静音计入调准）与 `export_timeline_csv` / `export_timeline_csv_to_path`（含 `#` 配置元数据、不泄露本机路径、静音留空、支持中文路径）。
  - 新增 `tests/file_analysis.rs`（5 项集成测试，覆盖 WAV/FLAC/MP3 三格式、分段静音、中途取消、A4=442 校准、中文路径 CSV 导出与 60 秒长文件流式回归）。
- **`tests/fixtures/`**：
  - 100% 合成、无版权风险的三格式 1 秒 440 Hz 夹具：`tone_440_48k.wav`、`tone_440_44k1.flac`、`tone_440_48k.mp3` 及 [README](file:///d:/2-workspace/Lyreniq/tests/fixtures/README.md)。

## 实际验收与命令

| 命令 | 实际结果 |
|---|---|
| `cargo test -p audio-decode -p tuner-engine --release --locked` | 退出 0；3 项解码 + 5 项离线分析测试全通过，60 秒 44.1 kHz 长流式分析在内总耗时 **0.06 s** |
| `powershell.exe -NoProfile -File scripts/test.ps1` | 退出 0；PS/MJS/Biome、Rust fmt/clippy（零警告）、59 项 Rust 测试 + 26 项前端 TS 测试全通过 |

## 唯一下一阶段

**P3 — 音频 I/O 与实时管线（`crates/audio-io` + `tuner-engine` `LiveSession`）。**

1. 核对 Git 状态并复跑 `cargo test --workspace --locked`。
2. 新建 `crates/audio-io`：引入并锁定 CPAL 与无锁 SPSC 环形缓冲（`rtrb` 或 `ringbuf`），实现麦克风枚举/采集、可选 Windows WASAPI Loopback 系统内录、CPAL 淡入淡出参考音输出（驱动 `tuner_core::ToneGenerator`）以及虚拟实时流源（文件/合成音按真实采样时钟推入环形缓冲）。
3. 在 `crates/tuner-engine` 实现 `LiveSession`：单活会话 ID、实时 DSP Worker（消费环形缓冲、按步长跑 `MultiScalePitchDetector` + `PitchTracker` + `SpectrumAnalyzer`、过载丢旧帧统计 `dropped_frames`）与状态生命周期测试。
4. 用虚拟实时流执行持续实时压测与会话隔离测试，记录真实硬件条件（无麦克风时的无设备恢复与可选 Loopback/输出实测），提交并推送。
