# Lyreniq 验证记录

## T00 验收范围（2026-11-24）

本轮只有文档和 Git 初始化。原目录没有上一轮成果、测试或应用；上一轮应用回归不适用。任务书中的算法、UI、设备与发布矩阵是未来目标，本轮不把它们记作通过。

| 检查 | 当前结果 | 证据 |
|---|---|---|
| 原目录、文件与 Git 盘点 | 通过 | 原目录只有任务书；Git 初始不存在；见 HANDOFF |
| 完整阅读主任务书 | 完成 | 按行段读取全部 1,213 行 |
| Git 初始化 | 通过 | `git init -b main` 返回 0 |
| 原件保留与 docs 副本哈希 | 通过 | SHA-256 均与 DECISIONS D002 基准相同 |
| 基础文档、链接、任务 ID 一致性 | 通过；PowerShell 退出 0 | 11 个文件、8 个本地链接；当前 T00，唯一下一任务 T01；没有提前创建应用工程 |
| Git 空白及 staged 内容审查 | 通过；退出 0 | `git diff --check`、`git diff --cached --check`；仅 11 个本轮文件 |
| 忽略缓存、音频、用户数据和密钥 | 通过 | 15/15 忽略；6 项 fixtures/锁文件/环境示例未忽略；分别退出 0/1（均符合预期） |
| Git 暂存区任务书原始字节 | 通过 | 两个 indexed blob 与 `git hash-object --no-filters` 相同；见 HANDOFF |
| 常见凭据模式扫描与文本审查 | 通过 | 私钥/GitHub token/AWS access key 模式无匹配（rg 退出 1，预期）；人工检查 staged 文档 |
| GitHub remote / gh 认证 | 待配置 / CLI 不可用 | 无 remote；`gh auth status` 返回 1，未安装 gh |
| 应用测试、构建、GUI、设备、安装 | 未执行；不适用 | 没有应用、清单、测试或脚本 |

## 复验入口

从项目根目录运行；PowerShell 使用 `-NoProfile` 可避开本机已有 profile 的 conda 启动错误。

```powershell
git rev-parse --show-toplevel
git status --short --branch
git branch --show-current
git log -5 --oneline
git diff
git diff --cached
git remote -v
git diff --check
git show --format= --check HEAD
Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256
Get-FileHash -LiteralPath docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256
```

检查 AGENTS、README、STATUS、HANDOFF、DECISIONS、TESTING 的本地 Markdown 链接均指向现有文件；核实软件名 Lyreniq、T00 结果和唯一下一任务 T01，并与实际文件和 Git 日志对照。T00 两份任务书哈希相同；未来主文档获批修订后以决策记录为准。

本轮文档内联断言，可在 T01 开始实施前复跑（不安装工具、不创建文件）：

```powershell
$ErrorActionPreference = 'Stop'
$taskExpected = @('AGENTS.md','README.md','.gitignore','.gitattributes','.editorconfig','UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md','docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md','docs/STATUS.md','docs/HANDOFF.md','docs/DECISIONS.md','docs/TESTING.md')
foreach ($taskFile in $taskExpected) {
    if (-not (Test-Path -LiteralPath $taskFile -PathType Leaf)) { throw "Missing file: $taskFile" }
}
$taskFiles = @(& rg --files --hidden -g '!.git')
if ($LASTEXITCODE -ne 0 -or $taskFiles.Count -ne $taskExpected.Count) { throw 'Unexpected project file inventory' }
$taskLinks = 0
foreach ($taskFile in @('AGENTS.md','README.md','docs/STATUS.md','docs/HANDOFF.md','docs/DECISIONS.md','docs/TESTING.md')) {
    $taskText = Get-Content -LiteralPath $taskFile -Raw -Encoding UTF8
    foreach ($taskMatch in [regex]::Matches($taskText,'\[[^\]]+\]\(([^)]+)\)')) {
        $taskLink = $taskMatch.Groups[1].Value
        if ($taskLink -match '^(https?://|#)') { continue }
        $taskLinkPath = Join-Path (Split-Path -Parent (Join-Path (Get-Location) $taskFile)) $taskLink
        if (-not (Test-Path -LiteralPath $taskLinkPath -PathType Leaf)) { throw "Broken link in ${taskFile}: $taskLink" }
        $taskLinks++
    }
}
foreach ($taskFile in @('README.md','docs/STATUS.md','docs/HANDOFF.md')) {
    $taskText = Get-Content -LiteralPath $taskFile -Raw -Encoding UTF8
    foreach ($taskToken in @('Lyreniq','T00','T01')) {
        if (-not $taskText.Contains($taskToken)) { throw "Missing ${taskToken}: $taskFile" }
    }
}
foreach ($taskPath in @('Cargo.toml','package.json','apps','crates','scripts','.github')) {
    if (Test-Path -LiteralPath $taskPath) { throw "T00 scope exceeded: $taskPath" }
}
$taskOriginalHash = (Get-FileHash -LiteralPath UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256).Hash
$taskDocHash = (Get-FileHash -LiteralPath docs/UNIVERSAL_TUNER_DEVELOPMENT_PLAN.md -Algorithm SHA256).Hash
if ($taskOriginalHash -ne 'C26E8A2BC65E5C929D0E519878867CA7A51C5282E0346156A3164A38009372EF' -or $taskDocHash -ne $taskOriginalHash) { throw 'Plan bytes changed' }
Write-Output "PASS: $($taskFiles.Count) project files, $taskLinks local links, task/name consistency, T00 scope, both plan SHA-256 values"
```

该断言专用于 T00 基线。T01 开始新增工程后，文件数量/范围断言和副本哈希条件需按实际任务调整，不能误报后续合法变更失败。

忽略规则可用以下无须创建文件的命令复验：

```powershell
git check-ignore --no-index -- target/debug/probe.obj node_modules/probe.js apps/desktop/dist/probe.js .local/cache/probe .venv/probe .env .env.production debug.log recordings/probe.wav probe.wav user-data/settings.json settings.json installer.msi signing.key .vscode/settings.json
git check-ignore --no-index -- .env.example Cargo.lock pnpm-lock.yaml tests/fixtures/tone.wav tests/fixtures/tone.mp3 tests/fixtures/tone.flac
```

第一条期望逐个输出全部路径且退出码为 0；第二条期望无输出且退出码为 1（均未被忽略，属于预期结果）。正式夹具仅允许小型合成或明确授权音频，不能因存在例外就提交个人录音。

最终提交结果以 `git log` 和交付报告核实；GitHub push 状态独立记录，不能由本地 Git 成功推断。常见凭据模式扫描及人工 staged diff 审查用于本轮文本核验，不代表未来文件免于审查。
