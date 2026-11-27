# Lyreniq 项目状态

- 更新日期：2026-11-27（Asia/Shanghai）
- 现行计划：v2.0（四层架构 + P1–P8 垂直闭环，见 D018 与 `docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md`）
- 当前阶段：P1 — DSP 2.0 核心重构
- 状态：VERIFIED（本地 DSP 2.0 精度/性能/回归全通过；本轮 commit/push 在最终报告给出，下轮回填 SHA）
- 最近通过：P1；既有基线 T00–T07 保持完整通过
- 当前分支：main，跟踪 origin/main
- 起点 commit：0db1504（T07，已核实与 origin/main 0/0 同步）
- 本轮新增：
  - `docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 升级至 v2.0，`docs/DECISIONS.md` 记录 D018
  - `crates/tuner-core` 新增 `rustfft = "=6.4.1"`、`serde = "=1.0.229"`（dev: `serde_json = "=1.0.154"`），保持 `#![forbid(unsafe_code)]`
  - `spectrum.rs`：Hann 窗幅频谱、窗口加权去 DC、相干增益 dBFS、谐波峰值查询、对数频率分桶
  - `yin.rs`：FFT 加速 Fast YIN（$O(N \log N)$ 恒定最坏复杂度）、谐波感知 2x/3x 周期验证、`MultiScalePitchDetector`（85 ms 快速路径 + 4x Hamming-sinc 抗混叠降采样低音路径）；移除生产公共 API `detect_reference`
  - `smoothing.rs`：对数频率（音分）域 `PitchTracker`（3 帧中值 + 自适应 EMA + 可配置 `hold_frames` 瞬时防闪烁 + 静音/跳变立即重置）
  - `profile.rs`：10 种内置乐器预设 + 自定义目标校验
  - `synth.rs`：零分配、相位连续、15 ms 防爆音淡入淡出 `ToneGenerator`
  - `model.rs` / `tuning.rs`：统一 `camelCase` 序列化契约（`TunerSnapshot`、`TuningResult` 含 `pitchClass`/`octave`、`MidiNote` 反序列化校验）
- 精度与性能（i5-14600KF Release）：
  - 原 T07 640 帧固定矩阵全数通过（纯音/泛音 $\le 1$ cent，20/10 dB 达标，0 dB 与静音无误报），新增非谐性弱基频吉他 E2（$< 5$ cents）与双尺度 20 Hz–1760 Hz（$\le 1$ cent）通过
  - 48 kHz/300 ms `FastYIN` 最坏 p95 = **0.156 ms**（对比原 T07 3.555 ms 提速 ~23x）；`MultiScale` 中高音 p95 = **0.037–0.043 ms**，低音/纯噪声双路径最坏 p95 = **0.110 ms**；`SpectrumAnalyzer` 4096 点 p95 = **0.056 ms**
- 验证：47 核心测试（4 单元 + 43 集成）+ 2 文档测试 + 2 桌面 Rust 测试 + 26 前端 TS 测试 + fmt/clippy/lint/typecheck/PS AST 全通过
- 未实现：P2（Symphonia 流式解码与离线分析引擎）、P3（CPAL 麦克风/WASAPI Loopback/参考音与实时管线）、P4–P8
- 未验证：真实乐器声学录音（将在 P2 接入后立即实测）、真实麦克风/声卡设备、Windows 原生 DPI/标题栏、生产离线与安装包
- 唯一下一阶段：**P2 — 流式解码与离线文件分析引擎**

| 阶段/ID | 状态 | 验收证据 | 对应提交/分支 |
|---|---|---|---|
| T00–T07 | VERIFIED（历史基线） | 工程脚本、桌面壳、音律数学、640 帧合成基线 | 562cfad..0db1504 / main |
| P1 | VERIFIED | v2.0 计划、DSP 2.0 47+2 核心测试、Release 基准、完整工作区回归 | 本轮提交后报告 hash / main |
| P2 | NOT_STARTED | 唯一下一阶段：Symphonia WAV/FLAC/MP3 解码 + `tuner-engine` 离线流式分析 | 无 |
| P3 | NOT_STARTED | `audio-io`（CPAL 麦克风 + WASAPI Loopback + 参考音 + 虚拟流）+ `LiveSession` | 无 |
| P4 | NOT_STARTED | 极薄 `src-tauri` 桥接、Tauri Channel、本地设置持久化与契约测试 | 无 |
| P5 | NOT_STARTED | 实时调音与频谱工作台 UI 重构及真实后端接入 | 无 |
| P6 | NOT_STARTED | 文件分析工作台 UI（导入/时间轴/统计/CSV 导出） | 无 |
| P7 | NOT_STARTED | 参考音与设置页面完整落地 | 无 |
| P8 | NOT_STARTED | 稳定性压测、离线隐私审查与 Windows 安装包发布 | 无 |
