class CathedralP2PHost:
    def __init__(self, listen_addr):
        self.listen_addr = listen_addr
        self.peers = []

    def set_stream_handler(self, protocol, handler):
        pass

    async def start(self):
        pass

    async def connect(self, target):
        self.peers.append(target)
        pass

    async def send_message(self, addr, message):
        pass
