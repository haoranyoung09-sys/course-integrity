# Optional classifier smoke

These numbers are one CPU run on the synthetic files already in this repository. They are not an accuracy evaluation, not a calibration, and not a claim about student work. The files are labeled as teaching examples and are not ground truth for human or AI authorship.

The label next to each number in the report is: AI 模型分类分数（未校准） / AI classifier score (uncalibrated)

This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.

## What was installed

CPU torch was installed before the other pins, because the `+cpu` wheel is not a PyPI `torch==2.5.1` pin:

```bash
python3 -m venv .course-integrity/venv
. .course-integrity/venv/bin/activate
python -m pip install "torch==2.5.1+cpu" --index-url https://download.pytorch.org/whl/cpu
python -m pip install -r classifier/requirements.txt
python classifier/setup_models.py --dest .course-integrity/models
```

Versions that completed that install: `torch==2.5.1+cpu`, `transformers==4.46.3`, `tokenizers==0.20.3`, `safetensors==0.4.5`, `huggingface_hub==0.26.5`, `numpy==2.1.3`.

| Model | Revision | License | Safetensors SHA-256 |
| --- | --- | --- | --- |
| `yuchuantian/AIGC_detector_env3` | `3ceb1d1bb7b9ca11585ce3f4299950e1c465b6d8` | apache-2.0 | `4657ba99b58fd2dacf484e0d3a2266cdc5de5453cf1965c15d508115545d60ee` |
| `yuchuantian/AIGC_detector_zhv3` | `47695ff451b32c225dd938f4f478f7fdc6aa6bb0` | apache-2.0 | `babf51f45f0110215629bbaf34d439dc5ab6a5070c95dd0ab43e5adf045ec631` |

The published checkpoints are `pytorch_model.bin` only. Setup used `torch.load(..., map_location="cpu", weights_only=True)`, wrote `model.safetensors` with metadata `format=pt`, and deleted the pickle file. Inference used `local_files_only=True` and `use_safetensors=True`. `trust_remote_code` was not used.

Class index 0 is human and class index 1 is AI. The author's Space `app.py` sets `id2label = ['Human', 'AI']` and `['人类', 'AI']`. `dataset.py` agrees once the return order is followed: `load_texts_original` returns ChatGPT answers first and human answers second, those are bound to `(real_train, fake_train)`, and label 1 is then the ChatGPT slot. The pinned `config.json` files have no `id2label`.

## Commands and timing

```bash
export PATH="$PWD/.course-integrity/venv/bin:$PATH"
export HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 HF_HUB_DISABLE_TELEMETRY=1
unset HF_TOKEN HUGGING_FACE_HUB_TOKEN
cargo run --locked --quiet -- scan examples/lab-report-en.md --ai-classify --ai-models .course-integrity/models --format all --out-dir /tmp/ai-smoke-final/en
cargo run --locked --quiet -- scan examples/lab-report-zh.md --ai-classify --ai-models .course-integrity/models --format all --out-dir /tmp/ai-smoke-final/zh
```

Wall clock on this machine, including process start and model load, was 1.907 seconds for the English example and 1.873 seconds for the Chinese example. A repeated English sentence of 6560 characters took 2.138 seconds. The same Chinese sentence repeated to 1600 characters took 2.247 seconds. That is one machine, not a performance guarantee.

The same English command was then run inside `unshare --net --map-root-user`. It still returned status `ok` and the same first score, 96.063. After setup, inference did not need the network.

Committed copies of those two example reports:

- [examples/demo/classifier-smoke-en.html](../examples/demo/classifier-smoke-en.html)
- [examples/demo/classifier-smoke-zh.html](../examples/demo/classifier-smoke-zh.html)

`scripts/demo.sh` does not pass `--ai-classify`. Its reports stay `not_run` and do not show a number.

## Measured scores

English example `examples/lab-report-en.md`. Model `env3`. Every paragraph was under 512 tokens, so each paragraph is one window. Scores were not averaged.

| Paragraph | Tokens | Class | Score |
| --- | ---: | --- | ---: |
| 1 | 33 | ai | 96.063 |
| 2 | 10 | human | 38.1024 |
| 3 | 25 | ai | 99.466 |
| 4 | 4 | ai | 53.8459 |
| 5 | 61 | ai | 72.3675 |
| 6 | 4 | human | 42.8105 |
| 7 | 59 | human | 9.2844 |
| 8 | 4 | ai | 56.6981 |
| 9 | 85 | human | 5.4137 |

Scored characters 1117. Skipped 0. Uncovered 0. Chunks with class-1 softmax greater than 0.5 contain 516 of those 1117 characters, so the flagged-character share is 0.462. The denominator is scored characters, not the file length. This share is not the portion of the file written by AI.

Chinese example `examples/lab-report-zh.md`. Model `zhv3`. The first paragraph contains the Latin words `course-integrity`, so the language check called it mixed and did not score its 61 characters. That is not 0%.

| Paragraph | Tokens | Class | Score |
| --- | ---: | --- | ---: |
| 1 | not scored | mixed | none |
| 2 | 7 | human | 30.2897 |
| 3 | 31 | human | 37.6919 |
| 4 | 6 | ai | 72.2747 |
| 5 | 74 | human | 15.9945 |
| 6 | 6 | human | 25.7306 |
| 7 | 67 | ai | 70.1804 |
| 8 | 6 | human | 36.4657 |
| 9 | 108 | ai | 98.6571 |

Scored characters 340. Skipped 61. Uncovered 0. Flagged-character share 0.6265, which is 213/340 scored characters, using the same 0.5 rule.

A synthetic English paragraph made by repeating one sentence was split into three non-overlapping windows of 512, 512, and 263 tokens. Scores were 23.0387, 25.5951, and 17.8479, all predicted human. Two characters between windows were not covered by an offset and were not given a score. They were not folded into a document percentage.

The same kind of repeated Chinese sentence, 1600 characters, became windows of 512, 512, 512, and 72 tokens. Scores were 99.5162, 99.472, 99.5162, and 33.4727. Coverage of that string was complete. The four numbers were not averaged.

## What this run does not show

Short headings in the synthetic examples received scores, including windows of 4 tokens. Mixed text was left unscored unless a language is forced. Academic and non-native English text can be classified the wrong way. This page does not state a minimum length or an accuracy. A failed or skipped run stays `not_run` or `unavailable` and does not become 0%.

## 这次冒烟说明什么

上面的数字来自本仓库里的合成示例，在 CPU 上跑了一次。它们不是准确率，也不是校准结果，更不是学生作业的结论。报告里每个分数旁边都写着「AI 模型分类分数（未校准） / AI classifier score (uncalibrated)」。中文示例的第一段因为夹有英文 `course-integrity` 被标成混合语言，没有打分，也没有写成 0%。窗口分数没有被平均成整篇的 AI 百分比。
