# Pastel 项目指南

Pastel 是 Design System 语言——用代码定义设计规范、画设计、校验一致性、生成前端代码。纯 Rust 实现，Skia 渲染。

## 构建与测试

```bash
cargo build                  # 构建全部 5 个 crate
cargo test --workspace       # 运行全部 143 个测试

# CLI 命令
pastel check <file>                           # 验证语法语义
pastel plan <file>                            # 显示节点树
pastel build <file> -o x.png                  # 渲染 PNG/SVG/PDF
pastel fmt <file>                             # 格式化源文件
pastel inspect <file> --json                  # 输出 IR JSON
pastel lint <file>                            # 检查设计是否符合 token 规范
pastel gen <file> --format tokens -o dir/     # 生成 CSS tokens + JSON
pastel gen <file> --format html -o dir/       # 生成 HTML
pastel gen <file> --format react -o dir/      # 生成 React 组件
```

## 5 个 Crate

| Crate | 职责 |
|-------|------|
| `pastel-lang` | 编译前端：词法/语法/语义分析、IR、格式化 |
| `pastel-render` | Skia 渲染：布局 + 绘制 + PNG/SVG/PDF 导出 |
| `pastel-codegen` | 代码生成：HTML/React/CSS tokens |
| `pastel-lint` | 设计规范校验：检查设计值是否匹配 token |
| `pastel-cli` | CLI 入口：8 个命令 |

## DSL 能力

**节点：** frame, text, image, shape (rect/ellipse/line/path)
**布局：** flex (horizontal/vertical), grid, absolute positioning
**填充：** 纯色, 透明, 线性渐变, 径向渐变
**效果：** 阴影/内阴影, 模糊/背景模糊, 圆角, 透明度, 旋转, 混合模式, 虚线描边
**文字：** 字号/字重/字体/颜色/对齐/行高/字间距/下划线/删除线/大小写/自动换行
**系统：** token 定义, component (参数+默认值), include (命名空间+冲突检测), 多页, 格式化
**绘图：** SVG path data (贝塞尔曲线/多边形/自由路径)
**导出：** PNG, SVG, PDF, HTML, React, CSS tokens, JSON IR

## 文件规范

- 单文件不超过 200 行
- 测试放 `tests/` 目录
- 11 个示例在 `examples/`

## 回复语言

始终使用中文(简体)回复。
