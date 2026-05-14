# Excel 拆分工具

一个基于 Rust + egui 的 Excel 文件拆分工具，支持按指定列将 Excel 文件拆分为多个独立文件。

## 功能特性

- 拖拽或选择 Excel 文件
- 自动识别 Sheet 和列名
- 按指定列拆分数据
- 支持公共 Sheet（所有输出文件包含）
- 实时进度显示
- 中文界面支持

## 编译

### 本地编译

```bash
# 克隆项目
git clone <repository-url>
cd excel-splitter

# 编译发布版本
cargo build --release

# 运行
cargo run
```

### GitHub Actions 自动编译

项目已配置 GitHub Actions，每次推送标签或手动触发时，会自动编译 Windows 可执行文件并发布到 Releases。

## 使用方法

1. 选择或拖拽 Excel 文件到程序窗口
2. 点击「加载 Sheet 列表」
3. 勾选需要拆分的 Sheet，选择拆分列
4. 配置输出选项
5. 点击「开始拆分」

## 许可证

MIT
