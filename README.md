# Lyreniq

Lyreniq 是 Universal Tuner 通用乐器调音与音准分析项目的软件名称。目标是提供现代、专业的本地调音工具；首个交付平台为 Windows 桌面。

当前已通过 T01 的 Windows Tauri + React 最小窗口验收；真实 Rust 桥接和重新连接按钮已验证，调音功能和安装包尚未实现。

## 项目入口

| 文档 | 用途 |
|---|---|
| [AGENTS.md](AGENTS.md) | 每轮恢复、任务范围、验收和 Git 规则 |
| [开发任务书](docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md) | 完整需求、架构、T00–T25 任务及验收标准 |
| [STATUS](docs/STATUS.md) | 当前事实、验收状态与唯一下一任务 |
| [HANDOFF](docs/HANDOFF.md) | 本轮证据、Git 状态及下一轮恢复步骤 |
| [DECISIONS](docs/DECISIONS.md) | 命名、文档位置和未决事项 |
| [TESTING](docs/TESTING.md) | 已执行检查、未验证范围与复验方式 |
| [ENVIRONMENT](docs/ENVIRONMENT.md) | 实测工具版本、共享安装与项目内配置 |

根目录的 `UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 保留为用户原始输入；`docs/` 版本是后续维护入口。T00 时二者字节一致。

## 计划范围

- Windows MVP：实时麦克风单音调音、音名/Hz/cents、频谱、本地 WAV/MP3/FLAC 分析、CSV、乐器预设、参考音和本地设置。
- 所有音频完全本地处理，无账号、遥测或上传；深色默认、浅色可选，采用专业音频工具的视觉和交互标准。
- 候选技术：Rust、Tauri 2、React、TypeScript、Vite。具体版本和兼容性在 T01 实测并锁定。
- 其他平台及复音检测留待 Windows MVP 后的独立任务。

以上是计划功能，当前均未实现。

## 开发、安装与验证

已建立 Cargo/pnpm workspace，仅包含 `apps/desktop` 最小窗口。Rust、Node、pnpm 与直接依赖版本已明确约束；Cargo.lock 和 pnpm-lock.yaml 用于复现依赖。

安装共享工具链并核对 [环境基线](docs/ENVIRONMENT.md) 后，在项目根目录运行：

```powershell
pnpm install --frozen-lockfile
cargo check --workspace --locked
pnpm build
pnpm dev
```

`pnpm build` 当前构建前端；`pnpm dev` 启动真实 Tauri 原生窗口及本机 Vite 开发服务器。原生窗口显示 Rust 返回的版本/平台信息，可点击重新验证连接；普通浏览器预览不具备桌面后端。

统一 PowerShell 脚本、自动化测试及 CI 在 T02 建立，当前尚不存在。Tauri bundle 暂未启用；T01 不生成或发布安装包。UI 设计系统从 T03 开始。

当前可执行的文档与 Git 检查见 [TESTING](docs/TESTING.md)。更换对话前后以仓库文件和 Git 记录恢复，不依赖旧聊天记忆。

## Git 与许可证

远端为用户授权的 [Hcreativelf/Lyreniq](https://github.com/Hcreativelf/Lyreniq)，本地 `main` 跟踪 `origin/main`。T00 已成功推送并核实远端 hash；当前任务提交/推送状态见 STATUS 与 HANDOFF。

项目许可证尚未确定，当前没有声明开源许可。第三方组件及资产在实际引入时核对许可证并记录。
