.PHONY: build test lint fmt check install clean audit release tag ci help

# ────────────────────────────── 开发 ──────────────────────────────

build:               ## 构建全部 crate (debug)
	cargo build --workspace

release:             ## 构建 release 二进制
	cargo build --release

test:                ## 运行全部测试
	cargo test --workspace

lint:                ## Clippy 静态分析
	cargo clippy --workspace -- -D warnings

fmt:                 ## 格式化代码
	cargo fmt --all

fmt-check:           ## 检查格式（不修改）
	cargo fmt --all -- --check

check:               ## 快速编译检查
	cargo check --workspace

audit:               ## 依赖安全审计 (需要 cargo-audit)
	cargo audit

# ────────────────────────────── CI 复合 ──────────────────────────────

ci: check fmt-check lint test   ## 本地跑完整 CI 流水线

# ────────────────────────────── 发布 ──────────────────────────────

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

tag:                 ## 基于 Cargo.toml 版本打 tag 并推送
	@echo "Tagging v$(VERSION)..."
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	git push origin "v$(VERSION)"

# ────────────────────────────── 安装 ──────────────────────────────

install: release     ## 安装到 ~/.local/bin
	@mkdir -p $(HOME)/.local/bin
	cp target/release/pastel $(HOME)/.local/bin/

clean:               ## 清理构建产物
	cargo clean

# ────────────────────────────── 帮助 ──────────────────────────────

help:                ## 显示可用目标
	@grep -E '^[a-zA-Z_-]+:.*## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'
