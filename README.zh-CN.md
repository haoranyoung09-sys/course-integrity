# course-integrity 中文快速开始

`course-integrity` 是一个**本地**的课程作业预检小工具，面向英语授课的本科写作、实验报告和小组讨论草稿，并直接处理中文。它不是 Turnitin，也不是知网，不能代替它们，也不会连接它们的服务器或语料库。

英文说明和完整规则见 [README.md](README.md) 与 [docs/methodology.md](docs/methodology.md)。下面这句英文是报告里的正式界限，中英文文档用同一句，避免两边说法不一样：

Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

中文意思：这些只是描述性的复核信号。它们不是“AI 概率”，不是经过校准的风险分，不带置信区间，也不声称准确率。它们不足以判断作者是谁。相似命中也不是抄袭结论。

报告里没有总分。数字不会被加权、相加，或变成一个风险标签。

## 构建

需要 Rust 1.83 或更新版本。构建时会下载依赖；运行时不访问网络。

```bash
cargo build --locked --release
./target/release/course-integrity --help
```

## 查看一篇草稿

```bash
cargo run --locked -- scan examples/lab-report-zh.md
cargo run --locked -- scan examples/lab-report-en.md --format json
```

只运行 `scan` 时**不会**做重合检查。输出里会写明这一点。没有命中，不等于原稿是原创，因为根本没有比较。

退出码 0 只表示命令跑完了，不表示作业通过。

中文按**字**切分，没有词典分词。英文按词切分。两边的句长和词汇比不能直接对比。句子变体、重复、词汇多样性、衔接词都只是描述。实验报告里很短的步骤句和大量数字很常见，工具会说明这一点，不会把它当成问题结论。

## 本地语料

数据库路径必须用 `--db` 写明。工具没有默认数据库，不会在你不知情时开始保存作业。

```bash
mkdir -p .course-integrity
cargo run --locked -- corpus add fixtures/synthetic/zh_source.md \
  --db .course-integrity/demo.db
cargo run --locked -- corpus check fixtures/synthetic/zh_exact_paragraph.md \
  --db .course-integrity/demo.db
cargo run --locked -- corpus stats --db .course-integrity/demo.db
```

达到可比较长度的文件，**原文会完整保存在这个 SQLite 文件里**。请不要提交这个数据库，也不要把它指向你无权保存的作业。短到无法比较的文件不会写入。

重合分数是 containment：分子是查询段落和来源段落共有的、不重复的 4 词元片段数，**分母是查询段落自己的不重复 4 词元片段数**。默认阈值是 0.8，并且至少要有 4 个共有片段。这不是抄袭判决。小组共用的题目说明、引用、模板和实验套话都可能命中。

同一个路径再次检查时，默认会排除自己已经存入的那一条，避免把自己和自己比成“命中”。内容相同但标签不同的文件仍会报告为命中。`--include-self` 可以关掉这个排除。

## 示例

- [examples/lab-report-zh.md](examples/lab-report-zh.md) 合成中文实验记录
- [examples/lab-report-en.md](examples/lab-report-en.md) 合成英文实验记录
- [examples/group-discussion.md](examples/group-discussion.md) 合成小组讨论

`fixtures/synthetic/` 里的文件都是合成的，只用于测试和演示。它们不是学生作业，也没有被标成“人写”或“机器写”的标准答案。

## 隐私

没有上传，没有遥测，没有付费模型。细节见 [docs/privacy.md](docs/privacy.md)。
