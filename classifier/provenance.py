"""Pinned identity for the optional local classifiers.

Label map is taken from the author's code, not from config.json
(config.json has no id2label) and not from sample outputs.

- Hugging Face Space yuchuantian/AIGC_text_detector app.py loads
  AIGC_detector_env3 and AIGC_detector_zhv3 and sets
  id2label = ['Human', 'AI'] and ['人类', 'AI'] on the argmax of the softmax.
- YuchuanTian/AIGC_text_detector dataset.py assigns label 1 to the
  ChatGPT text slot and label 0 to the human text slot
  (comment: real->label1->chatgpt).
"""

EN_REPO = "yuchuantian/AIGC_detector_env3"
EN_REVISION = "3ceb1d1bb7b9ca11585ce3f4299950e1c465b6d8"
ZH_REPO = "yuchuantian/AIGC_detector_zhv3"
ZH_REVISION = "47695ff451b32c225dd938f4f478f7fdc6aa6bb0"
LICENSE = "apache-2.0"
MAX_TOKENS = 512
LABELS = {0: "human", 1: "ai"}
