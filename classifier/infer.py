#!/usr/bin/env python3
"""Score chunks with the local safetensors classifiers.

Stdin is one JSON object. Stdout is one JSON object. The draft is not logged.
Hub access is forced off. trust_remote_code is not used.
"""

import json
import os
import sys
from pathlib import Path

os.environ["HF_HUB_OFFLINE"] = "1"
os.environ["TRANSFORMERS_OFFLINE"] = "1"
os.environ["HF_HUB_DISABLE_TELEMETRY"] = "1"
sys.path.insert(0, str(Path(__file__).resolve().parent))

from provenance import EN_REPO, EN_REVISION, LABELS, MAX_TOKENS, ZH_REPO, ZH_REVISION
from windows import content_windows

MODELS = {
    "en": {"repo": EN_REPO, "revision": EN_REVISION, "dirname": "AIGC_detector_env3", "kind": "roberta"},
    "zh": {"repo": ZH_REPO, "revision": ZH_REVISION, "dirname": "AIGC_detector_zhv3", "kind": "bert"},
}


def emit(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload, ensure_ascii=False))
    sys.stdout.write("\n")


def fail(reason: str) -> None:
    emit({"status": "not_run", "reason": reason, "chunks": []})


def load_record(models_dir: Path, key: str):
    spec = MODELS[key]
    folder = models_dir / spec["dirname"]
    record_path = folder / "course-integrity-provenance.json"
    weights = folder / "model.safetensors"
    if not record_path.is_file() or not weights.is_file():
        return None
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("repo") != spec["repo"] or record.get("revision") != spec["revision"]:
        return None
    if record.get("weight_format") != "safetensors":
        return None
    if (folder / "pytorch_model.bin").exists():
        return None
    return record, folder


def softmax_pair(logits):
    import math

    a, b = float(logits[0]), float(logits[1])
    m = max(a, b)
    ea, eb = math.exp(a - m), math.exp(b - m)
    total = ea + eb
    human, ai = ea / total, eb / total
    predicted = 1 if ai > human else 0
    return human, ai, predicted


def score_text(tokenizer, model, text: str):
    import torch

    encoded = tokenizer(
        text,
        return_tensors="pt",
        add_special_tokens=True,
        truncation=False,
    )
    token_count = int(encoded["input_ids"].shape[-1])
    if token_count > MAX_TOKENS:
        return None
    with torch.no_grad():
        logits = model(**encoded).logits[0].detach().to(dtype=torch.float64).cpu().tolist()
    if len(logits) != 2:
        return None
    human, ai, predicted = softmax_pair(logits)
    return {
        "token_count": token_count,
        "human_softmax": human,
        "ai_softmax": ai,
        "predicted_index": predicted,
    }


def main() -> None:
    try:
        request = json.load(sys.stdin)
    except json.JSONDecodeError:
        fail("bad_request")
        return
    models_dir = Path(request.get("models_dir", ""))
    chunks = request.get("chunks", [])
    if not isinstance(chunks, list):
        fail("bad_request")
        return
    needed = []
    for chunk in chunks:
        if not isinstance(chunk, dict):
            fail("bad_request")
            return
        language = chunk.get("language", "")
        if language in MODELS and language not in needed:
            needed.append(language)
    loaded = {}
    for key in needed:
        found = load_record(models_dir, key)
        if found is not None:
            loaded[key] = found
    if needed and not loaded:
        fail("missing_model")
        return

    sessions = {}
    model_meta = {}
    if loaded:
        import torch
        from transformers import BertForSequenceClassification, BertTokenizerFast
        from transformers import RobertaForSequenceClassification, RobertaTokenizerFast

        torch.set_grad_enabled(False)
        try:
            for key, (record, folder) in loaded.items():
                if MODELS[key]["kind"] == "roberta":
                    tokenizer = RobertaTokenizerFast.from_pretrained(folder, local_files_only=True)
                    model = RobertaForSequenceClassification.from_pretrained(
                        folder, local_files_only=True, use_safetensors=True
                    )
                else:
                    tokenizer = BertTokenizerFast.from_pretrained(folder, local_files_only=True)
                    model = BertForSequenceClassification.from_pretrained(
                        folder, local_files_only=True, use_safetensors=True
                    )
                model.eval()
                sessions[key] = (tokenizer, model)
                model_meta[key] = {
                    "repo": record["repo"],
                    "revision": record["revision"],
                    "weight_format": "safetensors",
                    "weight_sha256": record.get("weight_sha256", ""),
                    "license": record.get("license", ""),
                }
        except Exception:
            fail("model_load_failed")
            return

    chunks_out = []
    for chunk in chunks:
        parent_id = str(chunk.get("id", ""))
        text = chunk.get("text", "")
        language = chunk.get("language", "")
        if not isinstance(text, str) or not parent_id:
            fail("bad_request")
            return
        if language == "mixed" or language not in sessions:
            chunks_out.append(
                {
                    "parent_id": parent_id,
                    "window_index": 0,
                    "language": language,
                    "char_start": 0,
                    "char_end": len(text),
                    "scored": False,
                    "reason": "mixed_or_unsupported_language" if language == "mixed" else "model_not_loaded",
                }
            )
            continue
        tokenizer, model = sessions[language]
        plan = content_windows(tokenizer, text)
        if plan["reason"] is not None:
            chunks_out.append(
                {
                    "parent_id": parent_id,
                    "window_index": 0,
                    "language": language,
                    "char_start": 0,
                    "char_end": len(text),
                    "scored": False,
                    "reason": plan["reason"],
                    "special_tokens": plan["special_tokens"],
                    "content_limit": plan["content_limit"],
                }
            )
            continue
        for window in plan["windows"]:
            if not window["within_limit"]:
                chunks_out.append(
                    {
                        "parent_id": parent_id,
                        "window_index": window["window_index"],
                        "language": language,
                        "char_start": window["char_start"],
                        "char_end": window["char_end"],
                        "token_count": window["token_count"],
                        "scored": False,
                        "reason": "over_token_limit",
                    }
                )
                continue
            piece = text[window["char_start"] : window["char_end"]]
            scored = score_text(tokenizer, model, piece)
            if scored is None or scored["token_count"] > MAX_TOKENS:
                chunks_out.append(
                    {
                        "parent_id": parent_id,
                        "window_index": window["window_index"],
                        "language": language,
                        "char_start": window["char_start"],
                        "char_end": window["char_end"],
                        "scored": False,
                        "reason": "score_failed",
                    }
                )
                continue
            chunks_out.append(
                {
                    "parent_id": parent_id,
                    "window_index": window["window_index"],
                    "language": language,
                    "model_key": language,
                    "char_start": window["char_start"],
                    "char_end": window["char_end"],
                    "token_count": scored["token_count"],
                    "special_tokens": plan["special_tokens"],
                    "content_limit": plan["content_limit"],
                    "scored": True,
                    "human_softmax": scored["human_softmax"],
                    "ai_softmax": scored["ai_softmax"],
                    "predicted_index": scored["predicted_index"],
                }
            )

    emit(
        {
            "status": "ok",
            "reason": None,
            "max_tokens": MAX_TOKENS,
            "label_map": {str(index): name for index, name in LABELS.items()},
            "models": model_meta,
            "chunks": chunks_out,
        }
    )


if __name__ == "__main__":
    main()
