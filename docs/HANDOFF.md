# Lyreniq 交接记录

## 任务与起点

- 日期：2026-11-25（Asia/Shanghai）；D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点：af37a77a126d4b90e0efede36c1b83b315ffca31（T05）。开始干净，安全 fetch 后 ahead/behind=0/0；[T05 CI run 45554383743](https://github.com/Hcreativelf/Lyreniq/actions/runs/45554383743) 按 SHA 核实 success，唯一 job 及全部步骤成功。
- 实施前先重跑 T05 核心与项目外五脚本；17 核心集成+1 文档、2 桌面 Rust+26 TS、release、16 工作区/44 Demo 全部通过。基线 UI：.local/ui-smoke/2026-11-25T09-02-16-280Z。
- 本轮仅完成 T06 基础 YIN，必需离散纯音范围通过验收；边界失败单列。T07 未开始；无 FFT、设备、实时线程或 UI 桥接。
- 用户报告本机无麦克风，本轮未枚举或采集，不据此断言 OS 输入设备数量。合成 PCM 测试无需麦克风，不播放声音。

## 本轮交付

- crates/tuner-core/src/yin.rs：YinConfig/YinDetector/PitchDetection。实际采样率、固定可配置窗口、去 DC、固定比较段差分/累计归一化、首个合格谷、CMNDF 谷深/原始差分周期抛物线插值、RMS 门限及置信度。
- 默认 300 ms、20–5000 Hz、阈值 0.1、−60 dBFS；配置/PCM/容量/Nyquist 校验，三组 f64 缓冲有界并复用。无合格谷返回 LowConfidence；空/短/静音/过低电平或频率越界返回 NoPitch；只有 Detected 携带一个音高。
- PitchDetection 不制造 session/timestamp；pitches 为 0/1 slice，RMS 为去 DC 的 dBFS，零能量/不完整窗口为 None。错误通过 Result 交给后续 worker。confidence 是周期性，不代表正确概率或音分误差界。
- 新增 2 单元、8 集成及 1 可执行文档示例；核心总计 2 单元+25 集成+2 文档。无新外部依赖，桌面没有链接核心，两份锁文件、UI/脚本/权限均未更改。
- [误差 CSV](evidence/T06/pure-tone-errors.csv) 从真实测试 stdout 提取 26 行：22 个必需频点组合和 4 个边界观察；[结果摘要](evidence/T06/result.json)。README、TUNING_CORE、TESTING、STATUS、D016 同步。

## 实际命令与结果

| 命令/检查 | 结果 |
|---|---|
| cargo test -p tuner-core --locked -- --nocapture | 0；2 单元+25 集成+当时 1 文档；新增示例随后单独验证 |
| cargo test -p tuner-core --test yin --locked -- --nocapture --test-threads=1 | 0；8 测试；26 行 CSV，必需 22 组合保持 ≤1 cent 断言 |
| cargo clippy -p tuner-core --all-targets --locked -- -D warnings | 0，无警告 |
| cargo test -p tuner-core --doc --locked | 0，2 个可执行示例 |
| cargo tree -p tuner-core --edges normal --locked | 0，只有自身，零外部依赖 |
| cargo test -p tuner-core --test yin --locked boundary_characterization -- --nocapture | 0，针对严格默认范围补充状态观察，日志 .local/logs/T06-boundary.log |
| 从 D:\ 用 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\check-env.ps1 / bootstrap.ps1 | 均 0；环境、frozen pnpm、locked Cargo 通过 |
| 同上 test.ps1 | 0；fmt/clippy、2 核心单元+25 集成+2 文档、2 桌面 Rust、26 TS、类型/lint/脚本解析通过 |
| 同上 build.ps1 | 0；release exe 生成，UI 资产仍为 JS 247.49 kB/gzip 78.48 kB；未构建 T24 安装包 |
| 同上 test-ui.ps1 | 0；T04 suite 16 组后台检查、16 工作区/44 Demo、真实 IPC、正常退出/端口释放通过 |

最终 UI 原始运行 .local/ui-smoke/2026-11-25T09-10-17-677Z，全程 native hidden/unfocused，无系统鼠标键盘注入。无新 UI，不重复提交截图。原生 DPI/标题栏/设备权限仍未验，渲染器比例模拟单列。标准脚本日志与测试原始 stdout 保持忽略。

过程说明：首次从 D:\ 单独调用 Cargo 未指定 manifest 而退出 101，补 --manifest-path D:\2-workspace\Lyreniq\Cargo.toml 后核心和完整基线均通过；首次 CSV 提取遗漏带测试名的首行导致行数检查失败，改为查找行内 YIN, 后提取完整 26 行。没有调整精度断言或虚报失败命令成功。

## 精度与未完成范围

- 44.1/48 kHz，各 11 个离散频点：27.5/55/110/123.45/220/261.6255653005986/432.123/440/442/880/1760 Hz；零初相、幅度 0.5、300 ms。22 组合全部 |error|≤1 cent，最坏 0.053691901 cent（48 kHz/1760 Hz）。这不是连续区间或声学测量保证。
- 边界另用 19.9–5100 Hz 配置观察：20 Hz 两例误差小于 0.000001 cent；5000 Hz 在 44.1 kHz 为 −1.319302299 cents，未达 ±1 cent，48 kHz 为 −0.959541949 cents。不能宣称全默认频段通过。
- 默认严格 20–5000 Hz 下，20 Hz 两例因极小负插值偏差返回 NoPitch，5000 Hz 两例 Detected。不得夹断或隐藏此限制；T07 纳入修复/有效范围决策，保留原始对照且不降低断言。
- N=floor(rate×window_ms/1000)，最多 65,536 样本；须满足 2×(ceil(rate/min_hz)+1)≤N。默认 20 Hz 与 60 ms 不相容，拒绝而非缩小范围；192 kHz 本轮仅校验窗口几何。
- 直接 O(W×lag) 参考实现未做 CPU 预算/实时性能验收，不可放音频回调。不保证弱基频/噪声/衰减/复音识别，无时间平滑或状态记忆。
- 无麦克风不影响当前任务。T09 实现真实枚举/选择/无设备分支，真实采集/切换/拔出需要可用设备后独立验收。T11 再做 Rust/TS 序列化映射，T12 再替换 UI 数据来源。
- 当前 UI 仍只有显式 Demo；不能用于测量真实乐器。生产离线、原生 DPI、安装仍未验。

## 环境与 Git

- 未安装新工具/包，两份锁文件未改；缓存继续使用 D:\4-stuff\Cache。项目 pnpm store、Cargo registry junction 及 npm/pnpm cache 实际值已检查，Vite/Vitest 沿用本机 cacheRoot。其他机器使用项目回退，不照搬盘符。
- 两份冻结任务书仍为 SHA-256 C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF。
- 授权远端 https://github.com/Hcreativelf/Lyreniq.git；建议提交 feat(dsp): implement and test YIN pitch detection；不强推、不改写历史。
- 本记录在提交/push 前写入，最终 SHA、远端核实与本轮 CI 由最终报告提供，下一轮回填；不要将提交自身 SHA 写入自身。
- GitHub fetch 工具已支持 actions/runs 集合和子资源。用 https://api.github.com/repos/Hcreativelf/Lyreniq/actions/runs?head_sha=<SHA>&per_page=5 查本轮 push；解析 structuredContent.content JSON，避免输出整个响应；再查 run/job 实际结论。无需使用此前可能命中共享 IP 限流的匿名 REST。

## 唯一下一任务

**T07 — YIN 鲁棒性与运行性能。**

1. 读取 AGENTS、README、完整主任务书、STATUS/HANDOFF、TUNING_CORE、TESTING、ENVIRONMENT、DECISIONS；核对当前分支/commit/diff/远端，安全 fetch 并核实 T06 commit/CI。
2. 复跑 cargo test -p tuner-core --locked（外部目录须加 --manifest-path）及 check-env/bootstrap/test/build/test-ui；复验 22 必需点和已记录边界，核对冻结文档哈希与缓存配置。失败先恢复基线。
3. 按 T07 增加弱基频/强泛音、固定噪声、衰减/静音/DC、长窗口测试；把 20 Hz 严格边界拒绝和 44.1 kHz/5000 Hz 未达标作为显式对照项，修复或明确支持范围。
4. 实测 CPU/帧耗时、p50/p95、误差/八度错误率/有效率，必要时优化并对照本轮参考；加入适当平滑/无效帧策略，不能把不同音符平均成假读数。预算达不到时明确低音降级方案。
5. 不提前做 T08 频谱功能、T09 设备或 T10 worker；UI 保持后台。独立验收、更新交接、显式 stage、commit/push、核实远端和当前 SHA 的 CI。
