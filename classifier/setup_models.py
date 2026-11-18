#!/usr/bin/env python3
"""Download the pinned AIGC v3 weights and convert them to safetensors.

This is explicit. A normal scan does not run it and does not download anything.
The published files are pytorch_model.bin. Conversion uses torch.load(..., weights_only=True)
and then deletes the pickle file so later inference does not load it.
"""

import argparse
import hashlib
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from provenance import EN_REPO, EN_REVISION, LICENSE, ZH_REPO, ZH_REVISION


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def convert_one(repo: str, revision: str, dest: Path) -> None:
    from huggingface_hub import snapshot_download
    import torch
    from safetensors.torch import save_file

    target = dest / repo.split("/")[-1]
    target.mkdir(parents=True, exist_ok=True)
    snapshot_download(
        repo_id=repo,
        revision=revision,
        local_dir=str(target),
        allow_patterns=[
            "config.json",
            "pytorch_model.bin",
            "tokenizer_config.json",
            "special_tokens_map.json",
            "vocab.json",
            "merges.txt",
            "vocab.txt",
            "tokenizer.json",
        ],
    )
    bin_path = target / "pytorch_model.bin"
    safe_path = target / "model.safetensors"
    if not bin_path.is_file() and not safe_path.is_file():
        raise SystemExit(f"{repo}: neither pytorch_model.bin nor model.safetensors was downloaded")
    if bin_path.is_file():
        state = torch.load(bin_path, map_location="cpu", weights_only=True)
        if not isinstance(state, dict):
            raise SystemExit(f"{repo}: checkpoint is not a state dict")
        tensors = {key: value for key, value in state.items() if hasattr(value, "shape")}
        if len(tensors) != len(state):
            raise SystemExit(f"{repo}: checkpoint contains non-tensor objects; refused")
        save_file(
            dict(sorted(tensors.items())),
            str(safe_path),
            metadata={"format": "pt"},
        )
        bin_path.unlink()
    record = {
        "repo": repo,
        "revision": revision,
        "license": LICENSE,
        "weight_format": "safetensors",
        "weight_sha256": sha256_file(safe_path),
        "conversion": "torch.load(pytorch_model.bin, map_location=cpu, weights_only=True); safetensors metadata format=pt; pickle file deleted",
        "label_map": {"0": "human", "1": "ai"},
        "label_map_sources": [
            "https://huggingface.co/spaces/yuchuantian/AIGC_text_detector/blob/main/app.py",
            "https://github.com/YuchuanTian/AIGC_text_detector/blob/main/dataset.py",
        ],
    }
    (target / "course-integrity-provenance.json").write_text(
        json.dumps(record, indent=2) + "\n", encoding="utf-8"
    )
    print(f"ready {repo}@{revision} sha256={record['weight_sha256']}", file=sys.stderr)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dest", required=True, type=Path)
    args = parser.parse_args()
    args.dest.mkdir(parents=True, exist_ok=True)
    convert_one(EN_REPO, EN_REVISION, args.dest)
    convert_one(ZH_REPO, ZH_REVISION, args.dest)


if __name__ == "__main__":
    main()
