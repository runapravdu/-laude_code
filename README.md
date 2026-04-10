# Claude Code 源码还原

> 从 source map 还原并可运行的 Claude Code CLI 完整 TypeScript 源码

![预览](preview.png)

> [!WARNING]
> 本仓库为**非官方**版本，基于公开 npm 发布包的 source map 还原，**仅供研究学习使用**。
> 不代表 Anthropic 官方原始内部开发仓库结构。
> 部分模块因 source map 无法完整还原，已用兼容 shim 或降级实现替代，行为可能与原版有所不同。

## 当前状态

- 源码树可还原并可在本地开发流程中运行
- `bun install` 可成功安装依赖
- `bun run version` 可正常输出版本号
- `bun run dev` 可启动还原后的 CLI 入口，作为交互式进程运行

## 基本信息

| 项目 | 说明 |
|------|------|
| 来源 | [@anthropic-ai/claude-code](https://www.npmjs.com/package/@anthropic-ai/claude-code) npm 包 |
| 源文件数 | 1,987 个 TypeScript/TSX 文件 |
| 运行时要求 | Bun ≥ 1.3.5、Node.js ≥ 24 |

## 快速开始

```bash
# 安装依赖
bun install

# 启动还原后的 CLI
bun run dev

# 输出版本号
bun run version
```

## 目录结构

```
├── src/                    # 还原的核心源码
│   ├── main.tsx            # CLI 主入口
│   ├── dev-entry.ts        # 开发入口
│   ├── commands.ts         # 命令注册
│   ├── commands/           # 斜杠命令实现（100+）
│   ├── tools/              # 工具实现（50+）
│   ├── components/         # 终端 UI 组件（React + Ink）
│   ├── hooks/              # 自定义 React Hooks
│   ├── services/           # API、MCP、分析等服务
│   ├── utils/              # 工具函数
│   ├── ink/                # 自定义 Ink 终端渲染器
│   ├── vim/                # Vim 模式引擎
│   ├── bridge/             # 远程桥接/控制
│   ├── coordinator/        # 多 Agent 协调器
│   ├── assistant/          # KAIROS 助手模式
│   ├── buddy/              # 伴侣精灵系统
│   ├── voice/              # 语音交互
│   ├── skills/             # 技能系统
│   ├── plugins/            # 插件系统
│   ├── remote/             # 远程会话管理
│   ├── server/             # IDE 集成直连服务器
│   └── keybindings/        # 快捷键绑定
├── shims/                  # 兼容性 shim 包（不可还原的原生模块替代）
├── vendor/                 # 原生绑定源码
├── package.json            # 项目配置
├── tsconfig.json           # TypeScript 配置
└── bun.lock                # 依赖锁文件
```

## 架构概览

### 启动流程

```
entrypoints/cli.tsx  →  main.tsx  →  REPL 渲染
     ↓                    ↓
  快速路径            完整初始化
(version/daemon)    (auth/MCP/settings/Commander.js)
```

### 工具系统（`src/tools/`，50+ 个工具）

| 类别 | 工具 |
|------|------|
| 文件操作 | `FileReadTool`、`FileWriteTool`、`FileEditTool`、`GlobTool`、`GrepTool` |
| 执行环境 | `BashTool`、`REPLTool`、`PowerShellTool`、`NotebookEditTool` |
| 网络 | `WebFetchTool`、`WebSearchTool`、`WebBrowserTool` |
| AI 协作 | `AgentTool`、`SendMessageTool`、`TeamCreateTool`、`TeamDeleteTool` |
| 任务管理 | `TaskCreateTool`、`TaskGetTool`、`TaskListTool`、`TaskUpdateTool`、`TaskStopTool` |
| MCP | `MCPTool`、`McpAuthTool`、`ListMcpResourcesTool`、`ReadMcpResourceTool` |
| 工作流 | `EnterPlanModeTool`、`ExitPlanModeTool`、`EnterWorktreeTool`、`ExitWorktreeTool` |
| 其他 | `SkillTool`、`ConfigTool`、`ScheduleCronTool`、`MonitorTool`、`WorkflowTool` |

### 服务层（`src/services/`）

- **`api/`** — Anthropic API 客户端、重试、速率限制、用量追踪
- **`mcp/`** — MCP 客户端/服务端、OAuth、通道管理
- **`compact/`** — 会话自动压缩策略
- **`analytics/`** — GrowthBook 特性开关、Datadog、事件日志

### UI 层

- **`ink/`** — 自定义 Ink 分支，含布局、焦点管理、ANSI 渲染、虚拟滚动
- **`components/`**（148 文件）— 终端 UI：消息、输入、diff 视图、权限对话框、状态栏
- **`hooks/`**（87 文件）— React Hooks：工具、语音、IDE、vim、会话、任务
- **`vim/`** — 完整 vim 键绑定引擎

## 还原说明

Source map 无法完整还原原始仓库，以下内容可能缺失或降级：

- 纯类型文件（type-only files）
- 构建时生成的文件
- 私有包装器和原生绑定
- 动态导入和资源文件

`shims/` 目录包含了对不可还原模块的兼容性替代实现。

## 声明

- 源码版权归 [Anthropic](https://www.anthropic.com) 所有
- 本仓库仅用于技术研究与学习，请勿用于商业用途
- 如有侵权，请联系删除
