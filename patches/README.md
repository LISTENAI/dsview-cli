# DSView Local Patches

这个目录用于保存“我们项目依赖、但不直接提交到上游 `DSView` 子模块”的本地补丁。

原则：

- `DSView/` 继续作为上游依赖使用，不把本地项目需求直接混进子模块历史
- 任何必须依赖的上游改动，都尽量以 patch 形式保存在这里
- patch 由主仓库追踪，便于审查、迁移和在更新 submodule 后重新应用

## 目录结构

- `dsview/`：针对 `DSView` 子模块的 patch 文件

## 当前 Patch

### `dsview/0001-sdcard-spi-write-block-syntax-fix.patch`

作用：

- 修复 `DSView/libsigrokdecode4DSL/decoders/sdcard_spi/pd.py` 中的一个 Python 语法错误
- 具体是 `handle_cmd24()` 里的多余右括号，导致该 decoder 无法正常导入

适用场景：

- 当 `DSView` 子模块切换到一个不包含该修复的上游提交时
- 或者本地需要重新把子模块恢复为上游干净状态后，再重新应用此 patch

## 如何应用

使用项目内脚本：

```bash
./scripts/apply-dsview-patches.sh
```

脚本行为：

- 逐个应用 `patches/dsview/*.patch`
- 如果 patch 已经应用，会显示 `Already applied`
- 如果 patch 不能干净应用，会报错退出，避免静默失败

## 维护约定

新增 patch 时建议遵循：

1. 文件名带顺序号和简短用途说明
2. 在这个 README 里补一条说明：
   - 改了什么
   - 为什么需要
   - 何时应当重新应用
3. 尽量让每个 patch 只解决一个明确问题，避免把多类改动混在一起

