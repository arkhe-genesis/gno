#!/usr/bin/env python3
"""
ARKHE Global Mesh — Discovery Protocol
Substrato 972 — ARKHE-GLOBAL-MESH

Protocolo de descoberta de nos usando multicast DNS + DHT.
"""

import asyncio
import socket
from typing import List, Optional
from dataclasses import dataclass

@dataclass
class DiscoveryMessage:
    node_id: str
    ip: str
    port: int
    region: str
    substrates: List[int]
    seal: str

class MeshDiscovery:
    """Descoberta de nos na malha."""

    def __init__(self, multicast_group: str = "224.0.0.251", port: int = 5353):
        self.multicast_group = multicast_group
        self.port = port
        self.discovered_nodes: List[DiscoveryMessage] = []

    async def announce(self, node: DiscoveryMessage):
        """Anuncia no na malha via multicast."""
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)

        message = f"ARKHE:{node.node_id}:{node.ip}:{node.port}:{node.region}"
        sock.sendto(message.encode(), (self.multicast_group, self.port))

    async def listen(self) -> DiscoveryMessage:
        """Escuta anuncios na malha."""
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.bind(("", self.port))

        mreq = socket.inet_aton(self.multicast_group) + socket.inet_aton("0.0.0.0")
        sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)

        while True:
            data, addr = await asyncio.get_event_loop().sock_recv(sock, 1024)
            # Parse message...
