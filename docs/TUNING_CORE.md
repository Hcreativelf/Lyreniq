# Lyreniq 音律核心与数据契约

日期：2026-11-25（Asia/Shanghai）；T05。实现位于 crates/tuner-core，只有 Rust 标准库依赖，不引用 Tauri、设备或 UI。桌面应用尚未链接此 crate，当前 UI 仍使用 T04 的显式 Demo。

## 音律计算

使用十二平均律，A4 对应 MIDI 69，C4 对应 MIDI 60。设测量频率 f、A4 参考频率 A、目标 MIDI 音符 n：

```text
m = 69 + 12 × (log2(f) − log2(A))
target_hz = A × 2^((n − 69) / 12)
cents = 1200 × (log2(f) − log2(target_hz))
```

对数之差等价于频率比取对数，可避免极端正有限数的比值上溢/下溢。计算使用 f64，不改写测量频率；正音分偏高、负音分偏低。442 Hz 相对 A4=440 Hz 约 +7.85141504 cents；A4 改成 442 Hz 后，同一物理频率对应 0 cents。

| API/类型 | 约定 |
|---|---|
| frequency_to_midi | 正有限 Hz → 小数 MIDI；允许数学外推到 0–127 外，返回值不夹断 |
| midi_to_frequency | 接受 0–127 的有限小数 MIDI；非法/非有限值返回错误 |
| MidiNote | 经过验证的 0–127 整数音符，提供音级、八度及完整音名 |
| cents_difference | 两个正有限 Hz 的完整音分差，不限制在 ±50 |
| NoteSpelling | Sharps/Flats；如 C#4/Db4；MIDI 0=C-1、21=A0、127=G9 |

TuningConfig 默认 A4=440 Hz、容差 ±3 cents、升号拼写。构造时验证 A4 为 400–480 Hz、容差为 0–50 cents（含边界）。这些是当前核心配置约束，不代表已实现设置页或其他音律。

一次计算使用 TargetSelection：

- Chromatic：最近半音；半音中点选择较高音符。四舍五入后超出 MIDI 0–127 时返回错误，不把无效频率夹成首尾音符。
- Locked：指定目标音；允许偏差超出 ±50 cents，完整保留结果。
- Nearest：从调用者给出的音符列表中按音分距离选最近项；空列表拒绝，重复目标不影响结果；等距选择较高音符，与列表顺序无关。此参数借用 slice，不是 IPC 序列化对象。

浮点中点比较采用 1e-10 半音的数值容差（1e-8 cent）；调准容差边界采用相同数量级的保护，防止理论 ±3 cents 因舍入落入错误方向。保护不改变返回的 cents，也不表示检测精度。尚无基频检测算法或测量精度保证。

## 数据模型与不变量

| 类型 | 数据与验证 |
|---|---|
| PitchEstimate | 正有限 frequency_hz、有限 0–1 confidence；置信度是指标而非概率 |
| PitchStatus | Idle / Listening / NoPitch / LowConfidence / Detected / Error |
| PitchFrame | session_id、非负有限 timestamp_s、pitches、status、可缺失的 rms_dbfs |
| TuningResult | MidiNote、完整音名、目标 Hz、未夹断音分、偏低/合适/偏高方向 |
| FrequencyRange | 正有限且 min<max；默认 20–5000 Hz；可检查实际采样率的严格 Nyquist 边界 |

当前帧只允许 0 或 1 个音高：Detected 必须且只能包含一个，其余状态必须为空。Idle/Error 同时清空信号指标；其他状态的 RMS 可缺失。存在的 RMS 必须有限且 ≤0 dBFS；静音的 −∞ 不可进入模型，可用 None 表示缺失。模型不会产生过期音高、NaN 或 Infinity。

字段私有，通过构造器验证后只读访问。帧拥有 Vec/String 等普通数据，不包含设备、线程或 OS 句柄。T05 尚未加入 serde 或任何 IPC；T11 需建立 Rust/TS 映射、序列化及契约测试，反序列化也必须经过验证，不能绕过不变量。Rust 完整音名 A4 与 T04 TS 的音级 A + 八度 4 需要显式映射。

这些结果对象和测试夹具供工作线程/测试使用，不应在实时音频回调内分配或运行。FrequencyRange 只校验范围与采样率相容，不证明算法在整个范围内有效。

## 确定性合成测试信号

test_signal 提供 sine 与 harmonic_tone，返回 Vec<f32>，接受显式采样率、样本数、频率及幅度。所有谐波相位为零，Harmonic 的 multiple≥1，幅度为 0–1，幅度总和≤1；不自动归一化或削波。每个谐波必须严格低于 Nyquist，避免把混叠信号误称为指定频率。

每次最多 480,000 个样本（约 1.92 MB PCM）、32 个谐波；超限在分配前拒绝，并使用 try_reserve_exact 处理分配失败。零样本数允许返回空数据，便于后续空输入测试；非法信号参数仍会拒绝。

同一环境、同一输入逐样本一致；测试比较了重复生成结果。跨平台 sin 实现可能有末位差异，目前只验证 Windows x64，不承诺跨平台字节一致。没有读写音频文件、设备采集、音频输出、随机噪声、YIN 或 FFT。

夹具测试检查已知正弦样点、44.1/48 kHz 下的周期数、弱基频/强二三次谐波的投影幅度和能量。投影仅用于测试信号质量，不是生产频谱或音高算法。后续 T06 可直接使用此生成器构造输入。

## 验证入口

```powershell
cargo test -p tuner-core --locked
cargo clippy -p tuner-core --all-targets --locked -- -D warnings
cargo tree -p tuner-core --edges normal --locked
pwsh -NoProfile -File scripts/test.ps1
```

当前核心有 17 项集成测试及 1 个可执行文档示例；128 个 MIDI 音符 × 四种 A4 参考值在其中一个测试内循环，不额外计成 512 个测试。完整实际结果见 [TESTING](TESTING.md)。不需要麦克风即可复现本轮计算与夹具测试。
