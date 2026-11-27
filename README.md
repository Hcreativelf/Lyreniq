# Lyreniq

Lyreniq 是 Universal Tuner 通用乐器调音与音准分析项目的软件名称。目标是提供完全本地、现代专业的乐器调音与音准分析桌面软件；首个交付里程碑为 Windows 10/11 x64 MVP。

项目已于 2026-11-27 升级至 **架构与开发计划 v2.0**（四层架构 + P1–P8 垂直闭环路线），目前已完成 **P1（DSP 2.0 核心）** 与 **P2（流式解码与离线文件分析引擎）**：
- `crates/tuner-core`：基于 `rustfft` 的 **Fast YIN** 与 **双尺度检测**、复用型 **Hann 窗 FFT 频谱分析器**（含对数分桶与谐波查询）、**对数频率域 `PitchTracker`**、**10 种内置乐器预设**、**零分配相位连续 `ToneGenerator`** 与 `serde` 统一数据契约；48 kHz/300 ms 最坏 p95 ~0.16 ms（`MultiScale` 中高音 ~0.04 ms，频谱 4096 点 ~0.056 ms）。
- `crates/audio-decode` & `crates/tuner-engine`：基于 Symphonia 的 WAV/FLAC/MP3 逐包流式解码（含错扩展名探测回退、声道选择、双声道反相抵消告警），恒定内存滑窗离线 `FileAnalyzer`（进度、原子取消、诚实统计）与带配置元数据的 CSV 导出；60 秒 44.1 kHz 长文件 Release 分析仅 ~0.06 s。
- 当前桌面界面仍为 T04 壳层与显式 Demo，尚未接入新引擎；下一步为 **P3（音频 I/O 与实时管线）**。

## 项目入口

| 文档 | 用途 |
|---|---|
| [AGENTS.md](AGENTS.md) | 每轮恢复、任务范围、验收和 Git 规则 |
| [开发计划 v2.0](docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md) | 四层架构、DSP 2.0 设计、Windows MVP 功能与 P1–P8 路线 |
| [STATUS](docs/STATUS.md) | 当前事实、验收状态与唯一下一阶段 |
| [HANDOFF](docs/HANDOFF.md) | 本轮证据、Git 状态及下一轮恢复步骤 |
| [DECISIONS](docs/DECISIONS.md) | 架构与工程决策记录（含 D018 v2.0 修订） |
| [TESTING](docs/TESTING.md) | 已执行检查、实测性能数据与未验证范围 |
| [ENVIRONMENT](docs/ENVIRONMENT.md) | 实测工具版本、共享安装与本机集中缓存 |
| [TUNING_CORE](docs/TUNING_CORE.md) | DSP 2.0 核心公式、模块契约与验证入口 |
| [UI_GUIDE](docs/UI_GUIDE.md) | 设计 token、深浅主题、可访问性与开源许可 |

根目录的 `UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 保留为冻结的 v1.0 原始输入；`docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 是现行维护的 v2.0 主计划。

## 开发、安装与验证

已建立 Cargo/pnpm workspace：桌面应用位于 `apps/desktop`，纯 Rust DSP 2.0 核心位于 `crates/tuner-core`。安装共享工具链并核对 [环境基线](docs/ENVIRONMENT.md) 后，在项目根目录运行：

```powershell
pwsh -NoProfile -File scripts/check-env.ps1
pwsh -NoProfile -File scripts/bootstrap.ps1
pwsh -NoProfile -File scripts/test.ps1
pwsh -NoProfile -File scripts/build.ps1
pwsh -NoProfile -File scripts/test-ui.ps1
pwsh -NoProfile -File scripts/bench-yin.ps1
```

同样兼容 Windows PowerShell 5.1（`powershell.exe -NoProfile -File scripts/<名称>.ps1`）。核心可单独运行 `cargo test -p tuner-core --locked`。

## Git 与许可证

远端为用户授权的 [Hcreativelf/Lyreniq](https://github.com/Hcreativelf/Lyreniq)，本地 `main` 跟踪 `origin/main`。项目许可证尚未确定，当前没有声明开源许可。Lucide 图标完整声明保留在 [docs/licenses/lucide-react-1.75.0.txt](docs/licenses/lucide-react-1.75.0.txt)。
