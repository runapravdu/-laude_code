# 用于安全研究的 Claude Code 源代码快照

English original: [README.md](README.md)

> 这个仓库镜像了一份**公开暴露的 Claude Code 源代码快照**，该快照于**2026 年 3 月 31 日**因 npm 发行包中的 source map 暴露而变得可被访问。维护此仓库的目的是用于**教育、防御性安全研究以及软件供应链分析**。

---

## 研究背景

这个仓库由一名**大学生**维护，其研究方向包括：

- 软件供应链暴露和构建产物泄露
- 安全软件工程实践
- 代理式开发工具架构
- 对真实世界 CLI 系统的防御性分析

此归档旨在支持：

- 教育学习
- 安全研究实践
- 架构审查
- 对打包和发布流程失误的讨论

它**不**声称拥有原始代码的所有权，也不应被解读为 Anthropic 的官方仓库。

---

## 公开快照如何变得可被访问

[Chaofan Shou (@Fried_rice)](https://x.com/Fried_rice) 公开指出，Claude Code 的源代码材料可以通过 npm 包中暴露的 `.map` 文件访问到：

> **“Claude code 源代码已通过其 npm registry 中的一个 map 文件泄露！”**
>
> — [@Fried_rice，2026 年 3 月 31 日](https://x.com/Fried_rice/status/2038894956459290963)

已发布的 source map 引用了托管在 Anthropic 的 R2 存储桶中的未混淆 TypeScript 源文件，这使得 `src/` 快照可以被公开下载。

---

## 仓库范围

Claude Code 是 Anthropic 的 CLI，用于从终端与 Claude 交互，以执行软件工程任务，例如编辑文件、运行命令、搜索代码库和协调工作流。

这个仓库包含一个被镜像的 `src/` 快照，用于研究和分析。

- **发现公开暴露时间**: 2026-03-31
- **语言**: TypeScript
- **运行时**: Bun
- **终端 UI**: React + [Ink](https://github.com/vadimdemedes/ink)
- **规模**: 约 1,900 个文件，512,000+ 行代码

---

## 目录结构

```text
src/
├── main.tsx                 # 入口编排（基于 Commander.js 的 CLI 路径）
├── commands.ts              # 命令注册表
├── tools.ts                 # 工具注册表
├── Tool.ts                  # 工具类型定义
├── QueryEngine.ts           # LLM 查询引擎
├── context.ts               # 系统/用户上下文收集
├── cost-tracker.ts          # Token 成本跟踪
│
├── commands/                # Slash 命令实现（约 50 个）
├── tools/                   # Agent 工具实现（约 40 个）
├── components/              # Ink UI 组件（约 140 个）
├── hooks/                   # React hooks
├── services/                # 外部服务集成
├── screens/                 # 全屏 UI（Doctor、REPL、Resume）
├── types/                   # TypeScript 类型定义
├── utils/                   # 工具函数
│
├── bridge/                  # IDE 与远程控制桥接
├── coordinator/             # 多 Agent 协调器
├── plugins/                 # 插件系统
├── skills/                  # Skill 系统
├── keybindings/             # 按键绑定配置
├── vim/                     # Vim 模式
├── voice/                   # 语音输入
├── remote/                  # 远程会话
├── server/                  # 服务端模式
├── memdir/                  # 持久化记忆目录
├── tasks/                   # 任务管理
├── state/                   # 状态管理
├── migrations/              # 配置迁移
├── schemas/                 # 配置 schema（Zod）
├── entrypoints/             # 初始化逻辑
├── ink/                     # Ink 渲染器包装层
├── buddy/                   # 伙伴精灵
├── native-ts/               # 原生 TypeScript 工具
├── outputStyles/            # 输出样式
├── query/                   # 查询流水线
└── upstreamproxy/           # 代理配置
```

---

## 架构摘要

### 1. 工具系统（`src/tools/`）

Claude Code 可调用的每个工具都被实现为一个自包含模块。每个工具都定义其输入 schema、权限模型和执行逻辑。

| 工具 | 描述 |
|---|---|
| `BashTool` | Shell 命令执行 |
| `FileReadTool` | 文件读取（图片、PDF、notebook） |
| `FileWriteTool` | 文件创建 / 覆写 |
| `FileEditTool` | 部分文件修改（字符串替换） |
| `GlobTool` | 文件模式匹配搜索 |
| `GrepTool` | 基于 ripgrep 的内容搜索 |
| `WebFetchTool` | 获取 URL 内容 |
| `WebSearchTool` | Web 搜索 |
| `AgentTool` | 子 Agent 派生 |
| `SkillTool` | Skill 执行 |
| `MCPTool` | MCP 服务器工具调用 |
| `LSPTool` | Language Server Protocol 集成 |
| `NotebookEditTool` | Jupyter notebook 编辑 |
| `TaskCreateTool` / `TaskUpdateTool` | 任务创建与管理 |
| `SendMessageTool` | Agent 间消息传递 |
| `TeamCreateTool` / `TeamDeleteTool` | 团队 Agent 管理 |
| `EnterPlanModeTool` / `ExitPlanModeTool` | Plan 模式切换 |
| `EnterWorktreeTool` / `ExitWorktreeTool` | Git worktree 隔离 |
| `ToolSearchTool` | 延迟工具发现 |
| `CronCreateTool` | 定时触发器创建 |
| `RemoteTriggerTool` | 远程触发器 |
| `SleepTool` | 主动模式等待 |
| `SyntheticOutputTool` | 结构化输出生成 |

### 2. 命令系统（`src/commands/`）

通过 `/` 前缀调用的面向用户的 slash 命令。

| 命令 | 描述 |
|---|---|
| `/commit` | 创建 git commit |
| `/review` | 代码审查 |
| `/compact` | 上下文压缩 |
| `/mcp` | MCP 服务器管理 |
| `/config` | 设置管理 |
| `/doctor` | 环境诊断 |
| `/login` / `/logout` | 身份验证 |
| `/memory` | 持久化记忆管理 |
| `/skills` | Skill 管理 |
| `/tasks` | 任务管理 |
| `/vim` | Vim 模式切换 |
| `/diff` | 查看变更 |
| `/cost` | 查看使用成本 |
| `/theme` | 更改主题 |
| `/context` | 上下文可视化 |
| `/pr_comments` | 查看 PR 评论 |
| `/resume` | 恢复上一会话 |
| `/share` | 分享会话 |
| `/desktop` | 桌面应用交接 |
| `/mobile` | 移动应用交接 |

### 3. 服务层（`src/services/`）

| 服务 | 描述 |
|---|---|
| `api/` | Anthropic API 客户端、文件 API、bootstrap |
| `mcp/` | Model Context Protocol 服务器连接与管理 |
| `oauth/` | OAuth 2.0 身份验证流程 |
| `lsp/` | Language Server Protocol 管理器 |
| `analytics/` | 基于 GrowthBook 的功能标志和分析 |
| `plugins/` | 插件加载器 |
| `compact/` | 对话上下文压缩 |
| `policyLimits/` | 组织策略限制 |
| `remoteManagedSettings/` | 远程托管设置 |
| `extractMemories/` | 自动记忆提取 |
| `tokenEstimation.ts` | Token 数量估算 |
| `teamMemorySync/` | 团队记忆同步 |

### 4. 桥接系统（`src/bridge/`）

一个双向通信层，将 IDE 扩展（VS Code、JetBrains）与 Claude Code CLI 连接起来。

- `bridgeMain.ts` — 桥接主循环
- `bridgeMessaging.ts` — 消息协议
- `bridgePermissionCallbacks.ts` — 权限回调
- `replBridge.ts` — REPL 会话桥接
- `jwtUtils.ts` — 基于 JWT 的身份验证
- `sessionRunner.ts` — 会话执行管理

### 5. 权限系统（`src/hooks/toolPermission/`）

在每次工具调用时检查权限。它要么提示用户进行批准/拒绝，要么根据配置的权限模式（`default`、`plan`、`bypassPermissions`、`auto` 等）自动处理。

### 6. 功能标志

通过 Bun 的 `bun:bundle` 功能标志进行死代码消除：

```typescript
import { feature } from 'bun:bundle'

// 未启用的代码会在构建时被完全剥离
const voiceCommand = feature('VOICE_MODE')
  ? require('./commands/voice/index.js').default
  : null
```

主要标志：`PROACTIVE`、`KAIROS`、`BRIDGE_MODE`、`DAEMON`、`VOICE_MODE`、`AGENT_TRIGGERS`、`MONITOR_TOOL`

---

## 关键文件详解

### `QueryEngine.ts`（约 4.6 万行）

LLM API 调用的核心引擎。处理流式响应、工具调用循环、思考模式、重试逻辑以及 token 计数。

### `Tool.ts`（约 2.9 万行）

定义所有工具的基础类型和接口，包括输入 schema、权限模型以及进度状态类型。

### `commands.ts`（约 2.5 万行）

管理所有 slash 命令的注册与执行。它使用条件导入来根据不同环境加载不同的命令集。

### `main.tsx`

基于 Commander.js 的 CLI 解析器和 React/Ink 渲染器初始化。在启动时，它会并行处理 MDM 设置、钥匙串预取和 GrowthBook 初始化，以加快启动速度。

---

## 技术栈

| 类别 | 技术 |
|---|---|
| 运行时 | [Bun](https://bun.sh) |
| 语言 | TypeScript（strict） |
| 终端 UI | [React](https://react.dev) + [Ink](https://github.com/vadimdemedes/ink) |
| CLI 解析 | [Commander.js](https://github.com/tj/commander.js)（extra-typings） |
| Schema 验证 | [Zod v4](https://zod.dev) |
| 代码搜索 | [ripgrep](https://github.com/BurntSushi/ripgrep) |
| 协议 | [MCP SDK](https://modelcontextprotocol.io)、LSP |
| API | [Anthropic SDK](https://docs.anthropic.com) |
| 遥测 | OpenTelemetry + gRPC |
| 功能标志 | GrowthBook |
| 身份验证 | OAuth 2.0、JWT、macOS Keychain |

---

## 值得注意的设计模式

### 并行预取

通过在重型模块求值开始之前并行预取 MDM 设置、钥匙串读取和 API 预连接来优化启动时间。

```typescript
// main.tsx — 在其他导入之前作为副作用触发
startMdmRawRead()
startKeychainPrefetch()
```

### 惰性加载

重型模块（OpenTelemetry、gRPC、分析，以及某些受功能标志控制的子系统）会通过动态 `import()` 推迟到真正需要时再加载。

### Agent 集群

子 Agent 通过 `AgentTool` 派生，`coordinator/` 负责多 Agent 编排。`TeamCreateTool` 启用团队级并行工作。

### Skill 系统

定义在 `skills/` 中的可复用工作流通过 `SkillTool` 执行。用户可以添加自定义技能。

### 插件架构

内置和第三方插件通过 `plugins/` 子系统加载。

---

## 研究 / 所有权免责声明

- 这个仓库是一个由大学生维护的**教育性与防御性安全研究归档**。
- 它的存在是为了研究源代码暴露、打包失误，以及现代代理式 CLI 系统的架构。
- 原始 Claude Code 源代码仍归 **Anthropic** 所有。
- 这个仓库**不隶属于 Anthropic、未获 Anthropic 背书，也不是由 Anthropic 维护**。
