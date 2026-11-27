# Lyreniq DSP 2.0 核心与数据契约（`crates/tuner-core`）

日期：2026-11-27（Asia/Shanghai）；对应架构 v2.0 阶段 **P1**。实现位于 `crates/tuner-core`，`#![forbid(unsafe_code)]`，仅依赖纯 Rust 的 `rustfft`（`=6.4.1`）与 `serde`（`=1.0.229`），完全独立于 Tauri、设备与 OS I/O，可直接面向原生桌面与未来 WASM 复用。

## 模块总览

| 模块 | 核心类型/函数 | 职责 |
|---|---|---|
| `tuning` | `MidiNote`, `TuningConfig`, `TuningResult`, `TargetSelection`, `cents_difference` | 十二平均律、A4 校准（400–480 Hz）、音分差、方向判定与完整/拆分音名（`pitchClass` + `octave`） |
| `profile` | `InstrumentProfile`, `InstrumentProfileId`, `DetectionMode` | 10 种内置乐器预设（半音、吉他标准/Drop D、4/5 弦贝斯、尤克里里、小/中/大提琴、钢琴）与自定义目标校验 |
| `spectrum` | `SpectrumAnalyzer`, `SpectrumConfig`, `SpectrumFrame`, `LogSpectrum`, `HarmonicPeak` | 复用 `rustfft` 缓冲的 Hann 窗幅频谱、窗口加权去 DC、相干增益 dBFS、谐波峰值查询与对数频率轴分桶 |
| `yin` | `YinDetector`, `YinConfig`, `PitchDetection`, `MultiScalePitchDetector` | FFT 加速 Fast YIN（$O(N \log N)$ 恒定最坏复杂度）、谐波感知倍周期防八度错判、双尺度（85 ms 快速路径 + 4x 抗混叠降采样低音路径） |
| `smoothing` | `PitchTracker`, `PitchTrackerConfig`, `TrackedPitch` | 对数频率（音分）域 3 帧中值 + 自适应 EMA + 可配置瞬时 `LowConfidence` 保持（`hold_frames`）+ 静音/跳变立即复位 |
| `synth` | `ToneGenerator`, `ToneWaveform`, `DEFAULT_SAFE_TONE_GAIN` | 零分配、相位连续参考音与测试音发生器（纯正弦/柔和泛音，15 ms 防爆音淡入淡出与频率平滑） |
| `model` | `FrequencyRange`, `PitchEstimate`, `PitchFrame`, `PitchStatus`, `TunerSnapshot` | 统一可序列化（`camelCase` JSON）数据契约，强制有限值与非活跃态清理不变量 |
| `test_signal` | `sine`, `harmonic_tone`, `white_noise`, `Harmonic` | 有界、确定性合成测试信号生成器（内存内，不发声） |

## 关键算法约定

### 1. FFT 加速 Fast YIN（`YinDetector`）
- 按 Wiener–Khinchin 恒等式计算所有 lag 的 YIN 差分函数：
  $$d(\tau) = E_0 + E(\tau) - 2 R(\tau)$$
  其中 $E_0, E(\tau)$ 由前缀平方和 $O(1)$ 查表获得，$R(\tau)$ 通过将 $A[0..W]$ 与 $B[0..N]$ 复数打包为单一长度为 $M = 2^k \ge N$ 的缓冲，经一次正 FFT + 共轭对称解包 + 一次逆 FFT 计算，无循环混叠、零稳态分配、最坏复杂度恒定为 $O(N \log N)$。
- 谐波感知倍周期验证：当首个合格 CMNDF 谷深 `depth > 0.002` 时，检查 $2\times$ 与 $3\times$ 周期 $\pm 2$ lag，若出现谷深改善至 `< 0.25 * depth` 的更深局部谷则升级为真实基频，兼顾弱基频/缺基频合成音与具非谐性（inharmonicity）的真实弦乐。
- 精确周期细化：近数值零整数周期直接保留整数 lag；其余情形以 $O(1)$ 查表对已算 `difference[]` 做五点四次拟合（自适应 `stride`），必要时安全回退三点抛物线插值。

### 2. 双尺度实时检测（`MultiScalePitchDetector`）
- **快速路径**：取最新约 85 ms 窗口覆盖 $\ge 55\text{ Hz}$，实现吉他、提琴、尤克里里及人声低延迟锁定。
- **低音路径**：仅当配置下限 $< 55\text{ Hz}$ 且快速路径未高置信锁定中高音时，对最多 300 ms 缓冲做 31 抽头 Hamming 窗 sinc 抗混叠低通滤波 + 4x 降采样，在降低计算量与抑制高频噪声的同时可靠下潜至 20 Hz / 钢琴 A0（27.5 Hz）与 5 弦贝斯 B0（30.9 Hz）。

### 3. FFT 频谱分析器（`SpectrumAnalyzer`）
- 支持 512–16384 间 2 的幂次 FFT 长度（默认 4096），使用周期 Hann 窗与窗口加权均值去 DC（保证 DC bin 0 无泄漏），相干增益归一化使满幅纯音精确对应 0 dBFS（幅度 0.5 约 −6.02 dBFS）。
- `peak_frequency_hz` 仅供信息参考，严禁代替基频检测。`log_binned` 提供低带宽对数频率分桶视图，直接服务于 P4/P5 的 Tauri Channel 与 Canvas 2D 频谱渲染。

### 4. 对数域 `PitchTracker` 与统一契约
- 所有中值与自适应 EMA 均在 $\log_2 f$（音分）域执行，确保高低八度手感一致。
- `raw()` 完整保留原始检测结果，`display_frequency_hz()` 与 `is_held()` 单独表达平滑展示值。`NoPitch`（静音）、错误、`reset()` 或 $> 50\text{ cents}$ 跳变立即清空历史；仅短时 `LowConfidence` 在 `hold_frames` 预算内防闪烁保持。
- `TunerSnapshot` 与 `MidiNote` 等序列化类型采用 `camelCase`，反序列化经过 `TryFrom` 校验（例如非法 MIDI 128 会被拒绝），杜绝非法值或 `NaN`/`Infinity` 跨越 IPC。

## 验证入口

```powershell
cargo test -p tuner-core --locked
cargo clippy -p tuner-core --all-targets --locked -- -D warnings
cargo run -p tuner-core --example yin_report --release --locked -- --benchmark-only
```
