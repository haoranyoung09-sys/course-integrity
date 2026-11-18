# course-integrity 中文说明

[English](README.md)

[![声明的 Rust 版本为 1.83 或更高。CI 使用 1.83.0](https://img.shields.io/badge/Rust-1.83%2B-000000?logo=rust&logoColor=white)](Cargo.toml)
[![CI：cargo fmt、cargo test 与 cargo clippy](https://github.com/haoranyoung09-sys/course-integrity/actions/workflows/ci.yml/badge.svg)](https://github.com/haoranyoung09-sys/course-integrity/actions/workflows/ci.yml)
[![许可证 MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![仓库内 crate 版本 0.3.0。未发布到 crates.io，也没有 GitHub release](https://img.shields.io/badge/version-0.3.0%20source%20only-lightgrey)](Cargo.toml)

`course-integrity` 0.3.0 是一个**本地**的课程作业预检小工具，面向英语授课的本科写作、实验报告和小组讨论草稿，并直接处理中文。它读取 UTF-8 的 `.txt` / `.md`，并从 `.docx` 和带文字层的 PDF 里抽出文字。它也会列出隐藏字符和不寻常的换行。扫描件、图片和图表不会识别，也没有 OCR。

语料比对和字符清单都不是 AI 检测。红绿词元水印只有在拿到确切方案、分词器、密钥和参数时才能核验。这个程序没有这些参数，也不会去猜一种水印。

它不是 Turnitin，也不是知网，不能代替它们，也不会连接它们的服务器或语料库。

数据库仍是第 1 版，0.1.0 建的库可以直接打开。DOCX 和 PDF 存进库的是抽出的纯文本，不是原始文件。引用、参考文献，以及用 `--boilerplate` 指定的实验讲义，会单独列在 “set aside” 里，不当作正文重合的主要证据。比对方式仍然是 4 词元片段，不是语义理解。

离线 HTML 报告和 Markdown、JSON 一起提供。HTML 没有脚本，也不会访问网络。双向控制符和其他隐藏字符在报告里写成 `U+` 码位，避免它们把页面的字序打乱。

英文说明和完整规则见 [README.md](README.md) 与 [docs/methodology.md](docs/methodology.md)。下面这句英文是报告里的正式界限，中英文文档用同一句，避免两边说法不一样：

Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

中文意思：这些只是描述性的复核信号。它们不是“AI 概率”，不是经过校准的风险分，不带置信区间，也不声称准确率。它们不足以判断作者是谁。相似命中也不是抄袭结论。

报告里没有总分。数字不会被加权、相加，或变成一个风险标签。

`Cargo.toml` 里的 `rust-version` 是 1.83，CI 安装的也是 Rust 1.83.0。这是声明并在 CI 里测过的下限，没有再拿更旧的编译器测量。

## 构建

需要 Rust 1.83 或更新版本。在本仓库检出目录里构建。第一次构建会按锁文件下载依赖；运行时不访问网络。没有单独的安装步骤，也没有 crates.io 上的包。

```bash
cargo build --locked --release
./target/release/course-integrity --help
```

许可证是 [MIT](LICENSE)。版本号 0.3.0 只写在 `Cargo.toml` 里。`publish = false`。没有 crates.io 发布，也没有 GitHub release 标签。

## 演示

在干净的检出目录里运行：

```bash
bash scripts/demo.sh
```

脚本会扫描合成的英文实验笔记、中文实验笔记、英文 DOCX，以及一个带隐藏字符的小文件。它把 `fixtures/synthetic/shared_prompt.md` 放进临时数据库，再检查 `fixtures/synthetic/group_draft.md`，然后删除那个数据库。用浏览器打开：

- [examples/demo/lab-report-en.html](examples/demo/lab-report-en.html)
- [examples/demo/lab-report-zh.html](examples/demo/lab-report-zh.html)
- [examples/demo/group-draft.html](examples/demo/group-draft.html)
- [examples/demo/hidden-marks.html](examples/demo/hidden-marks.html)

这些 HTML 就是该脚本的输出，内容是合成的。说明见 [examples/demo/README.md](examples/demo/README.md)。

## 查看一篇草稿

```bash
cargo run --locked -- scan examples/lab-report-zh.md
cargo run --locked -- scan examples/lab-report-en.md --format json
cargo run --locked -- scan fixtures/synthetic/lab-report-zh.docx --format html --out-dir /tmp/ci-report
cargo run --locked -- scan fixtures/synthetic/hidden-marks.txt
```

`--format all` 会写出 `.review.md`、`.review.html` 和 `.review.json`，并且必须带 `--out-dir`。写出的路径打在标准错误上，这样标准输出上的 JSON 仍然可以被解析。

只运行 `scan` 时**不会**做重合检查。输出里会写明这一点。没有命中，不等于原稿是原创，因为根本没有比较。

退出码 0 只表示命令跑完了，不表示作业通过。退出码 2 表示命令行不对。退出码 1 表示文件或数据库读不了。

中文按**字**切分，没有词典分词。英文按词切分。两边的句长和词汇比不能直接对比。句子变体、重复、词汇多样性、衔接词都只是描述。实验报告里很短的步骤句和大量数字很常见，工具会说明这一点，不会把它当成问题结论。

隐藏字符清单包括字节顺序标记、UTF-8 文件里的 CRLF 或单独的 CR、行尾空格、不换行空格、零宽字符和双向控制符。每一条都有行号、列号和码位。表意文字空格（U+3000）在中文里很常见，不算隐藏控制符。DOCX 和 PDF 不报告换行风格，因为抽取程序自己插入了段落分隔。

## 本地语料

数据库路径必须用 `--db` 写明。工具没有默认数据库，不会在你不知情时开始保存作业。

```bash
mkdir -p .course-integrity
cargo run --locked -- corpus add fixtures/synthetic/shared_prompt.md \
  --db .course-integrity/demo.db
cargo run --locked -- corpus check fixtures/synthetic/group_draft.md \
  --db .course-integrity/demo.db
cargo run --locked -- corpus stats --db .course-integrity/demo.db
cargo run --locked -- corpus list --db .course-integrity/demo.db
cargo run --locked -- corpus remove --db .course-integrity/demo.db --id 1
```

`corpus remove --id` 只从这一个本地数据库里删除该条记录和它的段落。

不把讲义写进数据库、只在本次检查里把它排除在主要证据之外：

```bash
cargo run --locked -- scan fixtures/synthetic/group_draft.md \
  --boilerplate fixtures/synthetic/shared_prompt.md \
  --db .course-integrity/demo.db
```

只把文件夹里的多份草稿在内存中两两互查、不写任何数据库文件：

```bash
cargo run --locked -- scan fixtures/synthetic --pairwise --format html --out-dir /tmp/ci-pairwise
```

达到可比较长度的 UTF-8 文件，原文会保存在这个 SQLite 文件里。DOCX 和 PDF 保存的是抽出的纯文本，不是原始字节。请不要提交这个数据库，也不要把它指向你无权保存的作业。短到无法比较的文件不会写入。`corpus list` 不打印正文，但正文仍在数据库里。

重合分数是 containment：分子是查询段落和来源段落共有的、不重复的 4 词元片段数，**分母是查询段落自己的不重复 4 词元片段数**。默认阈值是 0.8，并且至少要有 4 个共有片段。这不是抄袭判决。小组共用的题目说明、引用、模板和实验套话都可能命中。

同一个路径再次检查时，默认会排除自己已经存入的那一条，避免把自己和自己比成“命中”。内容相同但标签不同的文件仍会报告为命中。`--include-self` 可以关掉这个排除。

## 示例

- [examples/lab-report-zh.md](examples/lab-report-zh.md) 合成中文实验记录
- [examples/lab-report-en.md](examples/lab-report-en.md) 合成英文实验记录
- [examples/group-discussion.md](examples/group-discussion.md) 合成小组讨论
- [fixtures/synthetic/hidden-marks.txt](fixtures/synthetic/hidden-marks.txt) 合成的 CRLF、不换行空格、零宽空格和双向控制符

`fixtures/synthetic/` 里的文件都是合成的，只用于测试和演示。它们不是学生作业，也没有被标成“人写”或“机器写”的标准答案。

## 隐私

没有上传，没有遥测，没有付费模型。扫描不会改你指定的文件。细节见 [docs/privacy.md](docs/privacy.md)。
