# Lyreniq 音律核心与数据契约

日期：2026-11-25（Asia/Shanghai）；T05/T06。实现位于 crates/tuner-core，只有 Rust 标准库依赖，不引用 Tauri、设备或 UI。桌面应用尚未链接此 crate，当前 UI 仍使用 T04 的显式 Demo。

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

浮点中点比较采用 1e-10 半音的数值容差（1e-8 cent）；调准容差边界采用相同数量级的保护，防止理论 ±3 cents 因舍入落入错误方向。保护不改变返回的 cents，也不表示检测精度。下文 YIN 的合成测试与这类音律算术测试分开记录，均不保证真实声学测量精度。

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

同一环境、同一输入逐样本一致；测试比较了重复生成结果。跨平台 sin 实现可能有末位差异，目前只验证 Windows x64，不承诺跨平台字节一致。生成器本身没有读写音频文件、设备采集、音频输出、随机噪声或分析算法。

夹具测试检查已知正弦样点、44.1/48 kHz 下的周期数、弱基频/强二三次谐波的投影幅度和能量。投影仅用于测试信号质量，不是生产频谱或音高算法。T06 使用此生成器验证 YIN；复杂泛音的检测鲁棒性属于 T07。

## T06 基础 YIN

YinDetector::detect(pcm, actual_sample_rate) 接受归一化的单声道 f32 PCM [-1,1]，频率和累加使用 f64。YinConfig 先验证配置，sample_count(rate) 根据实际采样率给出精确窗口长度；detector 可在调用间换采样率并复用缓冲，不保存历史音高或平滑值。A4 不参与检测，只影响随后音律映射。

| 配置/输出 | 约定 |
|---|---|
| frequency_range | 默认 20–5000 Hz；严格低于实际 Nyquist，不保证整个范围均达到精度目标 |
| window_ms | 整数 60–300 ms，默认 300；N=floor(rate×ms/1000)，44.1/48 kHz 默认 13,230/14,400 样本 |
| threshold | 有限 0<threshold<1，默认 0.1，CMNDF 插值谷深须严格低于它 |
| min_rms_dbfs | 有限 −160..0，默认 −60；去直流 RMS ≤门限时无音高 |
| PitchDetection | pitches() 为长度 0/1 的 slice，status() 和可缺失的 rms_dbfs()；无会话/时间戳，后续 worker 补充元数据 |

步骤与边界：

1. 在分析前校验实际采样率、频率范围、窗口几何及所有 PCM。N 最多 65,536；超长输入、非有限值或幅度超过 ±1 返回错误，不悄悄裁剪。空/不足一个完整窗口返回 NoPitch，RMS 为 None。
2. 完整窗口减去均值。rms_dbfs=10×log10(mean(centered²))，参考满幅常数为 1，属于 dBFS 而非 dB SPL；精确零能量用 None，避免 −∞。静音/直流/低电平返回 NoPitch。
3. min_lag=floor(rate/max_hz)，max_lag=ceil(rate/min_hz)，额外计算 max_lag+1 供插值。要求 2×(max_lag+1)≤N；短窗口不支持低频配置时直接报错，不静默缩小频率范围。默认 20 Hz 配 60 ms 不相容；应显式提高最低频率或加长窗口。
4. 每个 lag 使用相同 W=N−(max_lag+1) 个比较样本，d(tau)=sum((x[j]−x[j+tau])²)。W 至少覆盖最长周期；不套 Hann 窗，不随 lag 缩短比较段。计算 1..max_lag+1 的累计归一化 D(tau)=tau×d(tau)/sum(d(1..tau))；零分母设为 1。
5. 在范围内按 lag 递增检查局部谷，选择插值谷深小于 threshold 的首个谷，不取可能落在倍周期处的全局最小值。用 CMNDF 三点抛物线的谷深选谷，再用原始差分三点抛物线的横坐标估计周期，frequency=actual_rate/interpolated_lag。
6. 置信度 clamp(1−interpolated_CMNDF_depth,0,1)，仅为周期性指标，不是音高正确概率或误差界。结果频率越出配置范围就返回 NoPitch，不夹成边界、不改选倍周期。超过能量门限但没有合格谷为 LowConfidence。只有 Detected 携带一个经过校验的 PitchEstimate；其他结果不留旧音高。参数/PCM 错误通过 Result 返回，生命周期的 Idle/Listening/Error 由后续 worker/适配器处理。

抛物线 offset=(left−right)/(2×(left−2×center+right))；只在正曲率且 |offset|≤1 时使用，否则退回整数 lag。三组 f64 缓冲采用有界、可失败的分配并复用。直接实现的复杂度为 O(W×max_lag)，只是可对照的参考，不承诺 20–30 Hz 的实时预算，禁止在音频回调中运行；T07 再测量与优化。

依据：[de Cheveigné / Kawahara, YIN (2002), §II.B–E](https://www.ee.columbia.edu/~dpwe/papers/deChevK02-yin.pdf)。公式独立实现，没有复制第三方代码；与论文“未过门限则取全局最小值”的兜底不同，本项目返回 LowConfidence，避免强制输出不可靠读数。未实现论文后续局部时间搜索或其他复杂自适应。

## 初步纯音误差与已知限制

实际误差数据：[pure-tone-errors.csv](evidence/T06/pure-tone-errors.csv)。使用 T05 sine，幅度 0.5、零初相、300 ms、实际 44.1/48 kHz；不是声学或设备采集。22 个必需/内部频点组合全部满足 |error|≤1 cent，最坏绝对误差约 0.053692 cent（48 kHz、1760 Hz）。这只是已测离散频点，不是连续区间保证；完整表及可复现命令见 TESTING。

边界另以 19.9–5100 Hz 配置观察未夹断的原始估计：20 Hz 两种采样率误差均小于 0.000001 cent；5000 Hz 在 44.1 kHz 为 −1.319302299 cents（**未达 ±1 cent 目标**），48 kHz 为 −0.959541949 cents。默认严格 20–5000 Hz 配置下，同样的 20 Hz 两例均返回 NoPitch（微小负偏差越过下限），5000 Hz 两例返回 Detected。不能宣称整个默认范围通过精度验收。T07 必须纳入边界与窗口对照，修复或明确限制有效范围，不降低 ±1 cent 断言。

尚未验证噪声/弱基频/衰减/多音的识别、八度错误率、稳定时间与实时性能；无平滑、FFT、音频线程、设备采集或 UI 桥接。无麦克风不妨碍复现以上测试。

## 验证入口

```powershell
cargo test -p tuner-core --locked
cargo clippy -p tuner-core --all-targets --locked -- -D warnings
cargo tree -p tuner-core --edges normal --locked
pwsh -NoProfile -File scripts/test.ps1
```

当前核心有 2 项单元 + 25 项集成测试及 2 个可执行文档示例；128 个 MIDI 音符 × 四种 A4 参考值、22 个 YIN 必需频点组合均在单个测试内循环，不额外计成几百个测试。完整实际结果见 [TESTING](TESTING.md)。不需要麦克风即可复现。
