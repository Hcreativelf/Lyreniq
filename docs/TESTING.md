# Lyreniq 验证记录

## T07 YIN 鲁棒性、平滑与性能（2026-11-25）

起点 58830d6536561526af734ee55ecb201a6984225b（T06），开始干净，安全 fetch 后 ahead/behind=0/0。[T06 CI run 45583704093](https://github.com/Hcreativelf/Lyreniq/actions/runs/45583704093) 按 SHA 核实 completed/success。先重跑 cargo test -p tuner-core --locked（从 D:\ 加 --manifest-path）和五个统一脚本：27 核心+2 文档、2 桌面 Rust、26 TS、release、16 工作区/44 Demo 通过；基线 UI .local/ui-smoke/2026-11-25T09-25-15-825Z。

| 实际命令/检查 | 结果 |
|---|---|
| cargo test -p tuner-core --release --locked | 0；4 单元+33 集成+2 文档，共 37 核心测试+2 文档 |
| cargo clippy -p tuner-core --all-targets --locked -- -D warnings | 0；所有目标无警告 |
| 从 D:\ 经 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\check-env.ps1 / bootstrap.ps1 | 均 0；环境、frozen pnpm、locked Cargo，无新依赖 |
| 同上 test.ps1 | 0；fmt/clippy、37 核心+2 文档、2 桌面 Rust、26 TS、类型/lint/脚本检查通过 |
| 同上 build.ps1 | 0；release exe 生成；UI JS 仍为 247.49 kB/gzip 78.48 kB，未构建 T24 安装包 |
| 同上 test-ui.ps1 | 0；16 组检查、16 工作区/44 Demo、真实 IPC、hidden/unfocused、正常退出/端口释放通过 |
| 同上 bench-yin.ps1 | 0；新增脚本在项目外 Windows PowerShell 5.1 实际运行，顺序隐藏进程，正式误差/耗时/CPU 报告生成 |
| 全部 scripts/*.ps1 的 PowerShell AST 解析 | 通过；包含后加的基准脚本 |

最后完整回归原始 UI .local/ui-smoke/2026-11-25T09-38-53-078Z，沿用 T04 suite，不重复提交未改变的截图。完整回归后只新增基准包装脚本及修改注释；新脚本已实际运行/解析，算法没有再修改。正式基准独立于构建运行，目录 .local/dsp-bench/2026-11-25T09-40-27-263Z；早期并发试跑未用于正式性能表。

[YIN_VALIDATION](YIN_VALIDATION.md) 记录固定矩阵、精确配置、原始对照与计算口径；[T07 摘要](evidence/T07/result.json)、[640 行逐帧误差](evidence/T07/frame-errors.csv)、[64 组摘要](evidence/T07/summary.json)、[20 组耗时](evidence/T07/timings.csv)、[机器/进程 CPU](evidence/T07/machine.json) 可复查。每引擎 320 个窗口，280 个有已知目标；T07 接受 250/280，拒绝的 30 个都是 0 dB。无目标静音等 40 窗口没有假音高。重叠窗口不是独立随机试验。

新增 2 个差分/四次拟合单元测试，5 个鲁棒性/对照/长窗口/夹具集成测试，3 个中值、跳变/无效/错误复位及颤音集成测试。纯音/泛音 ≤1 cent，20/10 dB 接受帧 ≤5/10 cents，衰减 ≤5 cents；实际门槛见源码，未放宽旧 ±1 cent 断言。首轮 10 dB 低音约 43.55 cents 导致测试失败，扩大周期拟合跨度后原断言通过；常量 chunks_exact 的 Clippy 失败以 as_chunks 修复，未禁用检查。

正式 T07 最坏 p95 3.5552 ms；参考 T06 的 48 kHz/440 Hz p95 11.3099 ms，T07 为 0.2015 ms。批处理 CPU 6.78125→1.265625 s，各含 640 计时调用+30 预热；包含生成/输出的进程 CPU 不等于帧墙钟分位数，也不是定时应用整机占用。没有整个音频 worker/FFT/IPC 的性能结论。

用户报告本机无麦克风，本轮未枚举/采集；纯内存合成、不发声。设备/原生 DPI/标题栏/生产离线/安装仍未验。当前 UI 没有接入检测；本轮 Git/CI 在提交后按 SHA 核实，由最终报告给出，下一轮回填。

## T06 基础 YIN（2026-11-25）

起点 af37a77a126d4b90e0efede36c1b83b315ffca31（T05）。开始干净，安全 fetch 后 ahead/behind=0/0；上轮 [CI run 45554383743](https://github.com/Hcreativelf/Lyreniq/actions/runs/45554383743) 的 test-and-build 及全部步骤经 GitHub 连接器核实 success。此前 SHA/run 对应关系保存在忽略的 .local/logs/T05-ci.json。先复验 17 核心集成 + 1 文档、2 桌面 Rust + 26 TS、release 及 16 工作区/44 Demo 后台组合；原始 UI 运行 .local/ui-smoke/2026-11-25T09-02-16-280Z。

单独 Cargo 首次从 D:\ 调用时未指定清单而返回 101；补上 --manifest-path D:\2-workspace\Lyreniq\Cargo.toml 后退出 0，完整基线回归再通过。不是项目测试失败，也未跳过复验。

| 实际命令 | 结果 |
|---|---|
| cargo test -p tuner-core --locked -- --nocapture | 0；2 单元 + 25 集成 + 当时的 1 文档测试；新增 YIN 文档随后单独验证 |
| cargo clippy -p tuner-core --all-targets --locked -- -D warnings | 0，无警告 |
| cargo test -p tuner-core --test yin --locked -- --nocapture --test-threads=1 | 0；8 集成测试，stdout 导出 26 行误差数据 |
| cargo test -p tuner-core --doc --locked | 0；音律及 YIN 两个示例实际编译运行 |
| cargo tree -p tuner-core --edges normal --locked | 0；仅自身，仍无外部依赖 |

YIN 新增 2 个手算/插值单元测试和 8 个集成测试，覆盖：差分/累计归一化与零分母、抛物线顶点/平坦输入、必需纯音精度、静音/直流/空/短帧不留旧值、RMS 门限、实际时钟与缓冲长度变化、窄频率范围的非整数 lag、未达门限不强制输出、配置/Nyquist/容量错误、NaN/Infinity/越幅 PCM 和边界观察。192 kHz 仅验证窗口几何，不宣称检测精度已测。无实时线程或 DSP/UI 集成。

误差表由测试输出生成，未手填预期值。复现时在根目录初始化工具后执行：

```powershell
$accuracyLog = '.local/logs/T06-yin-accuracy.log'
cargo test -p tuner-core --test yin --locked -- --nocapture --test-threads=1 > $accuracyLog
if ($LASTEXITCODE -ne 0) { throw 'YIN accuracy failed' }
$rows = @(Get-Content $accuracyLog | Where-Object { $_.Contains('YIN,') } |
    ForEach-Object { $_.Substring($_.IndexOf('YIN,') + 4) })
if ($rows.Count -ne 26) { throw 'Unexpected accuracy row count' }
@('sample_rate_hz,expected_hz,detected_hz,error_cents,confidence,scope') + $rows |
    Set-Content docs/evidence/T06/pure-tone-errors.csv -Encoding ascii
```

数据见 [完整 CSV](evidence/T06/pure-tone-errors.csv)，配置与算法见 [TUNING_CORE](TUNING_CORE.md)。普通行使用默认 300 ms/20–5000 Hz/阈值 0.1/−60 dBFS、sine 幅度 0.5、零初相；22 个离散组合全部达到绝对误差 ≤1 cent。循环组合不另计成 22 个测试。

| 合成输入 Hz | 44.1 kHz 误差 cents | 48 kHz 误差 cents |
|---|---:|---:|
| 27.5（A0） | −0.000015346 | −0.000007953 |
| 55（A1） | −0.000089926 | −0.000065847 |
| 110（A2） | −0.000007859 | +0.000011695 |
| 123.45 | −0.000089746 | −0.000116142 |
| 220（A3） | +0.000055465 | +0.000085986 |
| 261.625565301（C4） | −0.000190702 | +0.000004716 |
| 432.123 | +0.000280763 | +0.000315424 |
| 440（A4） | +0.001014435 | +0.000381202 |
| 442 | −0.001036599 | −0.000631255 |
| 880（A5） | +0.004829356 | −0.002784191 |
| 1760（A6） | +0.019948049 | +0.053691901 |

边界行使用 19.9–5100 Hz 范围保留原始插值结果，不能与普通行混为同一配置。20 Hz 两个结果 |error|<0.000001 cent；5000 Hz 分别为 −1.319302299 / −0.959541949 cents，**44.1 kHz/5000 Hz 未达 ±1 cent**。边界测试只约束有限值/有效范围/不夹断，没有以放宽精度断言冒充通过。另执行 cargo test -p tuner-core --test yin --locked boundary_characterization -- --nocapture（退出 0），.local/logs/T06-boundary.log 证实：默认严格范围内 20 Hz 两例均 NoPitch、5000 Hz 两例均 Detected；20 Hz 被拒绝源于极小负插值偏差。本轮未承诺连续频段或全默认范围精度。T07 必须对照这些数据修复或记录有效范围限制，±1 cent 目标保持不变。

完整项目最终验收从 D:\ 用 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\<名称>.ps1 依次运行 check-env/bootstrap/test/build/test-ui，五脚本与外层进程均退出 0。test 包含 2 核心单元 + 25 集成 + 2 文档、2 桌面 Rust + 26 TS，以及 fmt/clippy/lint/typecheck/脚本检查；release exe 生成。后台原始运行 .local/ui-smoke/2026-11-25T09-10-17-677Z，16 组检查、16 工作区/44 Demo、真实 IPC、hidden/unfocused、正常关闭/端口释放均通过；这是未改变的 T04 UI 回归，不重复提交截图。详细摘要在 [T06 结果](evidence/T06/result.json)。CI 在本轮 push 后按 SHA 核实，由最终报告给出，下一轮回填。

本机无麦克风为用户报告，本轮未枚举或采集；所有 PCM 仅内存合成、不发声。噪声/衰减/弱基频检测与 CPU 预算留给 T07。原生 DPI/标题栏、真实设备、生产离线与安装仍未验证，既有比例模拟不能代替系统级验收。

## T05 音律核心、模型与合成夹具（2026-11-25）

起点：`784149b18a36ec51f3a0938460e98b9719cb0768`（T04）。工作区干净，fetch 后 ahead/behind=0/0；[T04 CI run 45405643741](https://github.com/Hcreativelf/Lyreniq/actions/runs/45405643741) 按 SHA 核实 success。先从 D:\ 复跑五个统一脚本，原有 2 Rust + 26 TS、release、16 工作区/44 Demo 后台组合全部通过。

| 实际命令/检查 | 结果 |
|---|---|
| cargo check -p tuner-core --offline | 0；仅新增本地 workspace 包，Cargo.lock 增加 4 行，未下载新依赖 |
| cargo test -p tuner-core --locked | 0；17 集成测试（8 音律、4 模型、5 合成信号）+ 1 文档测试通过 |
| cargo clippy -p tuner-core --all-targets --locked -- -D warnings | 0，无警告 |
| cargo tree -p tuner-core --edges normal --locked | 0；只有 tuner-core，自身无外部依赖 |
| 从 D:\ 经 powershell.exe -NoProfile -File 调用 check-env.ps1 / bootstrap.ps1 | 均 0；工具/运行时、pnpm frozen 安装和 cargo fetch --locked 通过 |
| 同上调用 test.ps1 | 0；fmt/clippy、17 核心集成 + 1 文档 + 2 桌面 Rust 测试、26 TS、类型/lint/脚本解析通过 |
| 同上调用 build.ps1 | 0；release exe 构建通过；UI 资产仍为 247.49 kB JS / gzip 78.48 kB；安装包留到 T24 |
| 同上调用 test-ui.ps1 | 0；沿用 T04 的 16 组后台检查，16 工作区/44 Demo 组合、真实只读 Rust IPC、正常退出及端口释放通过 |

17 项核心测试覆盖：A4=400/440/442/480 下全部 128 MIDI 往返（循环 512 个组合，不额外计作 512 个测试）；A0/C4/A4、升降号/八度、442 对 440 约 +7.85141504 cents；正负偏差和容差边界、锁定目标不夹断、列表等距/顺序/重复、MIDI 首尾、极端有限 Hz 的稳定计算、非法/非有限值拒绝。帧测试检查无效值和过期音高拒绝；夹具测试检查样点、两种采样率周期数、谐波投影幅度/能量、Nyquist、大小/幅度约束。

核心及实际回归摘要见 [T05 结果](evidence/T05/result.json)，公式和 API 说明见 [TUNING_CORE](TUNING_CORE.md)。最终后台回归原始运行位于 .local/ui-smoke/2026-11-25T05-25-20-556Z，suite 名仍为 T04，表示旧 UI 回归而非 T05 新功能。UI 源码没有修改，不重复提交相同截图；既有 [T04 截图](evidence/T04/demo-dark-1024x700-tuned.png)继续有效。日志保持在 .local/logs。

用户于本日说明本机没有麦克风；本轮未执行设备枚举/采集，没有设备数量结论。合成信号只在内存中生成，未发声，不能替代设备或检测精度结果。当前无 YIN/FFT、无 UI 桥接/IPC 序列化；设备、原生 DPI/标题栏、生产离线与安装仍待验。本轮 CI 在提交/push 后按 SHA 核实，由最终报告给出，下一轮回填。

## T04 调音显示与显式 Demo 验收（2026-11-24）

起点：`3c6779919ff169f874d074baac3a7578673a8f76`（T03）。开始时工作区干净，fetch 后 ahead/behind=0/0；上轮 [Windows CI run 45387529930](https://github.com/Hcreativelf/Lyreniq/actions/runs/45387529930) 按 SHA 核实 success。先从 D:\ 复跑 check-env/bootstrap/test/build/test-ui，2 Rust + 6 TS、release 及 T03 全套后台 UI 均通过，随后实施 T04。

最终仍从 D:\ 使用 `powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\<名称>.ps1` 逐个调用，外层进程也退出 0。

| 实际脚本/检查 | 结果 |
|---|---|
| check-env / bootstrap | 均 0；环境、pnpm frozen 安装、cargo fetch --locked 通过；集中缓存链接正常 |
| test | 0；PS/MJS 解析、脚本/前端 Biome、Rust fmt/clippy、2 Rust IPC、类型及 26 项 TS 测试通过 |
| build | 0；release exe 生成；Vite 2,144 模块，JS 247.49 kB / gzip 78.48 kB；不生成安装包 |
| test-ui | 0；16 组检查通过，包含原有 16 工作区组合和新增 44 Demo 组合 |
| 默认状态 | 无音高/频率/音分读数、无指针/信号数值；开始监听禁用；Demo 必须主动进入 |
| 指针与数值 | -50/0/+50、±75 实际渲染；指针范围 ±50，±75 数字不截断，量程提示可见 |
| 状态与音符 | A4→C4 替换；等待、静音、低置信度、错误、idle 清除旧音高；错误/idle 清除信号指标 |
| 恢复和退出 | Demo 错误重试回到 A4；退出清空数据并聚焦入口；离页/返回不恢复旧 Demo |
| 键盘与布局 | CDP Enter、Home/方向键选择、退出焦点；两主题 × 1024×700/1440×900 × 11 场景，无溢出或裁切控件；全程 hidden/unfocused |
| 回归桥接/退出 | 真实 Rust runtime_label/重连 IPC、主题导航/焦点、18 组文字 token 对比度、正常退出 0 和端口释放均通过 |

新增 20 项 TS 测试：10 项展示/状态/生命周期检查和 10 项适配器/非法值检查。包含旧适配器迟到帧被忽略、NaN/Infinity/非法目标/置信度/音名等导致错误清理。原有 6 项 T03 行为测试和 2 项 Rust 测试均保留。

真实失败记录：最初项目外 Git 命令遗漏 -C，纠正目录后恢复检查通过，基线无故障。新增 Demo 夹具返回类型过宽导致 typecheck 失败，收窄 detected 判别类型后通过。首轮后台重试检查在 React 更新前读到临时空值而失败；改为等待明确的音名/状态/焦点完成条件，保留原断言，随后完整运行通过，不使用固定延时或放宽结果。

最终原始运行：.local/ui-smoke/2026-11-24T13-33-29-937Z；[脱敏结果](evidence/T04/result.json)和 45 张 PNG 保存在 docs/evidence/T04。44 个 Demo 组合均验证布局与读数；1024×700 保存全部场景，两种大窗口主题另保存准确状态截图；部分大窗口场景只记录 DOM/布局断言，没有逐项截图。已直接查看默认、深浅准确/超量程及错误状态截图。

本次 UI 音高数值全部来自标明来源的固定 Demo 夹具，真实 Rust 验证只针对已有只读 runtime_label；不把 Demo 称为麦克风采集、DSP 精度或音频设备测试。Windows 原生 DPI/标题栏、真实设备、生产离线、安装仍未验；125%/150% 继续只是渲染器模拟。云端 CI 只执行工程检查/release，实际结果在 push 后核实，由交付报告提供，下一轮回填。

## T03 工作区壳与主题验收（2026-11-24）

起点：`1355bd94121e4bd8dec8b82e15dee765f8d4106e`（T02-b）。开始时工作区干净、fetch 后与 origin/main 一致；上轮 [Windows CI run 45345059150](https://github.com/Hcreativelf/Lyreniq/actions/runs/45345059150) 按 SHA 核实 success。先从 D:\ 运行 check-env/bootstrap/test/build/test-ui，原有 2 Rust + 3 TS、release 和 5 项隐藏原生 UI 检查均通过，随后才实施 T03。

| 实际命令/检查 | 结果 |
|---|---|
| PowerShell 7 前端 typecheck/test、Biome | 退出 0；最终 6 项 TS 测试通过，保留原有 3 项连接行为回归 |
| 在 D:\ 调用 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\check-env.ps1 | 退出 0；版本、MSVC、SDK、WebView2 通过 |
| 同上调用 bootstrap.ps1 | 退出 0；pnpm frozen 安装、cargo fetch --locked 通过；迁移后的缓存链接可用 |
| 同上调用 test.ps1 | 退出 0；PS/MJS 解析、脚本/前端 Biome、Rust fmt/clippy、2 Rust IPC、类型及 6 TS 测试通过 |
| 同上调用 build.ps1 | 退出 0；release exe 生成；Vite 2,139 模块，JS 237.58 kB / gzip 74.95 kB；安装包明确留给 T24 |
| 同上调用 test-ui.ps1 | 退出 0；11 组检查全部通过，包含 16 个原生窗口/主题/页面组合 |
| 原生窗口矩阵 | 真正 set_size 到 1024×700、1440×900；深浅主题 × 实时调音/文件分析/参考音/设置；无文档/主体溢出、可操作控件不裁切、未开放动作禁用 |
| 真实桥接与退出 | Rust runtime_label 返回值、按钮 POST IPC 200/ok/返回值、hidden/unfocused、正常关闭退出 0 与端口释放通过 |
| 键盘与焦点 | 页面级 CDP Tab、Enter、Space；跳转链接、导航、页面 h1 聚焦、主题选择和 2px 可见焦点实测通过；没有系统键盘注入 |
| 文字对比度 | 18 组 token/表面组合全部 ≥4.5:1，最低 4.8486:1 |
| 125%/150% | CDP 设备比例模拟，在逻辑 1024×700 无溢出；与原生窗口矩阵分开，不等于 Windows DPI 切换验收 |
| 缓存迁移 | Cargo 10,845、npm 62、旧 Vite/Vitest 21 个文件逐一 SHA-256 一致；兼容 junction、官方配置及实际新路径写入核实；之后完整回归退出 0 |

新增 3 项行为测试覆盖四页面导航及焦点、两处主题控件同步/跨导航保持、键盘入口与无虚构测量值。首次 lint 检出分组/列表语义问题，已改用 fieldset/ul；新增测试一度使用 Testing Library 不支持的 exact 参数，已修正为其默认精确名称匹配。首次完整 UI 检查发现浅色次要文字 4.45:1，实际失败记录保留；调深后两次完整后台测试通过。没有关闭规则、放宽对比度或跳过断言。

最终原始 UI 运行：.local/ui-smoke/2026-11-24T11-37-34-367Z；[脱敏结果](evidence/T03/result.json)及全部 20 张 PNG 保存在 docs/evidence/T03。已直接查看两种尺寸/主题、设置页及键盘焦点的实际截图。例：[深色 1024×700](evidence/T03/dark-1024x700-tuner.png)、[浅色 1440×900](evidence/T03/light-1440x900-tuner.png)、[设置](evidence/T03/light-1024x700-settings.png)、[键盘焦点](evidence/T03/keyboard-focus.png)。

最终日志位于 .local/logs/bootstrap.log、test.log、build.log、ui-smoke.log、dev-ui-smoke.log；缓存验证与开源参考快照同在 .local/logs，不提交机器数据。check-env 输出由本次工具执行记录保存，该脚本不生成单独日志文件。

T03 本地验收通过。本轮 CI 实际结果在 push 后按 SHA 核实，由最终报告给出并在下一轮回填。CI 执行工程检查/release，不执行本机隐藏 GUI。未执行：Windows 原生 DPI/跨屏切换、标题栏实体交互、音频设备、生产断网、安装包及完整调音功能验收。

## T02-b 后台原生 UI 验收（2026-11-24）

起点：`def1765d43745d9b310fbeba511ff257ee140f9a`（T02-a）。开始时工作区干净、远端一致；先复跑 check-env/bootstrap/test/build。上轮 [Windows checks run 45338108012](https://github.com/Hcreativelf/Lyreniq/actions/runs/45338108012) 的实际 conclusion=success，工程检查与 release 构建全部通过。

用户正在使用桌面，本轮采用隐藏的真实 Tauri/WebView2 实例，通过 loopback CDP 操作页面；没有使用桌面控制工具、前置窗口或注入系统鼠标键盘。测试配置、profile 和构建目录隔离，不修改生产窗口配置或 ACL。

| 命令/检查 | 实际结果 |
|---|---|
| scripts/test-ui.ps1（PowerShell 7） | 最终方案全部 5 项检查通过，退出 0 |
| 在 D:\ 经 Windows PowerShell 5.1 调用 scripts/test-ui.ps1 | 最终格式化脚本全部 5 项检查通过，退出 0；8.345 秒 |
| 真实页面与 Rust | 页面显示 Lyreniq；实际 runtime_label 返回 Lyreniq 0.1.0 · windows |
| 后台状态 | 实际原生 is_visible=false、is_focused=false；按钮与截图后重复断言通过 |
| 重连按钮 | CDP 点击页面按钮；观察真实 POST ipc.localhost/runtime_label，状态 200、tauri-response=ok，响应值与 Rust 版本一致 |
| 渲染器截图 | 1000×700、无页面溢出；已人工查看生成 PNG；不包含用户桌面 |
| 正常关闭 | 测试专用窗口 close IPC；dev 退出 0；Vite 与 CDP 端口释放；不冒充标题栏实体点击 |
| 最终 bootstrap.ps1 | 退出 0；pnpm frozen-lockfile、cargo fetch --locked 通过 |
| 最终 test.ps1 | 退出 0；PS/MJS 解析、MJS Biome recommended、Rust fmt/clippy、2 项 Rust IPC、前端 lint/types 和 3 项 Vitest 通过 |
| 最终 build.ps1 | 退出 0；target/release/lyreniq-desktop.exe 生成；安装包留给 T24 |

证据：[后台渲染截图](evidence/T02-b/native-window.png)、[脱敏结果](evidence/T02-b/result.json)。原始最终运行位于 .local/ui-smoke/2026-11-24T06-49-07-123Z，独立日志和 latest.json 保留在被忽略的 .local 内。playwright-core 1.65.1 精确锁定，直接连接已安装 WebView2，没有下载另一个浏览器。

中间一次观测方案因尝试替换 Tauri 只读 invoke 属性而超时，结果保留为 failed；最终改为监听真实 IPC 网络响应，随后两次完整验证均通过，没有更改保护或降低返回值断言。

T02-b 与 T02 父任务的工程 smoke 验收完成。标题栏实体交互、DPI 矩阵、系统权限、设备与生产离线不在本次结果内。现有 GitHub CI 只执行工程检查和 release 构建，后台原生 UI 目前仅本机实测。本轮提交后的 CI 实际状态由交付报告和对应 SHA 的 Actions 核实，下一轮回填。

## T02-a 验收与当时的 T02-b 待验（历史快照，2026-11-24）

以下保留当时的真实状态；T02-b 的后续完成证据及 T02-a 的实际 CI 成功结果见上节。

起点：`0ba2701de101f39f6205b0836127969a773cf456`。工作区起初干净，fetch 后与远端一致；先复跑 T01 的 frozen 安装、fmt、cargo check、前端构建，并实际观察原生窗口及真实 runtime_label 成功，再实施 T02。

| 命令/检查 | 实际结果 |
|---|---|
| 在 D:\ 调用 scripts/check-env.ps1 | PowerShell 7 和 Windows PowerShell 5.1 均退出 0；工具版本、MSVC、SDK、WebView2 通过 |
| 在 D:\ 调用 scripts/bootstrap.ps1 | 0；pnpm install --frozen-lockfile、cargo fetch --locked --target x86_64-pc-windows-msvc 通过 |
| scripts/test.ps1 | PowerShell 7、5.1 均退出 0；PowerShell 解析、Rust fmt/clippy、前端 Biome/typecheck 通过 |
| Rust IPC 测试 | 2 passed：注册命令可经 IPC 调用、未注册命令被拒绝；main/doc test 为 0 项，不计入 2 项 |
| 前端测试 | 3 passed：等待真实响应才显示成功、失败后可重试、后续连接失败清除旧成功读数 |
| Invoke-Checked 的受控 exit 7 演练 | 成功捕获为失败；没有被 Out-Host/日志管线吞掉退出码 |
| scripts/build.ps1 | 0；最终 Windows 清单配置在 PowerShell 5.1（项目目录外）通过；产物 target/release/lyreniq-desktop.exe；跳过安装包 |
| scripts/dev.ps1 | 在 D:\ 经 PowerShell 5.1 启动 Vite 与 cargo run --locked，原生 exe 已运行；最终可视窗口/按钮/退出复验未完成，见下 |
| 本地日志 | .local/logs/bootstrap.log、test.log、build.log、dev.log；命令、退出码与 stdout，stderr 同时显示于控制台；均忽略 |
| Windows CI | 配置已建立；本记录写于 push 前，实际运行状态由 push 后交付报告及 GitHub Actions 记录核实 |

首次 Rust 测试在运行前报 STATUS_ENTRYPOINT_NOT_FOUND（0xc0000139）；链接诊断确认 Common Controls v6 API。第一次直接追加清单还造成重复 manifest（LNK1123）；最终使用单一 windows.manifest，为应用和测试共同链接，随后 Rust 测试与 release 构建通过。没有降低断言、跳过测试或安装替代 DLL。

新增依赖固定为 Vitest 4.1.12、Biome 2.6.8、Testing Library React 16.3.3/user-event 14.6.13、jest-dom 7.0.1、jsdom 27.4.0；保持 T01 的 Vite/React 主版本。Biome 使用 recommended preset，不关闭 lint 规避问题。

**T02-b 尚未验收**：最终统一 dev 脚本的窗口检查期间，桌面工具报告物理 Esc 停止。本轮已停止桌面控制并终止测试用开发进程。脚本启动输出、mock IPC 和此前 T01 的窗口证据均不能替代这一最终 GUI 复验，因此 T02 父任务仍为 IN_PROGRESS。

T02-b 恢复时先核实 Git/文件并重跑 check-env/bootstrap/test/build，再启动 dev.ps1，实际检查窗口、Rust 返回值、按钮和退出；必要时保存 docs/evidence/T02-b。通过后才将 T02 父任务标为 VERIFIED，并进入 T03。

## T01 验收（2026-11-24）

起点为 T00 提交 `562cfadb1b818b1f04b19d05c114b9cba211c5fd`。实施前重跑 T00 的文档/哈希断言、15 项忽略与 6 项可跟踪例外，检查 Git 工作区、历史、diff 与提交空白，全部符合预期；上一轮没有应用，应用构建/GUI 回归不适用。

| 命令/检查 | 结果 | 证据 |
|---|---|---|
| `node --version`、`pnpm --version`、`rustc --version` | 均 0 | v24.21.0 / 11.25.0 / rustc 1.100.0；环境选择见 ENVIRONMENT |
| `pnpm install --frozen-lockfile` | 0 | Already up to date；两项目 workspace；仅 esbuild 构建脚本被批准 |
| `cargo fmt --all -- --check` | 0 | Rust 格式检查通过 |
| `cargo check --workspace --locked` | 0 | lyreniq-desktop 编译检查通过；最终配置复验耗时 0.68s |
| `pnpm build` | 0 | TypeScript 无错误；Vite 7.3.8 构建 32 个模块；最终 459ms |
| `pnpm dev` | 原生窗口/IPC 通过；退出 0 | 原生 exe=target/debug/lyreniq-desktop.exe；页面显示 Lyreniq 0.1.0 · windows；重新验证按钮调用后仍成功 |
| Windows 原生窗口证据 | 已保存 | [截图](evidence/T01/native-window.jpg)、[观测记录](evidence/T01/native-window.json) |
| T00 GitHub 补推及核实 | 0，远端 hash 一致 | `git push -u origin main`、`git ls-remote --heads origin main` 返回 T00 hash |
| 提交前远端同步 | 0 | `git fetch origin`、`git rev-list --left-right --count HEAD...origin/main` 输出 0/0 |

首次 pnpm 安装因旧式配置/脚本未授权失败，修正 workspace YAML 和共享 pnpm 选择后重装成功；重复键错误也已修复，未把这些失败记成通过。MSVC 安装器返回 3010，后续 vswhere isComplete/isLaunchable=true、isRebootRequired=false，未重启即完成实际编译和窗口验收。

窗口正常渲染并完成按钮验证；后续 inventory 显示窗口已退出，开发 CLI 返回 0。尝试代理 Alt+F4 时因检测到用户输入被保护机制拦截，未把这一动作计作代理自动关闭测试通过。首次遮挡的截图没有用作证据；已前置窗口后保存纯 Lyreniq 截图。

T01 尚无单元测试套件、lint 或 CI；不声称 cargo test、Vitest、安装包构建、真实音频或生产离线运行通过。T02 将建立相应工程回归基线。

下一轮先选择 ENVIRONMENT 中的共享工具链，在项目根目录复跑：

```powershell
node --version
pnpm --version
rustc --version
pnpm install --frozen-lockfile
cargo fmt --all -- --check
cargo check --workspace --locked
pnpm build
pnpm dev
git diff --check
git show --format= --check HEAD
```

smoke test 必须观察真实原生窗口和 Rust 返回值，不能只看到 Vite ready 就判断桌面验收通过。核对两份锁文件仍被跟踪，以及 target/node_modules/dist/.local/gen 未被提交。原任务书 SHA-256 仍与 D002 基准一致。

## T00 验收范围（历史快照，2026-11-24）

本轮只有文档和 Git 初始化。原目录没有上一轮成果、测试或应用；上一轮应用回归不适用。任务书中的算法、UI、设备与发布矩阵是未来目标，本轮不把它们记作通过。

| 检查 | 当前结果 | 证据 |
|---|---|---|
| 原目录、文件与 Git 盘点 | 通过 | 原目录只有任务书；Git 初始不存在；见 HANDOFF |
| 完整阅读主任务书 | 完成 | 按行段读取全部 1,213 行 |
| Git 初始化 | 通过 | `git init -b main` 返回 0 |
| 原件保留与 docs 副本哈希 | 通过 | SHA-256 均与 DECISIONS D002 基准相同 |
| 基础文档、链接、任务 ID 一致性 | 通过；PowerShell 退出 0 | 11 个文件、8 个本地链接；当前 T00，唯一下一任务 T01；没有提前创建应用工程 |
| Git 空白及 staged 内容审查 | 通过；退出 0 | `git diff --check`、`git diff --cached --check`；仅 11 个本轮文件 |
| 忽略缓存、音频、用户数据和密钥 | 通过 | 15/15 忽略；6 项 fixtures/锁文件/环境示例未忽略；分别退出 0/1（均符合预期） |
| Git 暂存区任务书原始字节 | 通过 | 两个 indexed blob 与 `git hash-object --no-filters` 相同；见 HANDOFF |
| 常见凭据模式扫描与文本审查 | 通过 | 私钥/GitHub token/AWS access key 模式无匹配（rg 退出 1，预期）；人工检查 staged 文档 |
| GitHub remote / gh 认证 | 待配置 / CLI 不可用 | 无 remote；`gh auth status` 返回 1，未安装 gh |
| 应用测试、构建、GUI、设备、安装 | 未执行；不适用 | 没有应用、清单、测试或脚本 |

## 复验入口

从项目根目录运行；PowerShell 使用 `-NoProfile` 可避开本机已有 profile 的 conda 启动错误。

```powershell
git rev-parse --show-toplevel
git status --short --branch
git branch --show-current
git log -5 --oneline
git diff
git diff --cached
git remote -v
git diff --check
git show --format= --check HEAD
Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256
Get-FileHash -LiteralPath docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256
```

检查 AGENTS、README、STATUS、HANDOFF、DECISIONS、TESTING 的本地 Markdown 链接均指向现有文件；核实软件名 Lyreniq、T00 结果和唯一下一任务 T01，并与实际文件和 Git 日志对照。T00 两份任务书哈希相同；未来主文档获批修订后以决策记录为准。

本轮文档内联断言，可在 T01 开始实施前复跑（不安装工具、不创建文件）：

```powershell
$ErrorActionPreference = 'Stop'
$taskExpected = @('AGENTS.md','README.md','.gitignore','.gitattributes','.editorconfig','UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md','docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md','docs/STATUS.md','docs/HANDOFF.md','docs/DECISIONS.md','docs/TESTING.md')
foreach ($taskFile in $taskExpected) {
    if (-not (Test-Path -LiteralPath $taskFile -PathType Leaf)) { throw "Missing file: $taskFile" }
}
$taskFiles = @(& rg --files --hidden -g '!.git')
if ($LASTEXITCODE -ne 0 -or $taskFiles.Count -ne $taskExpected.Count) { throw 'Unexpected project file inventory' }
$taskLinks = 0
foreach ($taskFile in @('AGENTS.md','README.md','docs/STATUS.md','docs/HANDOFF.md','docs/DECISIONS.md','docs/TESTING.md')) {
    $taskText = Get-Content -LiteralPath $taskFile -Raw -Encoding UTF8
    foreach ($taskMatch in [regex]::Matches($taskText,'\[[^\]]+\]\(([^)]+)\)')) {
        $taskLink = $taskMatch.Groups[1].Value
        if ($taskLink -match '^(https?://|#)') { continue }
        $taskLinkPath = Join-Path (Split-Path -Parent (Join-Path (Get-Location) $taskFile)) $taskLink
        if (-not (Test-Path -LiteralPath $taskLinkPath -PathType Leaf)) { throw "Broken link in ${taskFile}: $taskLink" }
        $taskLinks++
    }
}
foreach ($taskFile in @('README.md','docs/STATUS.md','docs/HANDOFF.md')) {
    $taskText = Get-Content -LiteralPath $taskFile -Raw -Encoding UTF8
    foreach ($taskToken in @('Lyreniq','T00','T01')) {
        if (-not $taskText.Contains($taskToken)) { throw "Missing ${taskToken}: $taskFile" }
    }
}
foreach ($taskPath in @('Cargo.toml','package.json','apps','crates','scripts','.github')) {
    if (Test-Path -LiteralPath $taskPath) { throw "T00 scope exceeded: $taskPath" }
}
$taskOriginalHash = (Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256).Hash
$taskDocHash = (Get-FileHash -LiteralPath docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256).Hash
if ($taskOriginalHash -ne 'C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF' -or $taskDocHash -ne $taskOriginalHash) { throw 'Plan bytes changed' }
Write-Output "PASS: $($taskFiles.Count) project files, $taskLinks local links, task/name consistency, T00 scope, both plan SHA-256 values"
```

该断言专用于 T00 基线。T01 开始新增工程后，文件数量/范围断言和副本哈希条件需按实际任务调整，不能误报后续合法变更失败。

忽略规则可用以下无须创建文件的命令复验：

```powershell
git check-ignore --no-index -- target/debug/probe.obj node_modules/probe.js apps/desktop/dist/probe.js .local/cache/probe .venv/probe .env .env.production debug.log recordings/probe.wav probe.wav user-data/settings.json settings.json installer.msi signing.key .vscode/settings.json
git check-ignore --no-index -- .env.example Cargo.lock pnpm-lock.yaml tests/fixtures/tone.wav tests/fixtures/tone.mp3 tests/fixtures/tone.flac
```

第一条期望逐个输出全部路径且退出码为 0；第二条期望无输出且退出码为 1（均未被忽略，属于预期结果）。正式夹具仅允许小型合成或明确授权音频，不能因存在例外就提交个人录音。

最终提交结果以 `git log` 和交付报告核实；GitHub push 状态独立记录，不能由本地 Git 成功推断。常见凭据模式扫描及人工 staged diff 审查用于本轮文本核验，不代表未来文件免于审查。
