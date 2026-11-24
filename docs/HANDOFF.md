# Lyreniq 交接记录

## 本轮任务与起点

- 日期：2026-11-24（Asia/Shanghai）；目录 D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点：1355bd94121e4bd8dec8b82e15dee765f8d4106e（T02-b）。工作区起初干净；安全 fetch 后 ahead/behind=0/0。
- 上轮 [Windows checks run 45345059150](https://github.com/Hcreativelf/Lyreniq/actions/runs/45345059150) 已按起点 SHA 核实 completed/success。
- 实施前从 D:\ 复跑 check-env/bootstrap/test/build/test-ui，2 Rust + 3 TS、release 和原有 5 项后台原生 UI 检查通过。
- 本轮只完成 T03 — 现代 UI 设计系统与工作区壳；T04 未开始。用户要求的缓存偏好同步落实为本机配置与兼容链接。

## 交付

- 四个工作区：实时调音、文件分析、参考音、设置；原生语义导航、aria-current、跳转链接、切页后 h1 聚焦。
- 深浅主题、token、Button/Badge、统一图标与原创线条标记；主题仅本次会话保留。中文文案集中 content/zh-CN.ts。
- 未开放动作明确禁用，无模拟测量、音频采集、文件读取或播放；底部只显示真实 Rust runtime_label 的连接状态/重试。
- 窗口默认 1200×700，最小 1024×700；后台测试通过真实 set_size 检查两种尺寸、两主题和四页共 16 组合。
- ui-shell-checks.mjs 覆盖矩阵、页面内 CDP 键盘/焦点、文字 token 对比度、125%/150% 渲染比例模拟；测试权限只在 .local 配置，不扩宽生产 ACL。
- lucide-react 1.75.0 精确锁定，完整 ISC/Feather MIT 声明保留在 docs/licenses。开源参考的版本、许可、活跃度及借鉴点见 UI_GUIDE；不复制 GPL 代码或品牌。
- UI_GUIDE、README、ENVIRONMENT、DECISIONS、TESTING、STATUS 已同步；[全部脱敏截图与结果](evidence/T03/result.json)保存在 docs/evidence/T03。
- 两份原始任务书保持冻结，提交前再次核对 SHA-256。

## 实际命令与结果

最终在 D:\ 使用以下命令形式逐个执行：powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\<名称>.ps1。五个脚本及外层执行进程均退出 0。

| 脚本/检查 | 实际结果 |
|---|---|
| check-env | 锁定 Node/pnpm/Rust、MSVC、SDK、WebView2 通过 |
| bootstrap | pnpm frozen-lockfile、cargo fetch --locked 通过；迁移后的缓存可用 |
| test | PS/MJS 解析、Biome、Rust fmt/clippy、2 Rust IPC、类型与 6 TS 测试通过 |
| build | release exe 生成；前端 JS 237.58 kB / gzip 74.95 kB；安装包明确跳过到 T24 |
| test-ui | 11 组检查通过：含 16 GUI 组合、真实 IPC、键盘与焦点、对比度、比例模拟、正常退出/端口释放 |
| PowerShell 7 的前端及后台验证 | 同样通过；最终外部目录完整验收另使用 Windows PowerShell 5.1 |

最终 UI 原始运行：.local/ui-smoke/2026-11-24T11-37-34-367Z。最终截图已复制到 docs/evidence/T03，结果移除了本机临时路径。日志在 .local/logs，保持忽略；不把安装/编译输出当作 GUI 证据。

真实失败与修复：初次 lint 发现分组/列表语义问题，改用 fieldset/ul；新增测试的 exact 参数不属于 Testing Library 类型，已改成默认精确名称匹配；浅色次要文字首次 4.45:1 未达标，调深后 18 组最低 4.8486:1。最后两次完整后台运行通过，没有关闭规则或跳过断言。最新图标曾触发 pnpm 自动发布时间豁免，最终改用 11-18 的 1.75.0 并移除豁免。

## 本机缓存偏好

- 用户已授权当前 Windows 的可迁移缓存集中到 D:\4-stuff\Cache；具体有效路径、兼容链接和跨机器恢复见 ENVIRONMENT，不将这个盘符当作其他电脑前置条件。
- pnpm 保留原项目链接与 storeDir；npm 使用原生用户级 cache 设置；Cargo 只迁移有缓存标签的 registry；Vite/Vitest 使用原生 cacheDir。
- Cargo 10,845、npm 62、旧 Vite/Vitest 21 个文件逐一 SHA-256 核实；原路径均保留 junction。源码、node_modules、程序、凭据、构建产物和测试 profile 仍按用途管理。
- 本机 .local/cache-paths.json 设置 cacheRoot=D:/4-stuff/Cache，忽略不提交；其他 clone/CI 没有此配置时自动用项目 .local/cache。
- 恢复时检查 junction 和实际写入路径；不要因为 pnpm store path 显示逻辑 .local 路径就断言缓存未集中。Cargo 内部缓存布局不是稳定 API，升级时复核。

## Git 与 CI

- 授权仓库：https://github.com/Hcreativelf/Lyreniq.git；不改可见性、不强推、不改写历史。
- 本记录写于提交/push 前；最终提交 hash、远端 SHA 核实和本轮 CI 状态由最终报告给出，下一轮按 Git/Actions 回填，不把自身 hash 写进提交。
- 本轮建议提交：feat(ui): create modern desktop shell and themes。
- CI 运行工程检查与 release 构建，不运行本机后台 GUI。push 后查该 SHA 的实际结果，不能由配置存在推断 success。

## 唯一下一任务

**T04 — 调音器 UI 与状态模型。**

1. 读取 AGENTS、README、完整主任务书、STATUS/HANDOFF、UI_GUIDE、TESTING、ENVIRONMENT 和 DECISIONS；核对实际分支、工作区、远端，安全 fetch，核实本轮 commit 与 CI。
2. 从项目外运行 check-env，随后 bootstrap/test/build/test-ui；必须先恢复通过的 T03 基线，核实两份锁文件跟踪、缓存忽略及任务书 D002 哈希。
3. 依任务书实现音名/Hz/cents/指针/电平/置信度与 idle/listening/no pitch/error，Demo 必须显式标注，生产默认仍为空状态；不接麦克风，不实现 DSP。
4. 复用 token、基础组件与集中中文文案；扩展对应状态测试和后台两主题/小窗口截图。原生 DPI/跨屏、标题栏、设备和生产离线仍待对应真机验收，渲染模拟不可冒充系统实测。
5. UI 默认后台，不前置窗口、不注入系统输入；需用户前台操作时等其明确允许的空闲时段。结束更新交接、实际验收、显式暂存、提交/push、核实远端及 CI。

当前不具备可用调音功能或安装包；许可证、正式分发标识和签名仍未决定。
