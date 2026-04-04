# Pastel 项目指南

Pastel 是一门编译型设计 DSL——"Design as Code"。AI 直接编写 `.pastel` 文本文件，编译器验证语法语义，渲染器输出像素或前端代码。整个流程：`.pastel` 源码 → 词法分析 → 语法分析 → 语义分析 → IR (JSON) → Canvas 2D / JSX / CSS。

## 构建与测试

```bash
# Rust 编译器
cargo build                  # 构建全部 crate
cargo test                   # 运行全部 Rust 测试

# TypeScript 包
pnpm install                 # 安装依赖
pnpm -r build                # 构建全部 packages
pnpm -r test                 # 运行全部 TS 测试

# CLI 常用命令
cargo run --bin pastel -- check <file>         # 验证语法语义
cargo run --bin pastel -- plan <file>          # 显示节点树
cargo run --bin pastel -- inspect <file> --json # 输出 IR JSON
cargo run --bin pastel -- fmt <file>           # 格式化源文件
```

## 架构概览

### Rust Crate

| Crate | 路径 | 职责 |
|-------|------|------|
| `pastel-lang` | `crates/pastel-lang/` | 编译前端：词法分析、语法分析、语义分析、IR 类型定义 |
| `pastel-cli` | `crates/pastel-cli/` | CLI 入口：build / check / plan / fmt / inspect / serve |

### TypeScript Package

| 包 | 路径 | 职责 |
|----|------|------|
| `@pastel/renderer` | `packages/renderer/` | Canvas 2D 渲染引擎 |
| `@pastel/codegen` | `packages/codegen/` | IR → JSX / CSS / Design Tokens |
| `@pastel/preview` | `packages/preview/` | 实时预览服务 (文件监听 + WebSocket) |
| `@pastel/web` | `packages/web/` | Web 编辑器 (React 代码+预览分屏) |

### 关键模块

- `pastel-lang/src/lexer/` — 词法分析 (mod.rs 核心调度, scan.rs 扫描规则)
- `pastel-lang/src/parser/` — 语法分析 (mod.rs 顶层, frame.rs 节点, expr.rs 表达式)
- `pastel-lang/src/semantic/` — 语义分析 (mod.rs 入口, resolve.rs 变量解析, builder.rs IR构建, expand.rs 组件展开)
- `pastel-lang/src/ir/` — IR 类型 (mod.rs 文档, node.rs 节点, style.rs 样式)

## 文件规范

- 单文件不超过 **200 行**，超出则按职责拆分
- 测试文件放在独立的 `tests/` 目录，不与实现混合
- Rust 测试：`crates/*/tests/*.rs`
- TypeScript 测试：`packages/*/tests/*.test.ts`
- 测试夹具：`fixtures/` 目录

## 语言设计要点

- **非图灵完备**：无循环、无条件、无递归
- `component` 是编译期宏展开，IR 中无组件引用
- `.pastel` 是唯一源格式，IR JSON 是编译产物
- 变量在编译期全部解析完成

## 回复语言

始终使用中文(简体)回复。
