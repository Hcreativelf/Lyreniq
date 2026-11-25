# Lyreniq 交接记录

## 任务与起点

- 日期：2026-11-25（Asia/Shanghai）；D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点：784149b18a36ec51f3a0938460e98b9719cb0768（T04）。起初干净，安全 fetch 后 ahead/behind=0/0；上轮 [CI run 45405643741](https://github.com/Hcreativelf/Lyreniq/actions/runs/45405643741) 按 SHA 核实 success。
- 实施前从 D:\ 复跑 check-env/bootstrap/test/build/test-ui；原有 2 Rust、26 TS、release、16 工作区与 44 Demo 后台回归通过。
- 本轮仅完成 T05 — 音律与 DSP 数据模型；T06 未开始，不实现 YIN/FFT/设备或 UI 桥接。
- 用户明确说明本机没有麦克风。这是用户提供的硬件条件，本轮未枚举设备，不预设 OS 可用输入设备数量。

## 本轮交付

- Cargo workspace 新增 crates/tuner-core，Cargo.lock 仅增加本地包条目；没有外部依赖，cargo tree 已实际核实。桌面 crate 未链接核心，UI 保持 T04 显式 Demo。
- 十二平均律 MIDI/Hz/cents、A4 校准、升降号/八度、最近半音/锁定/目标列表映射；完整音分不夹断。使用对数之差避免极端比值溢出；中点选高音并记录浮点保护。
- 私有只读校验模型：PitchEstimate/PitchFrame/PitchStatus/TuningResult/TuningConfig/FrequencyRange；拒绝非有限或非法值。Detected 只能一个音高，其他状态无音高；Idle/Error 清空信号指标。
- test_signal 模块生成有界、确定性的正弦/谐波 Vec<f32>；指定实际采样率、样本数和幅度预算，校验 Nyquist；无设备、音频播放、文件生成或随机数据。
- 17 核心集成测试 + 1 文档测试；[TUNING_CORE](TUNING_CORE.md)记录公式、API、约束、Rust/TS 后续映射和夹具边界。
- README/STATUS/TESTING/ENVIRONMENT/DECISIONS 已同步；[T05 结果](evidence/T05/result.json)保存实际验收摘要。两份原始任务书未改。

## 实际命令与结果

| 命令/检查 | 结果 |
|---|---|
| cargo check -p tuner-core --offline | 0，离线编译，锁文件只增本地包 |
| cargo test -p tuner-core --locked | 0；8 音律 + 4 模型 + 5 信号 =17 集成测试，1 文档测试 |
| cargo clippy -p tuner-core --all-targets --locked -- -D warnings | 0，无警告 |
| cargo tree -p tuner-core --edges normal --locked | 0，只显示 tuner-core，零外部依赖 |
| 从 D:\ 经 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\check-env.ps1 / bootstrap.ps1 | 均 0；环境、frozen pnpm、locked Cargo 通过 |
| 同上 test.ps1 | 0；fmt/clippy、17 核心 + 1 文档 + 2 桌面 Rust、26 TS、前端/脚本 lint/typecheck 全通过 |
| 同上 build.ps1 | 0；release exe 生成；UI 仍为 247.49 kB JS / gzip 78.48 kB；安装包留给 T24 |
| 同上 test-ui.ps1 | 0；T04 suite 的 16 组后台检查，16 工作区/44 Demo、真实 runtime_label IPC、关闭/端口释放通过 |

覆盖 128 MIDI×4 A4 的循环、A0/C4/A4、442→440 的 +7.85141504 cents、正负/超量程、目标/边界/非法值与极端有限输入；夹具验证样点、周期数、谐波幅度和能量。循环组合不额外计成几百个测试。文档示例也实际编译运行。

最终完整 UI 回归原始运行 .local/ui-smoke/2026-11-25T05-25-20-556Z，suite 标识保留 T04；T05 没有新 UI，不重复提交截图。日志 .local/logs/bootstrap.log、test.log、build.log、ui-smoke.log、dev-ui-smoke.log 保持忽略。完整回归后仅整理校准测试名称/移除局部常量断言，并再次通过 core test 和 fmt；库代码未再改动。

## 设备、契约与边界

- 无麦克风不影响本轮或基础合成算法测试。T09 再实际枚举/选择输入并实现无可用输入时的友好提示；真实采集/切换/拔出需有输入设备后的独立证据，不能拿 Demo 代替。
- TuningConfig 当前约束 A4 400–480 Hz、容差 0–50 cents；MIDI 音符 0–127；合成夹具最多 480,000 样本/32 谐波。细节见 TUNING_CORE 与 D014。
- T04 TS 展示契约未更改。Rust 结果为完整音名 A4，TS 为音级 A + 八度 4；T11 必须显式映射和做序列化契约测试。当前未引入 serde，未来反序列化不可绕过验证。
- 尚无音高检测器/FFT，不能将音律算术误差测试称作基频检测精度。原生 DPI/标题栏、设备、生产离线和安装仍待验。
- 本机缓存沿用 D:\4-stuff\Cache 及兼容链接；无新工具/包缓存设置。其他机器仍按 ENVIRONMENT 的项目内回退，不照搬盘符。

## Git 与 CI

- 授权仓库：https://github.com/Hcreativelf/Lyreniq.git；不强推、不改写历史或改变可见性。
- 建议提交：feat(core): implement tuning math and pitch contracts。
- 本记录写于提交/push 前；最终 SHA、远端核实和本轮 CI 由最终报告给出，下一轮从 Git/Actions 回填。不要将提交自身 SHA 写入自身。
- Windows CI 使用统一脚本，会自动覆盖新 workspace 成员；只执行工程检查/release，不运行本机 GUI。必须查当前 SHA 的实际结论。

## 唯一下一任务

**T06 — 基础 YIN 单音算法。**

1. 读取 AGENTS、README、完整主任务书、STATUS/HANDOFF、TUNING_CORE、TESTING、ENVIRONMENT 和 DECISIONS；核对 Git、分支、diff、远端，安全 fetch，核实本轮提交/CI。
2. 复跑 cargo test -p tuner-core --locked 与项目 check-env/bootstrap/test/build/test-ui；核对锁文件、缓存、原任务书 D002 哈希。前置失败先修复。
3. 按 T06 实现 YIN 差分/累计归一化/lag 边界/谷值/插值/能量门限/置信度；使用实际采样率与配置频率范围，返回零或一个结果，不接入设备或 UI。
4. 用本轮 test_signal 在 44.1/48 kHz 验证 A0/A1/A2/A4/C4、非标准频率与空/静音/无效输入。稳定合成纯音目标 ±1 cent；未达标如实记录并拆修复，不降低断言或假称可用。
5. 本轮不提前做 T07 复杂鲁棒性或实时 worker；UI 测试继续后台。结束更新交接、真实验收、显式 stage、commit/push、核实远端/CI。

当前仍只能体验 UI Demo 与核心计算，不能测量真实乐器音高。
