# Lyreniq 交接记录

## 本轮起点与范围

- 日期：2026-11-24（Asia/Shanghai）
- 任务：仅 T00 — 仓库盘点与开发规则
- 路径：`D:\2-workspace\Lyreniq`；分支：main
- 起点 commit：无。首次盘点只有根目录任务书（91,308 字节），无 Git、源码、历史交接或测试，所以上轮回归不适用。
- 已完整读取任务书 1,213 行及第 6 节协议；未安装工具链，未创建应用工程、脚本或 CI，未进入 T01。

## 本轮文件

- 新建：AGENTS.md、README.md、.gitignore、.gitattributes、.editorconfig。
- 新建：docs/STATUS.md、docs/HANDOFF.md、docs/DECISIONS.md、docs/TESTING.md。
- 原样复制任务书到 docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md；保留根目录原件。
- 原件 SHA-256：`C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF`。

## 实际命令与结果

| 命令/检查 | 退出码或结果 | 说明 |
|---|---|---|
| `Get-Location`、`Get-ChildItem -Force`、`rg --files` | 0 | 确认项目路径；起初仅有任务书，无适用的已有 AGENTS |
| `Get-Content -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Encoding UTF8` 分段读取 | 0 | 完整读取全部 1,213 行，避免输出截断 |
| 初始化前 `git status --short --branch`、`git remote -v`、`git log -5 --format="%h %s"` | 命令批次返回 1；未单独采集各退出码 | 三条均报告不是 Git 仓库，属于初始事实 |
| `git --version` | 0 | git version 2.55.0.windows.5 |
| `git config --get user.name`、`git config --get user.email` | 各 0 | 已有提交身份可用；未修改全局配置 |
| `gh auth status` | 1 | gh 命令不可用；无法通过 CLI 核实认证；本轮不安装 |
| GitHub 连接器 `search_installed_repositories_v2` | 请求成功，均为空 | 分别查询 Lyreniq、Universal Tuner；未据此推断账户没有其他仓库 |
| `git init -b main` | 0 | 在本目录建立 Git |
| 初始化后 `git status --short --branch` | 0 | main，无提交，仅原任务书未跟踪 |
| 初始化后 `git log -5 --oneline` | 报告 main 尚无提交；未单独采集退出码 | 符合首次初始化状态 |
| `git diff`、`git diff --cached`、`git remote -v` | 各 0 | 无已跟踪差异、无 staged 文件、无 remote |
| `Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256` | 成功 | 哈希见上 |
| PowerShell 文档、链接、任务 ID、范围及哈希断言（见 TESTING） | 0 | 11 个文件、8 个本地链接；原件和副本哈希一致；无应用清单/目录 |
| `git check-ignore --no-index -- ...`（TESTING 内列出完整参数） | 忽略组 0，例外组 1（预期） | 15/15 项忽略；6 项 fixtures/锁文件/示例环境配置未忽略 |
| `git diff --check`、`git diff --cached --check` | 各 0 | 无空白错误；原任务书 Markdown 硬换行有专用属性 |
| `git diff --cached --stat` 及基础文档 staged diff 审查 | 0 | 仅本轮 11 个文件；无源码、缓存、音频或密钥 |
| `git rev-parse :UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md`、`git rev-parse :docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md`、`git hash-object --no-filters UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md` | 均 0，结果相同 | 两份 indexed blob 与未转换原件一致：61c31b195ee92a3f805d6982a336da146b84ab73 |
| `rg` 常见私钥/GitHub token/AWS access key 模式扫描 | 1（无匹配，预期） | PowerShell 断言退出 0；已人工审阅 staged 基础文档 |

实际暂存命令（首次沙箱内返回 128：不能写 .git/index.lock；权限流程后相同命令返回 0）：

```powershell
git add -- AGENTS.md README.md .gitignore .gitattributes .editorconfig UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md docs/STATUS.md docs/HANDOFF.md docs/DECISIONS.md docs/TESTING.md
```

暂存曾提示三个配置文件会随 core.autocrlf 转为 CRLF，已将 .gitattributes 默认规则明确为 `text=auto eol=lf`，在最终暂存时复核。此变更只影响项目规则，未修改全局 Git 配置。

首次 shell 启动曾显示用户 PowerShell profile 的 conda 启动错误；后续使用无 profile 的 PowerShell 命令完成 T00，未修改系统/profile。开发工具链是否受其影响留到 T01 检查。

本地文档验收通过，详见 TESTING。应用测试、构建、GUI、麦克风、解码、离线运行和安装均未执行，尚无应用，不作通过声明。

## Git 与待办

- 本地提交：验收已通过，提交命令为 `git commit -m "docs: initialize project plan and handoff rules"`。本记录按协议在提交前写入；提交最终结果以 Git 日志和本轮最终报告为准。
- 提交自身 hash 不写入本提交；完成后由最终报告提供，下一轮通过 `git log -1 --format="%H %s"` 读取并可回填。
- Push：未执行，remote 未配置。取得已授权仓库后先检查远端历史，再正常推送；禁止强推或覆盖现有历史。
- 若提供的是空仓库，复核后可运行 `git remote add origin <已授权仓库URL>`、`git push -u origin main`，再用 `git ls-remote --heads origin main` 与本地 `git rev-parse HEAD` 对比。
- 若远端已有提交或受分支保护约束，先 fetch 并检查历史/仓库规则，保留本地初始提交，不直接覆盖。
- 尚需确定仓库 URL（或所有者、名称、可见性）、项目许可证；不擅自创建公开仓库。

## 下一轮强制恢复

1. 阅读 AGENTS、README、docs 下任务书、STATUS、HANDOFF、DECISIONS、TESTING。
2. 运行 `git rev-parse --show-toplevel`、`git status --short --branch`、`git branch --show-current`、`git log -5 --oneline`、`git diff`、`git diff --cached`、`git remote -v`。确认本轮初始提交及实际文件，保留用户修改。
3. 重跑 `git diff --check`、`git show --format= --check HEAD`、TESTING 中的文档链接与任务一致性检查；确认原始任务书哈希。若主文档未来经过批准修订，按 DECISIONS 复核，不要求永远与冻结原件相等。
4. 若已有授权远端，安全 fetch 并检查 ahead/behind；未推送的 T00 必须继续如实标注。没有应用时，上一轮构建和 GUI smoke test 不适用。
5. 上述失败先修复；通过后仅执行 **T01 — Windows 工具链与最小窗口**，原因是工程尚不存在。不要提前执行 T02 或产品功能。

唯一下一任务 ID：**T01**。GitHub 配置属于 T00 的交付待办，不是另一个产品开发任务。
