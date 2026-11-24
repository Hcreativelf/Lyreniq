# Lyreniq

Lyreniq 是 Universal Tuner 通用乐器调音与音准分析项目的软件名称。目标是提供现代、专业的本地调音工具；首个交付平台为 Windows 桌面。

当前已通过 T03 工作区壳验收：四个页面导航、深浅主题、统一组件和可访问键盘操作。T02 工程基线、release 构建及后台原生 UI 回归保持通过。调音、文件分析、参考音和安装包尚未实现，界面明确显示尚未开放。

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
| [UI_GUIDE](docs/UI_GUIDE.md) | 设计 token、页面与组件、可访问性、开源参考及许可 |

根目录的 `UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` 保留为用户原始输入；`docs/` 版本是后续维护入口。T00 时二者字节一致。

## 计划范围

- Windows MVP：实时麦克风单音调音、音名/Hz/cents、频谱、本地 WAV/MP3/FLAC 分析、CSV、乐器预设、参考音和本地设置。
- 所有音频完全本地处理，无账号、遥测或上传；深色默认、浅色可选，采用专业音频工具的视觉和交互标准。
- 候选技术：Rust、Tauri 2、React、TypeScript、Vite。具体版本和兼容性在 T01 实测并锁定。
- 其他平台及复音检测留待 Windows MVP 后的独立任务。

以上音频能力是计划功能；当前仅实现工作区、主题和真实桌面桥接状态。

## 开发、安装与验证

已建立 Cargo/pnpm workspace，桌面应用位于 `apps/desktop`。Rust、Node、pnpm 与直接依赖版本已明确约束；Cargo.lock 和 pnpm-lock.yaml 用于复现依赖。本机缓存集中管理及其他电脑的回退方式见 ENVIRONMENT，不要求其他机器存在同样的 D 盘目录。

安装共享工具链并核对 [环境基线](docs/ENVIRONMENT.md) 后，在项目根目录运行：

```powershell
pwsh -NoProfile -File scripts/check-env.ps1
pwsh -NoProfile -File scripts/bootstrap.ps1
pwsh -NoProfile -File scripts/test.ps1
pwsh -NoProfile -File scripts/build.ps1
pwsh -NoProfile -File scripts/test-ui.ps1
pwsh -NoProfile -File scripts/dev.ps1
```

也支持 `powershell.exe -NoProfile -File scripts/<名称>.ps1`（Windows PowerShell 5.1）。脚本从自身位置推导项目根目录，可在其他工作目录用绝对脚本路径调用。check-env 只检查；bootstrap 使用锁文件下载项目依赖；test 执行 PowerShell 解析、Rust fmt/clippy/test 和前端 lint/typecheck/test；build 生成 `target/release/lyreniq-desktop.exe`，当前明确跳过安装包；dev 启动原生窗口及本机 Vite。

日志保存在 `.local/logs`，不提交。首次 bootstrap 需要网络；工具链缺失由检查报告，不静默安装系统组件。CI 在 Windows runner 上执行同一套脚本；`-SkipWebView2` 仅供无 GUI 的构建/CI，明确跳过运行时检查，不能代替原生窗口验收。CI 实际结果见 TESTING/交付报告。

日常 UI 验证优先运行 `test-ui.ps1`：真实 Tauri 窗口保持隐藏、不聚焦，通过本机 WebView2 协议检查真实 Rust IPC、重连、正常关闭和窗口尺寸/主题/导航。无需下载额外浏览器，不操作系统鼠标键盘。测试构建、数据、临时配置及原始结果在 `target/ui-smoke` 和 `.local/ui-smoke`；测试专用权限不写入生产 capabilities。T03 [深色截图](docs/evidence/T03/dark-1024x700-tuner.png)、[浅色截图](docs/evidence/T03/light-1440x900-tuner.png)和[结果](docs/evidence/T03/result.json)已保存。

`dev.ps1` 默认会打开普通窗口，使用桌面时不要运行它；后台测试会通过专用参数调用同一脚本。标题栏实体交互、系统 DPI、权限弹窗和音频设备另做人工/真机验收。当前 GitHub CI 执行自动化检查与 release 构建，后台原生 UI smoke test 已在本机实测，尚未纳入 CI。

低层命令仍可使用：`pnpm build` 构建前端，`pnpm dev` 启动桌面，`pnpm test` 仅跑前端测试；完整回归使用 scripts/test.ps1。窗口最小 1024×700，默认深色，可在侧栏或设置页切换；外观仅本次会话保留。底部显示真实 Rust 版本/平台及连接状态；普通浏览器预览没有桌面后端。

当前可执行的文档与 Git 检查见 [TESTING](docs/TESTING.md)。更换对话前后以仓库文件和 Git 记录恢复，不依赖旧聊天记忆。

## Git 与许可证

远端为用户授权的 [Hcreativelf/Lyreniq](https://github.com/Hcreativelf/Lyreniq)，本地 `main` 跟踪 `origin/main`。T00 已成功推送并核实远端 hash；当前任务提交/推送状态见 STATUS 与 HANDOFF。

项目许可证尚未确定，当前没有声明开源许可。新增 Lucide 图标的完整 [ISC/Feather MIT 声明](docs/licenses/lucide-react-1.75.0.txt)已保留，设计参考及采用边界见 UI_GUIDE。
