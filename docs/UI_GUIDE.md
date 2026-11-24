# Lyreniq UI 设计规范

更新：2026-11-24（Asia/Shanghai）；T03。当前交付为工作区壳与外观选择，音频功能尚未开放。

## 视觉方向

采用石墨灰、暖白与低饱和薄荷绿，突出工作区层级和留白。侧栏、页标题、主工作台、状态栏保持稳定位置；不使用霓虹、装饰性渐变或自动播放动画。调音数据的最大字号与仪表将在 T04 建立，本轮空状态不显示音名、Hz、cents 或虚构波形。

原生窗口默认 1200×700，最小客户区 1024×700（逻辑像素）。在 1024×700 和 1440×900 检查两种主题与四个页面。侧栏通常 220px，大窗口 240px；顶部 58px，底部状态栏 34px。主体使用可伸缩列和独立滚动容器；这两种验收尺寸均无需滚动且控件不裁切。

## Token 与基础组件

统一样式位于 `apps/desktop/src/styles.css`，主题由 `.app-shell[data-theme]` 控制。组件使用语义变量，不另造页面专属色板。

| Token | 深色 | 浅色 | 用途 |
|---|---|---|---|
| --bg | #171a1b | #f5f6f3 | 工作区背景 |
| --sidebar | #121516 | #eef0ec | 导航底色 |
| --surface | #1c2021 | #fdfefb | 主工作台 |
| --text | #edf0ee | #202c27 | 标题与正文 |
| --muted | #a3aeaa | #55665d | 说明文字 |
| --subtle | #899590 | #5c6c62 | 次要说明 |
| --accent | #a0ddc8 | #23694e | 交互与图标 |
| --focus | #9de2c9 | #267054 | 键盘焦点 |

- 字体：系统 Segoe UI / Microsoft YaHei UI；不分发字体文件或下载远程字体。常规字号 14px、说明 12px、标题 30px；小字号只用于次要标签。未来测量数字使用 tabular-nums。
- 间距以 8px 为基准，使用 16/24/32px 等节奏；工作台圆角 12px，按钮 7px，细边框区分层级。
- `components/ui.tsx` 提供 Button（primary/secondary/quiet）和 Badge，保留原生 HTML 语义与 disabled 行为。复杂控件出现实际需求后再引入组件库。
- Lucide 图标统一线性样式，只导入使用的图标；装饰图标 aria-hidden。Lyreniq 线条标记是本项目原创 SVG，不取自参考产品品牌。

## 页面与状态

| 页面 | 当前行为 | 后续入口 |
|---|---|---|
| 实时调音 | 清楚显示未监听、音频输入尚未开放；开始监听禁用 | T04 展示组件；T12 真实输入 |
| 文件分析 | 文件分析空状态；导入禁用，不读取文件 | T17 文件页面 |
| 参考音 | 播放空状态；播放禁用，不输出声音 | T20 参考音 |
| 设置 | 两种主题即时选择；音频设置未开放 | T21 设置持久化 |

主题默认深色，在本次会话及页面切换间保留；关闭应用后不承诺恢复。底部状态栏仅报告真实 `runtime_label` 连接结果，支持重试、等待、失败后清除旧成功值。文案集中在 `src/content/zh-CN.ts`；当前语言为中文，英文切换尚未实现。

## 键盘与可访问性

- Tab 可到达跳转链接、四个导航、主题按钮及桌面重连。Enter/Space 使用原生按钮行为，不占用系统快捷键。
- 导航使用 nav 与 aria-current=page，切换后将页面内焦点送到 h1；跳转链接可直接聚焦 main。
- 外观选择使用 fieldset、可访问名称及 aria-pressed；选择状态同时有文字/边框/圆点，不只依赖颜色。
- 键盘焦点为 2px 实线、4px 外偏移；选中边框与焦点反馈可同时显示。forced-colors 下保留系统高亮。
- 普通、说明和次要文字 token 对背景/工作台/侧栏的 18 组组合均至少 4.5:1。首次检查发现浅色次要文字 4.45:1，已调深后复验，不放宽断言。
- 仅在未要求减弱动效时启用 120ms 状态过渡；没有自动动画。禁用控件保留可见说明，不以不可操作的装饰入口诱导点击。

## 开源参考与许可核实

以下为 2026-11-24 实际读取官方仓库、发布记录及许可的快照。星数只是规模信息，同时检查近期活动和非 archived 状态。

| 项目 | 实际版本与活跃度 | 许可和采用方式 |
|---|---|---|
| [shadcn/ui](https://github.com/shadcn-ui/ui) | [shadcn@4.23.0](https://github.com/shadcn-ui/ui/releases/tag/shadcn%404.23.0)，2026-10-27；128,922 stars，最近 push 11-23，未归档 | [MIT](https://github.com/shadcn-ui/ui/blob/main/LICENSE.md)；参考语义 token、组件状态及[深浅主题方式](https://ui.shadcn.com/docs/theming)，样式与组件自行实现，未运行其 CLI 或复制模板 |
| [Audacity](https://github.com/audacity/audacity) | [4.0.2](https://github.com/audacity/audacity/releases/tag/Audacity-4.0.2)，2026-11-02；19,162 stars，最近 push 11-24，未归档 | [整体 GPLv3，部分文件 GPLv2-or-later 等](https://github.com/audacity/audacity/blob/master/LICENSE.txt)；只研究工作区/状态信息层级，不复制代码、图标、截图或品牌 |
| [Lucide](https://github.com/lucide-icons/lucide) | 当日最新 [1.79.0](https://github.com/lucide-icons/lucide/releases/tag/1.79.0)，25,966 stars，11-24 有活动，未归档；实际依赖锁定 1.75.0（11-18 发布） | [ISC，Feather 衍生图标附 MIT](https://github.com/lucide-icons/lucide/blob/main/LICENSE)；本地 lucide-react 1.75.0，React 19 peer 相容，完整许可保存在 [第三方声明](licenses/lucide-react-1.75.0.txt) |

Lucide 选择较早已发布版本，移除了安装最新版本时 pnpm 自动添加的发布时间豁免；最终 workspace 没有 minimumReleaseAgeExclude。Audacity 的 GitHub 自动许可识别为 NOASSERTION，实际许可按 LICENSE.txt 内容判断。没有引入 Audacity 或 shadcn 的运行时依赖。Lyreniq 自身许可证仍未决定。

## 验证与证据

`scripts/test-ui.ps1` 通过隐藏的真实 Tauri/WebView2 窗口执行 16 组合、真实 Rust 重连、键盘、焦点与对比度检查；没有前置窗口或注入系统输入。测试专用的调整窗口尺寸权限只出现在 .local 临时配置，生产 capabilities 未扩权。

截图与完整结果见 [T03 证据](evidence/T03/result.json)。125%/150% 结果仅为 CDP 渲染器设备比例模拟，与真实原生尺寸矩阵分开记录；没有更改 Windows 显示设置，原生 DPI 切换、标题栏实体交互与跨屏行为仍待真机验收。

完整命令、结果和限制见 [TESTING](TESTING.md)；下一任务为 T04，不能把当前壳称为可用调音器。
