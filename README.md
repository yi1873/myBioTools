# myBioTools

一个用 Rust 实现的生物信息学工具集合，专注于高效和易用性。

> 个人常用小工具集合，不定时更新

## 特性

- **快速的 FASTA/表格处理** – 利用 Rust 的性能和内存安全
- **模块化的子命令架构** – 易于扩展新工具
- **Python 互操作性** – 通过 `pyo3` 调用 Python 库（在需要时）

## 工具列表

| 命令 | 描述 |
|------|------|
| `select` | 从 `FASTA`或`表格`中提取目标行/序列，多子工具集合，`-h`查看 |
| `split-fasta` | 将多序列 FASTA 文件拆分为单个文件 |
| `fasta-stats` | 计算 FASTA 统计信息（GC%、长度等） |
| `rev-comp` | 生成 DNA 序列的反向互补序列 |
| `hairpin` | 计算发夹形成的自由能 ΔG (kcal/mol) 【引物使用】 |
| `heterodimer` | 计算异源二聚体形成的自由能 ΔG (kcal/mol) 【引物使用】|
| `homodimer` | 计算同源二聚体形成的自由能 ΔG (kcal/mol)【引物使用】 |
| `tm` | 计算 DNA 序列的熔解温度 (Tm) 【引物使用】|

## 安装

### 从源码编译

```bash
git clone https://github.com/yi1873/myBioTools.git
cd myBioTools
cargo build --release
```

## 使用

```bash
./target/release/myBioTools  [子命令] [选项]
```

### `select` – 提取或过滤序列

三种操作模式：

- **`line`** – 从表格文件中提取与基因 ID 列表匹配的行，可`-c`按指定列进行筛选，默认为1;
- **`fa`** – 从 FASTA 文件中提取与基因 ID 列表匹配的序列;
- **`onlyfa`** – 按最小长度过滤 FASTA 序列（无需 ID 列表）;

**示例：**

```bash
# 从表格中提取行
myBioTools select --cls line -l geneid.txt -s table.txt  -o output.txt

# 从 FASTA 中提取序列
myBioTools select --cls fa -l geneid.txt -s sequences.fasta -o selected.fasta

# 按长度过滤 FASTA
myBioTools select --cls onlyfa -s sequences.fasta --len 100 -o filtered.fasta
```

### `split-fasta` – 拆分多序列 FASTA 文件

将多序列 FASTA 文件拆分为单个文件，每个序列一个文件。  
输出目录默认为 `./split_fasta.subdir`，可通过 `--output-dir` 更改。  
文件扩展名可通过 `--extension` 自定义。

**示例：**

```bash
myBioTools split-fasta -i input.fasta -o ./my_split_dir --ext .fa
```

### `tm` – 计算熔解温度

使用 `primer3-py`（通过 `pyo3`）计算 Tm，使用默认的盐浓度和引物浓度。  
输出包含单位并保留两位小数。

**示例：**

```bash
myBioTools tm -s "ATCGATCGATCG"
# 输出：Tm = 52.34 °C
```

### `fasta-stats` – FASTA 统计信息

计算 FASTA 文件中每个序列的基本统计信息：长度、GC 含量、N 计数等。

**示例：**

```bash
myBioTools fasta-stats -i sequences.fasta -o stats.csv
```

### `hairpin`、`heterodimer`、`homodimer` – 自由能计算

这三个命令分别计算二级结构形成的自由能变化 (ΔG)。  
它们是从现有的 Rust 子项目集成而来，输出格式一致。

**示例：**

```bash
myBioTools hairpin -s "CCCCGGGG"
myBioTools heterodimer -s1 "AAAAAA" -s2 "TTTTTT"
myBioTools homodimer -s "CACACACACA"
```

### `rev-comp` – 反向互补序列

生成 DNA 序列的反向互补序列。  
支持直接序列输入或 FASTA 文件输入。

**示例：**

```bash
# 直接序列输入
myBioTools rev-comp -i "ATCG"
# 输出：CGAT

# FASTA 文件输入
myBioTools rev-comp --input-file sequences.fasta --output-file rev.fasta
```

## 项目结构

```
src/
├── main.rs          # CLI 入口点，命令路由
├── lib.rs           # 公共模块和通用工具函数
├── select.rs        # select 子命令
├── split_fasta.rs   # split‑fasta 子命令
├── tm.rs            # tm 子命令（pyo3 桥接）
├── fasta_stats.rs   # fasta‑stats 子命令
├── hairpin.rs       # hairpin 子命令
├── heterodimer.rs   # heterodimer 子命令
├── homodimer.rs     # homodimer 子命令
└── rev_comp.rs      # rev‑comp 子命令
```

## 依赖项

- [`clap`](https://crates.io/crates/clap) – 命令行参数解析
- [`anyhow`](https://crates.io/crates/anyhow) / [`thiserror`](https://crates.io/crates/thiserror) – 错误处理
- [`needletail`](https://crates.io/crates/needletail) – 快速 FASTA 解析
- [`pyo3`](https://crates.io/crates/pyo3) – Python 绑定（用于 Tm 计算）
- [`csv`](https://crates.io/crates/csv) – CSV 读写
- [`serde`](https://crates.io/crates/serde) – 序列化

## 开发

1. 克隆仓库。
2. 确保已安装 Rust（稳定工具链，≥1.77）。
3. 运行测试：

   ```bash
   cargo test
   ```

4. 以发布模式构建：

   ```bash
   cargo build --release
   ```

二进制文件位于 `target/release/myBioTools`。

## 添加新工具

1. 在 `src/` 下创建新模块（例如 `src/new_tool.rs`）。
2. 定义 `NewToolArgs` 结构体，使用 `#[derive(Args)]`。
3. 实现 `run(args: NewToolArgs) -> anyhow::Result<()>` 函数。
4. 在 `src/lib.rs` 中声明模块，并在 `src/main.rs` 中添加子命令枚举变体。
5. 在 `main.rs` 中添加匹配分支来路由该命令。




