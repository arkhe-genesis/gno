"""
Cathedral ARKHE v9.0 LOGOS — Multimodal Early Fusion
V9-007: Fusão early-stage para texto, imagem, áudio.
Tokens multimodais compartilham o mesmo espaço de embedding.
Baseado em: Gemini 2.5, GPT-4o early fusion (2025).
Selo: MULTIMODAL-FUSION-v9.0.0-2026-01-15
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple, Union

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class MultimodalConfig:
    d_model: int = 4096
    # Modalities
    support_vision: bool = True
    support_audio: bool = True
    # Vision
    vision_patch_size: int = 14
    vision_image_size: int = 336
    vision_n_patches: int = 576       # (336/14)^2
    vision_d_proj: int = 1024
    # Audio
    audio_sample_rate: int = 16000
    audio_chunk_ms: int = 30
    audio_n_mels: int = 128
    audio_d_proj: int = 1024
    audio_frames_per_chunk: int = 150  # 30ms * 16000 / 1000 * 2 (hop)
    # Fusion
    fusion_type: str = "early_add"     # "early_add", "early_concat", "cross_attn"
    modality_tokens_per_type: int = 1  # Special tokens: <text>, <image>, <audio>
    # Safety
    vision_safety_filter: bool = True  # NSFW detection antes de fusion


class VisionEncoder(nn.Module):
    """
    Encoder visual: patches → d_model space.
    Usa ViT-style patch embedding + positional encoding.
    """

    def __init__(self, config: MultimodalConfig):
        super().__init__()
        self.config = config
        n_patches = config.vision_n_patches

        # Patch embedding
        self.patch_embed = nn.Conv2d(
            3, config.vision_d_proj,
            kernel_size=config.vision_patch_size,
            stride=config.vision_patch_size,
        )

        # Positional encoding
        self.pos_embed = nn.Parameter(torch.randn(1, n_patches + 1, config.vision_d_proj) * 0.02)
        self.cls_token = nn.Parameter(torch.randn(1, 1, config.vision_d_proj) * 0.02)

        # Project to d_model
        self.proj = nn.Linear(config.vision_d_proj, config.d_model, bias=False)
        self.norm = nn.RMSNorm(config.d_model, eps=1e-5)

    def forward(self, pixel_values: torch.Tensor) -> torch.Tensor:
        """
        Args:
            pixel_values: (B, 3, H, W)
        Returns:
            tokens: (B, n_patches+1, d_model)
        """
        B = pixel_values.shape[0]
        patches = self.patch_embed(pixel_values)  # (B, d_proj, n_h, n_w)
        patches = patches.flatten(2).transpose(1, 2)  # (B, n_patches, d_proj)

        # Add CLS + positional
        cls = self.cls_token.expand(B, -1, -1)
        tokens = torch.cat([cls, patches], dim=1)
        tokens = tokens + self.pos_embed

        # Project to d_model
        tokens = self.proj(tokens)
        tokens = self.norm(tokens)
        return tokens


class AudioEncoder(nn.Module):
    """
    Encoder de áudio: mel spectrogram → d_model space.
    """

    def __init__(self, config: MultimodalConfig):
        super().__init__()
        self.config = config

        # Mel features → embedding
        self.input_proj = nn.Linear(
            config.audio_n_mels * config.audio_frames_per_chunk,
            config.audio_d_proj,
        )

        # Positional encoding
        self.pos_embed = nn.Parameter(
            torch.randn(1, 100, config.audio_d_proj) * 0.02  # Max 100 chunks
        )

        # Project to d_model
        self.proj = nn.Linear(config.audio_d_proj, config.d_model, bias=False)
        self.norm = nn.RMSNorm(config.d_model, eps=1e-5)

    def forward(self, mel_spec: torch.Tensor) -> torch.Tensor:
        """
        Args:
            mel_spec: (B, n_chunks, n_mels, n_frames)
        Returns:
            tokens: (B, n_chunks, d_model)
        """
        B, N, M, F = mel_spec.shape
        flat = mel_spec.reshape(B, N, M * F)
        hidden = self.input_proj(flat)
        hidden = hidden + self.pos_embed[:, :N, :]
        hidden = self.proj(hidden)
        return self.norm(hidden)


class ModalityTokenEmbedder(nn.Module):
    """Embeds special tokens que indicam a modalidade."""

    def __init__(self, d_model: int, n_modalities: int = 3):
        super().__init__()
        self.embed = nn.Embedding(n_modalities, d_model)

    def forward(self, modality_id: int, batch_size: int) -> torch.Tensor:
        ids = torch.tensor([modality_id], device=self.embed.weight.device)
        return self.embed(ids).unsqueeze(0).expand(batch_size, -1, -1)


class MultimodalFusion(nn.Module):
    """
    Fusão multimodal early-stage.

    Todos os modalities são convertidos para (B, N, d_model) e
    concatenados na sequência de entrada com tokens especiais:

    <text> text_tokens... <image> vision_tokens... <audio> audio_tokens...

    O backbone processa a sequência mista uniformemente.
    Safety filter aplicado antes da fusão para vision.
    """

    def __init__(self, config: MultimodalConfig):
        super().__init__()
        self.config = config

        # Encoders por modalidade
        if config.support_vision:
            self.vision = VisionEncoder(config)
        if config.support_audio:
            self.audio = AudioEncoder(config)

        # Modality tokens
        self.modality_embedder = ModalityTokenEmbedder(
            config.d_model, config.modality_tokens_per_type
        )

        # Safety filter para vision
        if config.support_vision and config.vision_safety_filter:
            self.nsfw_detector = nn.Sequential(
                nn.Linear(config.d_model, config.d_model // 4),
                nn.GELU(),
                nn.Linear(config.d_model // 4, 1),
                nn.Sigmoid(),
            )

    def forward(self, text_tokens: torch.Tensor,
                pixel_values: Optional[torch.Tensor] = None,
                mel_spec: Optional[torch.Tensor] = None) -> Tuple[torch.Tensor, Dict]:
        """
        Args:
            text_tokens: (B, L_text, D) — text embeddings
            pixel_values: (B, 3, H, W) — optional image
            mel_spec: (B, N_chunks, n_mels, n_frames) — optional audio

        Returns:
            fused: (B, L_total, D) — sequência multimodal fusionada
            info: dict com metadados
        """
        B = text_tokens.shape[0]
        parts = []
        modality_order = []

        # Text
        mod_token = self.modality_embedder(0, B)  # <text>
        parts.append(mod_token)
        parts.append(text_tokens)
        modality_order.append(("text", text_tokens.shape[1]))

        # Vision
        if pixel_values is not None and self.config.support_vision:
            # Safety check
            vision_tokens = self.vision(pixel_values)  # (B, n_patches+1, D)

            if self.config.vision_safety_filter and hasattr(self, 'nsfw_detector'):
                cls_token = vision_tokens[:, 0:1, :]
                nsfw_score = self.nsfw_detector(cls_token).mean().item()
                if nsfw_score > 0.8:
                    vision_tokens = vision_tokens[:, :1, :] * 0  # Zero out
                    vision_tokens = vision_tokens + self.modality_embedder(0, B) * 0  # Neutral

            mod_token = self.modality_embedder(1, B)  # <image>
            parts.append(mod_token)
            parts.append(vision_tokens)
            modality_order.append(("vision", vision_tokens.shape[1]))

        # Audio
        if mel_spec is not None and self.config.support_audio:
            audio_tokens = self.audio(mel_spec)
            mod_token = self.modality_embedder(2, B)  # <audio>
            parts.append(mod_token)
            parts.append(audio_tokens)
            modality_order.append(("audio", audio_tokens.shape[1]))

        # Concatenate all parts
        fused = torch.cat(parts, dim=1)

        info = {
            "module": "MultimodalFusion",
            "seal": "MULTIMODAL-FUSION-v9.0.0-2026-01-15",
            "modality_order": modality_order,
            "total_length": fused.shape[1],
            "text_length": text_tokens.shape[1],
        }

        return fused, info

    def get_telemetry(self) -> dict:
        return {
            "module": "MultimodalFusion",
            "version": "9.0.0",
            "substrate": "v9-multimodal",
            "seal": "MULTIMODAL-FUSION-v9.0.0-2026-01-15",
            "vision": self.config.support_vision,
            "audio": self.config.support_audio,
            "fusion_type": self.config.fusion_type,
            "nsfw_filter": self.config.vision_safety_filter,
        }
