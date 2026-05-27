import struct
import numpy as np

class ArkheOSGGUF:
    def __init__(self, name):
        self.name = name
        self.metadata = {}
        self.tensors = []

    def set_metadata(self, key, value):
        self.metadata[key] = value

    def add_tensor(self, name, shape, dtype, offset):
        self.tensors.append({
            "name": name,
            "shape": shape,
            "dtype": dtype,
            "offset": offset
        })

    def to_gguf_binary(self):
        return b"GGUF_MOCK_DATA_" + self.name.encode()

    def compute_checksum(self):
        return "1234567890abcdef"
