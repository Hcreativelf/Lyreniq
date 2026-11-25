# Lyreniq 交接记录

## 任务与起点

- 2026-11-25（Asia/Shanghai），D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点 58830d6536561526af734ee55ecb201a6984225b（T06），开始干净，安全 fetch 后 ahead/behind=0/0；[T06 CI 45583704093](https://github.com/Hcreativelf/Lyreniq/actions/runs/45583704093) 按 SHA 核实 completed/success。
- 先从 D:\ 复跑 T06 核心（加 --manifest-path）及 check-env/bootstrap/test/build/test-ui；27 核心+2 文档、2 桌面 Rust、26 TS、release、16 工作区/44 Demo 通过。基线 UI .local/ui-smoke/2026-11-25T09-25-15-825Z。
- 本轮完成 T07 — YIN 鲁棒性与运行性能；T08 未开始。用户报告无麦克风，本轮未枚举/采集，不推断 OS 输入数量；合成 PCM 不发声。

## 本轮交付

- YinDetector::detect：逐 lag 提前结束、四路差分累加、二/三倍周期的有限验证、较宽低频五点四次拟合、近数值零残差的整数周期处理。默认参数和严格范围不变，无 FFT/unsafe/新依赖。
- detect_reference 保留 T06 原算法；26 个冻结 CSV 频率在 2e-9 Hz 容差内重现，不修改 T06 证据。
- PitchSmoother/SmoothedPitch：原始检测和展示频率分开，三帧中值；>50 cents 跳变、无效/错误清空历史；stop/session/rate 改变需未来 worker 调 reset。
- 有界固定种子白噪声；32 场景×2 采样率×5 重叠窗口×2 引擎的 640 行 CSV，以及覆盖率/误差/八度错误摘要。原始值不被平滑覆盖。
- examples/yin_report.rs、scripts/bench-yin.ps1：顺序隐藏进程的 Release 误差/耗时/CPU 对照；[YIN_VALIDATION](YIN_VALIDATION.md) 记录算法、口径、夹具和限制。
- [T07 结果](evidence/T07/result.json)、frame-errors.csv、summary.json、timings.csv、machine.json；README/TUNING_CORE/TESTING/STATUS/D017 已同步。两份锁文件、UI、权限、冻结任务书未变，核心仍无外部 crate。

## 实际验收与命令

| 命令 | 实际结果 |
|---|---|
| cargo test -p tuner-core --release --locked | 0，4 单元+33 集成+2 文档，即 37 核心测试+2 文档 |
| cargo clippy -p tuner-core --all-targets --locked -- -D warnings | 0，无警告；包含例子/测试目标 |
| 从 D:\ 经 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\check-env.ps1 / bootstrap.ps1 | 均 0，环境/frozen pnpm/locked Cargo |
| 同上 test.ps1 | 0，fmt/clippy、37 核心+2 文档、2 桌面 Rust、26 TS、类型/lint/脚本检查 |
| 同上 build.ps1 | 0，release exe；JS 仍 247.49 kB/gzip 78.48 kB，未构建 T24 安装包 |
| 同上 test-ui.ps1 | 0，16 组检查、16 工作区/44 Demo、真实 IPC、隐藏不聚焦、正常退出/端口释放 |
| 同上 bench-yin.ps1 | 0，新脚本从项目外 Windows PowerShell 5.1 实际运行；顺序 CreateNoWindow 进程，无构建并发 |
| 全部 scripts/*.ps1 的 PowerShell AST 检查 | 通过，包含后加的基准脚本 |

最终 UI .local/ui-smoke/2026-11-25T09-38-53-078Z，suite 沿用 T04，不重复提交相同截图。完整回归后只新增基准包装脚本和修改注释；新脚本已运行/解析，算法未再改。原生 DPI/标题栏/设备权限未验，不用渲染比例模拟代替。

正式基准 .local/dsp-bench/2026-11-25T09-40-27-263Z，latest.json 保存机器信息和位置；full.csv 的 FRAME 行为逐帧误差，正式 timings.csv 取 cpu-T06.csv/cpu-T07.csv 的 BENCH 行。较早试跑和曾与构建并发的报告仅保留在忽略日志，未用于正式性能表。

过程失败均已修复：首次固定矩阵中 10 dB 低音约 43.55 cents 超过 10 cents 断言，扩大周期拟合跨度后原断言通过；Clippy 常量 chunks_exact 提示改为 as_chunks，没有禁用检查或放宽精度。

## 实测范围与限制

- 44.1/48 kHz、默认 300 ms/20–5000 Hz/阈值 0.1/−60 dBFS，步长 20 ms；固定种子与分量见 YIN_VALIDATION/源码。重叠窗口不是独立随机试验。
- 130 纯音窗口和 40 泛音窗口均 ≤1 cent；20 Hz 五起点两采样率均有效，5000 Hz 相位零误差约 −0.162533/−0.093746 cents。不能外推全连续频段或真实声学精度。
- 20/10 dB 接受帧最坏约 0.299696/5.085480 cents，通过原 5/10 cents 门槛；0 dB 30 窗口无音高；静音/常量 DC/低电平/纯噪声 40 窗口无假读数。
- 已知目标 280 窗口中，T06 接受 244，T07 250（89.2857%）；T07 30 个拒绝均为 0 dB。泛音子集 T06 八度 10/40、粗大错误 20/40，T07 均 0/40。缺失误差不填零，定义和全部数据见报告。
- i5-14600KF/20 逻辑核、Windows NT 10.0.26300.0、Rust 1.100.0 Release；用户继续使用桌面，非隔离基准。每场景 3 次预热+64 次测量，T07 最坏 p95 3.5552 ms、单次 3.6437 ms，检测核心满足 33.3 ms 预算。
- 独立批进程 T06 CPU/墙钟=6.78125/6.85769 s，T07=1.265625/1.27486 s；各 640 次计时+30 预热，CPU 包含生成/输出，不等于帧分位数或未来定时应用整机占用。整个 worker/FFT/IPC 尚未实现/实测。
- T10 建议：常规显式 100 ms/50–5000 Hz、20–30 Hz；低音 300 ms/20–5000 Hz、10–20 Hz。耗时超步长只保留最新窗口并降频，禁止积压。这是后续策略，不是当前已实现 worker。
- 无新工具/依赖/缓存设置；沿用 D:\4-stuff\Cache 和兼容链接，其他机器仍使用项目回退。原任务书哈希仍 C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF。
- 仍无 FFT 频谱、音频采集、worker、IPC 或 UI 接入；界面只可体验显式 Demo。更多噪声/音色/非谐和乐器、真机时钟/稳定性、生产离线和安装未验。

## Git 与 CI

- 已授权远端 https://github.com/Hcreativelf/Lyreniq.git；建议提交 perf(dsp): improve pitch robustness and frame budget，不强推、不改写历史。
- 本记录写于本轮提交/push 前；最终 SHA、远端核实和 CI 在最终报告给出，下一轮回填，不能把提交自身 SHA 写入自身。
- 连接器 github_fetch 可 GET actions/runs?head_sha=<SHA>&per_page=5；解析 structuredContent.content，再查 run/job，避免输出整个响应。当前 CI 自动执行核心/前端检查和 release，不执行本机 UI 或性能基准。

## 唯一下一任务

**T08 — FFT 频谱核心。**

1. 读取 AGENTS、README、完整主任务书、STATUS/HANDOFF、TUNING_CORE、YIN_VALIDATION、TESTING、ENVIRONMENT、DECISIONS；核对实际 Git/分支/diff/远端，安全 fetch 并核实 T07 commit/CI。
2. 复跑 cargo test -p tuner-core --locked（外部目录加 --manifest-path）、check-env/bootstrap/test/build/test-ui，以及 bench-yin.ps1。性能基准单独运行，勿与构建并发；核对冻结文档哈希、锁文件和缓存。
3. 按 T08 核实并锁定 RustFFT，引入所需依赖；实现 Hann 窗、正确幅值/窗增益归一化、频率轴、相对 dBFS 与可复用缓冲/SpectrumFrame，不能拿最大谱峰替代基频。
4. 验证整 bin/非整 bin、两音、静音、有限值与配置边界；不接 UI，不做 T09 设备。新包缓存沿用本机集中路径，不能把 .cargo/config/凭据/程序当缓存。
5. 独立验收、后台回归、更新交接、显式 stage、commit/push，核实当前 SHA 的远端和 CI。
