# Pastel 项目指南

Pastel 是一门编译型设计 DSL——"Design as Code"。AI 直接编写 `.pastel` 文本文件，Rust 编译器验证语法语义，Skia 渲染引擎输出 PNG。全 Rust 实现，零外部运行时依赖。

## 构建与测试

```bash
cargo build                  # 构建全部 crate
cargo test --workspace       # 运行全部测试
cargo run --bin pastel -- check <file>          # 验证语法语义
cargo run --bin pastel -- plan <file>           # 显示节点树
cargo run --bin pastel -- build <file> -o x.png # 编译+渲染 PNG
cargo run --bin pastel -- inspect <file> --json # 输出 IR JSON
cargo run --bin pastel -- fmt <file>            # 格式化源文件
```

## 架构概览

| Crate | 路径 | 职责 |
|-------|------|------|
| `pastel-lang` | `crates/pastel-lang/` | 编译前端：词法/语法/语义分析、IR 定义 |
| `pastel-render` | `crates/pastel-render/` | Skia 渲染引擎：布局计算 + 绘制 + PNG 导出 |
| `pastel-cli` | `crates/pastel-cli/` | CLI：build / check / plan / fmt / inspect / serve |

### 关键模块

- `pastel-lang/src/lexer/` — 词法分析
- `pastel-lang/src/parser/` — 语法分析 (mod.rs 顶层, frame.rs 节点, expr.rs 表达式)
- `pastel-lang/src/semantic/` — 语义分析 (resolve.rs 变量, builder.rs IR构建, expand.rs 组件展开)
- `pastel-lang/src/ir/` — IR 类型 (node.rs 节点, style.rs 样式枚举)
- `pastel-render/src/layout.rs` — Flexbox 子集布局引擎
- `pastel-render/src/painter.rs` — Skia 绘制 (填充/描边/阴影/圆角/文字)

## 文件规范

- 单文件不超过 **200 行**，超出按职责拆分
- 测试放 `tests/` 目录，不与实现混合
- 测试夹具放 `fixtures/` 目录

## 语言设计要点

- **非图灵完备**：无循环、无条件、无递归
- `component` 是编译期宏展开，IR 中无组件引用
- `.pastel` 是唯一源格式
- 变量在编译期全部解析完成

## 回复语言

始终使用中文(简体)回复。
