# Lyreniq 交接记录

## 本轮任务与起点

- 日期：2026-11-24（Asia/Shanghai）；目录 D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点：def1765d43745d9b310fbeba511ff257ee140f9a（T02-a）。开始时工作区干净，fetch 后与远端一致。
- 已核实上轮 Windows CI 实际 success：[run 45338108012](https://github.com/Hcreativelf/Lyreniq/actions/runs/45338108012)，包含固定工具安装、项目外环境检查、锁定安装、格式/lint/类型/2+3 测试及 release 构建。
- 当前完成：T02-b 后台真实原生 UI 复验，T02 父任务 VERIFIED。本轮没有实施 T03。
- 用户正在使用桌面，明确要求后台测试。后续默认 scripts/test-ui.ps1，不前置窗口、不注入系统鼠标键盘。普通 dev.ps1 会显示窗口，须尊重该约束。

## 本轮交付

- scripts/test-ui.ps1、ui-smoke.mjs：Playwright 通过 loopback CDP 连接真实 Tauri/WebView2；无需下载额外浏览器。
- dev.ps1 增加 BackgroundTestConfig 参数，拒绝没有明确 visible=false/focus=false 的测试配置；普通开发启动行为不变。
- 测试配置、独立数据/profile、随机端口、日志和结果位于 .local/ui-smoke；专用构建目录 target/ui-smoke。正式 tauri.conf.json 与 capabilities 未修改。
- 仅测试配置授予窗口状态读取和 close 权限。实际核对 native is_visible/is_focused，按钮通过真正 Rust IPC 返回版本；截图来自渲染器，不是用户桌面。
- 正常关闭后检查 dev 退出 0、Vite/CDP 端口释放；失败时只清理本脚本创建的进程树。
- playwright-core 1.65.1 精确锁定于 package.json/pnpm-lock.yaml；test.ps1 加入 MJS 语法和 Biome recommended 检查。
- 脱敏证据：[渲染截图](evidence/T02-b/native-window.png)、[结果](evidence/T02-b/result.json)。AGENTS/README/ENVIRONMENT/DECISIONS/TESTING/STATUS 已同步。

## 实际验证

| 命令/检查 | 结果 |
|---|---|
| 上轮 Git/远端与 T02-a CI 核实 | 与已推送起点一致；CI success，实际记录见上 |
| check-env/bootstrap/test/build 恢复检查 | 全部通过后实施 T02-b |
| scripts/test-ui.ps1（PowerShell 7） | 最终方案 5 项检查通过，退出 0 |
| 在 D:\ 执行 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\test-ui.ps1 | 最终脚本 5 项检查通过，退出 0；8.345 秒 |
| 真实页面与 IPC | Lyreniq 页面、Rust Lyreniq 0.1.0 · windows、重连按钮实际 IPC 响应 200/ok 与返回值均通过 |
| 原生窗口状态 | 交互和截图前后 visible=false、focused=false |
| 渲染及退出 | 1000×700 无溢出；close 后 dev 退出 0；两个端口释放 |
| 修改后 bootstrap.ps1 | 退出 0；frozen pnpm + cargo fetch --locked 通过 |
| 修改后 test.ps1 | 退出 0；PS/MJS 解析、MJS lint、fmt/clippy、2 Rust + 3 TS、前端 lint/typecheck 通过 |
| 修改后 build.ps1 | 退出 0；release exe 生成；安装包明确留给 T24 |

中间尝试通过替换内部 invoke 观测按钮调用时，Tauri 的只读属性保护使测试超时；该次保留为 failed。最终改为非侵入监听真实 IPC 网络响应，未更改 Tauri 保护或削弱返回值断言，随后两次完整运行均通过。

最终本机原始结果：.local/ui-smoke/2026-11-24T06-49-07-123Z/result.json；latest.json 指向同轮。日志：.local/logs/ui-smoke.log、dev-ui-smoke.log、bootstrap.log、test.log、build.log。原始路径和 profile 不提交，仅提交脱敏证据。

## Git 和 CI

- 授权远端：https://github.com/Hcreativelf/Lyreniq.git。保持当前仓库可见性，不强推或改写历史。
- 本记录写于 T02-b 提交/push 前；最终 hash、push 及本轮 CI 实际结果由最终报告给出，下一轮从 Git/Actions 按 SHA 回填。
- push 后必须核实 git ls-remote --heads origin main 与本地 HEAD 一致，再检查该 SHA 的 Windows checks。
- 现有 CI 执行工程检查与 release 构建；后台原生 UI 在本机通过，尚未纳入云端 CI，不能声称 CI 测过 GUI。

## 唯一下一任务与恢复

**T03 — 现代 UI 设计系统与工作区壳。** 先复验，不叠加未通过的基线。

1. 读取 AGENTS、README、完整主任务书、STATUS/HANDOFF，以及 DECISIONS、TESTING、ENVIRONMENT；核对实际 Git、分支、diff、远端，安全 fetch 并核实本轮提交/CI。
2. 从项目外运行 check-env.ps1；运行 bootstrap.ps1、test.ps1、build.ps1、test-ui.ps1，所有结果应退出 0。确认两份锁文件跟踪、缓存忽略、两份原始任务书 SHA-256 仍为 D002 基准。
3. T03 按任务书验证开源设计参考许可/成熟度，建立本地 token、深浅主题、基础组件、导航空壳及 UI_GUIDE；不实现音频或分析。
4. UI 验证优先扩展后台测试至 T03 要求的两种主题、1024×700/1440×900、导航与焦点。后台协议的页面内键盘输入不等于系统键盘注入。需要前台系统/真机交互时，等用户明确允许的空闲时段并单独记录。
5. 每项结束更新 STATUS/HANDOFF，实际验收后显式 stage、commit/push、核实远端及 CI。

未实现或验证：调音、音频设备、主题/导航壳、DPI 矩阵、标题栏实体交互、系统权限弹窗、文件、设置、安装包及生产离线。项目许可证、正式分发标识仍待确定，不擅自补许可证或发布软件。
