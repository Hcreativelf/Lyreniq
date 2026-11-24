# Lyreniq

Lyreniq 是 Universal Tuner 通用乐器调音与音准分析项目的软件名称。目标是提供现代、专业的本地调音工具；首个交付平台为 Windows 桌面。

当前仅完成 T00 的文档和 Git 起点初始化，尚无应用源码、可运行程序、安装包或已验证的平台支持。

## 项目入口

| 文档 | 用途 |
|---|---|
| [AGENTS.md](AGENTS.md) | 每轮恢复、任务范围、验收和 Git 规则 |
| [开发任务书](docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md) | 完整需求、架构、T00–T25 任务及验收标准 |
| [STATUS](docs/STATUS.md) | 当前事实、验收状态与唯一下一任务 |
| [HANDOFF](docs/HANDOFF.md) | 本轮证据、Git 状态及下一轮恢复步骤 |
| [DECISIONS](docs/DECISIONS.md) | 命名、文档位置和未决事项 |
| [TESTING](docs/TESTING.md) | 已执行检查、未验证范围与复验方式 |

根目录的 `UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 保留为用户原始输入；`docs/` 版本是后续维护入口。T00 时二者字节一致。

## 计划范围

- Windows MVP：实时麦克风单音调音、音名/Hz/cents、频谱、本地 WAV/MP3/FLAC 分析、CSV、乐器预设、参考音和本地设置。
- 所有音频完全本地处理，无账号、遥测或上传；深色默认、浅色可选，采用专业音频工具的视觉和交互标准。
- 候选技术：Rust、Tauri 2、React、TypeScript、Vite。具体版本和兼容性在 T01 实测并锁定。
- 其他平台及复音检测留待 Windows MVP 后的独立任务。

以上是计划功能，当前均未实现。

## 开发、安装与验证

T00 不安装工具链、不创建应用工程。目前没有 Cargo/package 清单、锁文件、开发脚本或 CI，不能运行应用、构建或执行音频测试。

下一项 T01 验证 Windows 工具链并建立真实最小窗口；T02 建立项目内 check-env/bootstrap/dev/test/build 脚本及回归与 CI 基线。后续脚本以项目位置推导路径，工具和缓存尽量集中在项目内。

当前可执行的文档与 Git 检查见 [TESTING](docs/TESTING.md)。更换对话前后以仓库文件和 Git 记录恢复，不依赖旧聊天记忆。

## Git 与许可证

当前本地分支为 `main`，尚未配置 remote，未推送 GitHub。需要用户提供已授权的仓库 URL；若需新建，先确定所有者、名称和可见性。不要将未推送状态写成云端备份完成。

项目许可证尚未确定，当前没有声明开源许可。第三方组件及资产在实际引入时核对许可证并记录。
