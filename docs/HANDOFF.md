# Lyreniq 交接记录

## 阶段与起点

- 2026-11-27（Asia/Shanghai），`D:\2-workspace\Lyreniq`；`main` 跟踪 `origin/main`。
- 起点 `0db1504`（T07），工作区干净，`git fetch origin` 后 ahead/behind = 0/0；先复跑 `cargo test --workspace --locked` 确认基线（37 核心 + 2 文档 + 2 桌面 Rust 全部通过）。
- 经用户明确批准：保留根目录冻结原计划书不变，将 `docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 升级为 **v2.0**（四层架构 + P1–P8 垂直路线，含 WASAPI Loopback 可选输入源），并完成 **P1 — DSP 2.0 核心重构**。

## 本轮交付

- **计划与决策**：
  - `docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 全面修订为 v2.0；根目录冻结原件未动。
  - `docs/DECISIONS.md` 新增 D018 记录架构升级、DSP 2.0 重写与 WASAPI Loopback 决策。
  - `docs/TUNING_CORE.md`、`README.md`、`docs/STATUS.md`、`docs/TESTING.md` 同步更新。
- **DSP 2.0 核心（`crates/tuner-core`）**：
  - 引入并精确锁定 `rustfft = "=6.4.1"`、`serde = "=1.0.229"`（dev 依赖 `serde_json = "=1.0.154"`），继续保持 `#![forbid(unsafe_code)]`。
  - `src/spectrum.rs`：复用 FFT 缓冲的 Hann 窗幅频谱分析器（窗口加权去 DC、精确相干增益 dBFS、谐波峰值查询、低带宽对数频率分桶）。
  - `src/yin.rs`：重写为 FFT 加速 Fast YIN（复数打包单次正 FFT + 逆 FFT，$O(N \log N)$ 恒定最坏复杂度）、谐波感知 2x/3x 倍周期验证与五点四次细化；新增 `MultiScalePitchDetector`（85 ms 快速中高音路径 + 31 抽头 Hamming-sinc 4x 降采样低音路径）；移除生产 API `detect_reference`。
  - `src/smoothing.rs`：升级为对数频率（音分）域 `PitchTracker`（3 帧中值 + 自适应 EMA + 可配置 `hold_frames` 瞬时 `LowConfidence` 防闪烁 + 静音/跳变/错误立即重置）。
  - `src/profile.rs`：10 种内置乐器预设（半音/吉他标准与 Drop D/4 与 5 弦贝斯/尤克里里/提琴族/钢琴）与自定义目标校验。
  - `src/synth.rs`：零分配、相位连续 `ToneGenerator`（正弦/柔和泛音、安全默认低增益 0.15、15 ms 防爆音淡入淡出与频率平滑）。
  - `src/model.rs` / `src/tuning.rs`：统一 `camelCase` 序列化契约（`TunerSnapshot`、`TuningResult` 拆分 `pitchClass`/`octave`、`MidiNote` 反序列化边界校验）。
  - 新增 `tests/spectrum.rs`、`tests/profiles_and_synth.rs`，升级 `tests/yin_robustness.rs`、`tests/smoothing.rs`、`examples/yin_report.rs` 与 `scripts/bench-yin.ps1`。

## 实际验收与命令

| 命令 | 实际结果 |
|---|---|
| `cargo fmt --all -- --check` | 退出 0，格式通过 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 退出 0，零警告 |
| `cargo test -p tuner-core --release --locked` | 退出 0，4 单元 + 43 集成 + 2 文档（共 49 项核心测试）全部通过，`yin_robustness` 640 帧矩阵仅 0.08 s |
| `cargo run -p tuner-core --example yin_report --release --locked -- --benchmark-only` | 退出 0；48 kHz/300 ms `FastYIN` 最坏 p95 = 0.156 ms，`MultiScale` 中高音 0.037–0.043 ms / 低音与纯噪声 0.091–0.110 ms，`Spectrum_4096` 0.056 ms |
| `powershell.exe -NoProfile -File scripts/test.ps1` | 退出 0；PS/MJS/Biome、Rust fmt/clippy/test（51 项）、前端 lint/typecheck/test（26 项）全部通过 |

## 已知限制与未验证项

- 本轮仅完成纯 Rust 核心重构，尚未接入真实音频文件解码、CPAL 设备或重构前端 UI。
- 真实乐器声学录音、真机麦克风/WASAPI 设备、原生 DPI/标题栏、生产离线与 Windows 安装包留待 P2–P8 逐项实测。

## Git 状态

- 已授权远端 `https://github.com/Hcreativelf/Lyreniq.git`，提交建议 `feat(core): upgrade to DSP 2.0 and architecture plan v2.0`，最终 SHA 与远端核实在本轮交付报告给出。

## 唯一下一阶段

**P2 — 流式解码与离线文件分析引擎。**

1. 核对 Git 状态并复跑 `cargo test --workspace --locked` 与 `scripts/test.ps1`。
2. 新建 `crates/audio-decode`（引入并锁定 Symphonia WAV/FLAC/MP3 解码器，处理采样率、声道混合/单声道选择与反相检测、损坏/截断文件错误保护）。
3. 新建 `crates/tuner-engine`（平台无关应用编排层）并先落地离线 `FileAnalyzer`：恒定内存滑窗调用 DSP 2.0 核心、进度报告、原子取消、有效帧统计摘要与 CSV + 配置元数据导出。
4. 编写合成与真实音频特征夹具集成测试，验证长文件内存有界、取消生效与 CSV 严格一致，提交并推送。
