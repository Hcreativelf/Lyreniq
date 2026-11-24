# Lyreniq 交接记录

## 本轮任务与起点

- 日期：2026-11-24（Asia/Shanghai）；目录 D:\2-workspace\Lyreniq；main 跟踪 origin/main。
- 起点：3c6779919ff169f874d074baac3a7578673a8f76（T03）。工作区起初干净，安全 fetch 后 ahead/behind=0/0。
- 上轮 [Windows CI run 45387529930](https://github.com/Hcreativelf/Lyreniq/actions/runs/45387529930) 已按起点 SHA 核实 completed/success。
- 实施前从 D:\ 复跑 check-env/bootstrap/test/build/test-ui，2 Rust + 6 TS、release 和 T03 全套后台 UI 通过。
- 本轮只完成 T04 — 调音器 UI 与状态模型；T05 未开始。没有新增依赖、系统组件、真实音频权限或缓存迁移。

## 本轮交付

- TunerWorkspace 展示音名/八度、Hz、cents、目标音/频率、指针、电平和置信度；继承 T03 token 与两主题，文案独立集中在 content/tuner.zh-CN.ts，样式在 tuner.css。
- 默认为 unavailable/idle，无测量读数/指针/信号值，真实监听按钮禁用。点击“体验 Demo”才创建模拟适配器，持续显示模拟来源及非麦克风结果说明。
- services/audio-backend.ts 提供展示契约与输入校验；AudioBackend 是 source + subscribe/start/stop。DemoBackend 只在模拟扩展提供 selectScenario；没有设备 API、音频生成、定时器或 DSP。
- 六种状态：idle/listening/no-pitch/low-confidence/detected/error。只有 detected 含音高，其他状态清理旧值；无效/非有限值转错误。置信度是指标，不是准确概率。
- 11 个固定 Demo 场景含 -50/0/+50、±75、A4→C4、等待、静音、低置信度、错误与 idle。指针限幅 ±50，数字完整保留。A4=440 Hz 为默认参考参数，不是默认测量结果。
- Demo 进入后聚焦场景 select，Home/方向键可选，退出后焦点回入口；错误可重试。离页/退出取消订阅并停止，活动标记隔离迟到帧。
- scripts/ui-tuner-checks.mjs 接入原有隐藏原生 smoke；16 工作区回归外再验证 44 Demo 组合，不操作系统鼠标键盘。
- 已更新 README/UI_GUIDE/TESTING/ENVIRONMENT/DECISIONS/STATUS；[T04 脱敏结果](evidence/T04/result.json)及 45 张 PNG 已保存。两份原始任务书与锁文件未改。

## 实际命令与结果

最终在 D:\ 用 powershell.exe -NoProfile -File D:\2-workspace\Lyreniq\scripts\<名称>.ps1 逐个执行，五个脚本及外层进程均退出 0。

| 脚本/检查 | 实际结果 |
|---|---|
| check-env | 锁定工具、MSVC、SDK、WebView2 通过 |
| bootstrap | pnpm frozen-lockfile 与 cargo fetch --locked 通过；集中缓存可用 |
| test | PS/MJS 解析、Biome、Rust fmt/clippy、2 Rust IPC、类型与 26 TS 测试通过 |
| build | release exe 生成；Vite 2,144 模块，JS 247.49 kB / gzip 78.48 kB；安装包留给 T24 |
| test-ui | 16 组检查：16 工作区与 44 Demo 组合、真实只读 Rust IPC、键盘/焦点、状态/数值边界、重试/清理、正常退出/端口释放通过 |

新增 20 TS 测试：10 展示/状态/生命周期，10 适配器/非法输入；原有 6 项 UI 行为和 2 项 Rust 均保留。默认、两主题准确、超量程、静音/错误等截图直接检查；1024×700 的 11 场景均截图，两种大窗口主题另保存准确状态，其余大窗口场景记录 DOM/布局断言。

最终原始 UI 运行：.local/ui-smoke/2026-11-24T13-33-29-937Z；latest.json 汇总该轮。日志位于 .local/logs/bootstrap.log、test.log、build.log、ui-smoke.log、dev-ui-smoke.log。源配置和生产 capabilities 未扩权，测试权限只在忽略的临时配置中。

真实失败与修复：初始项目外 Git 检查遗漏 -C，已纠正，基线无故障。Demo 夹具类型过宽曾使 typecheck 失败，已收窄 detected 类型。首次 native 重试检查早于 React 更新而读到临时空值（.local/ui-smoke/2026-11-24T13-29-56-720Z 为 failed），改用明确状态/焦点完成条件后两次完整后台运行通过，没有固定延时、删除断言或冒充首轮通过。

## 缓存与范围

- 本机缓存偏好继续遵循 ENVIRONMENT；pnpm/Cargo/旧 Vite 兼容链接和 .local/cache-paths.json 实际核实正常，没有新增缓存路径。
- Demo 数据不是音频夹具、实际采集或 DSP 准确度结果。Rust 真实验证只覆盖已有 runtime_label，T04 不增加音频 IPC。
- Windows 原生 DPI/跨屏/标题栏、设备、真实音准、生产断网与安装仍未验。125%/150% 仍仅 CDP 渲染模拟，不能替代系统实测。
- 自身许可证、正式分发标识与签名待定；现有 Lucide ISC/Feather MIT 声明保持不变。

## Git 与 CI

- 授权仓库：https://github.com/Hcreativelf/Lyreniq.git；不强推、不改写历史、不改可见性。
- 本记录写于提交/push 前；最终 hash、远端 SHA 和本轮 CI 状态由最终报告提供，下一轮按 Git/Actions 回填。
- 本轮建议提交：feat(ui): build tuner display and signal states。
- CI 只跑工程检查/release，不跑本机隐藏 GUI；push 后核实当前 SHA 实际结果，不由配置存在推断成功。

## 唯一下一任务

**T05 — 音律与 DSP 数据模型。**

1. 读取 AGENTS、README、完整主任务书、STATUS/HANDOFF、UI_GUIDE、TESTING、ENVIRONMENT 和 DECISIONS；核对实际 Git、分支/工作区/remote，安全 fetch，核实本轮提交与 CI。
2. 从项目外复跑 check-env/bootstrap/test/build/test-ui；先恢复 T04 基线，再开发。核对锁文件跟踪、缓存忽略/兼容链接，以及原任务书 D002 哈希。
3. 按 T05 建立与 Tauri/设备无关的 tuner-core，定义 Rust 音高帧/音律模型、MIDI/Hz/cents/A4 转换、目标映射及确定性正弦/谐波测试生成器；不实现 YIN/FFT 或 UI 桥接。
4. 覆盖 A4=440/442、C4、A0、正负偏差、边界/非法/非有限值，442 对 440 约 +7.85 cents。T04 TS 是展示契约，后续 T11 再建立 Rust/TS IPC 映射与契约测试。
5. UI 验证继续后台，不前置窗口或注入系统输入；每项结束更新交接、实际验收、显式 stage、commit/push、核实远端及 CI。

当前可以体验调音 UI Demo，仍不能用于真实乐器调音。
