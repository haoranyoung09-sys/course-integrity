"""Split text into non-overlapping token windows using a real tokenizer.

The official demo calls the tokenizer with max_length=512 and truncation=True.
This module refuses that silent cut. A window is scored only when the
tokenizer's full encoding, including special tokens, is at most 512.
"""

from provenance import MAX_TOKENS


def special_token_count(tokenizer) -> int:
    encoded = tokenizer.encode("", add_special_tokens=True)
    return len(encoded)


def content_windows(tokenizer, text: str):
    """Return windows as dicts. Does not score them.

    Offsets come from a fast tokenizer's offset mapping. Characters the
    tokenizer does not assign to a content token are left uncovered.
    """
    specials = special_token_count(tokenizer)
    if specials >= MAX_TOKENS:
        return {
            "special_tokens": specials,
            "content_limit": 0,
            "windows": [],
            "reason": "special_tokens_fill_the_limit",
        }
    content_limit = MAX_TOKENS - specials
    encoded = tokenizer(
        text,
        add_special_tokens=False,
        return_offsets_mapping=True,
        truncation=False,
        return_attention_mask=False,
    )
    offsets = list(encoded["offset_mapping"])
    if not offsets:
        return {
            "special_tokens": specials,
            "content_limit": content_limit,
            "windows": [],
            "reason": "no_tokens",
        }
    windows = []
    start_token = 0
    index = 0
    while start_token < len(offsets):
        stop_token = min(start_token + content_limit, len(offsets))
        char_start = offsets[start_token][0]
        char_end = offsets[stop_token - 1][1]
        if char_end <= char_start:
            return {
                "special_tokens": specials,
                "content_limit": content_limit,
                "windows": [],
                "reason": "tokenizer_offset_empty",
            }
        piece = text[char_start:char_end]
        full = tokenizer.encode(piece, add_special_tokens=True)
        token_count = len(full)
        windows.append(
            {
                "window_index": index,
                "char_start": int(char_start),
                "char_end": int(char_end),
                "token_count": int(token_count),
                "content_tokens": stop_token - start_token,
                "within_limit": token_count <= MAX_TOKENS,
            }
        )
        index += 1
        start_token = stop_token
    return {
        "special_tokens": specials,
        "content_limit": content_limit,
        "windows": windows,
        "reason": None,
    }
