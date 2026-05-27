import torch
import torch.nn as nn

class DummyModel(nn.Module):
    def __init__(self):
        super().__init__()
        self.linear = nn.Linear(10, 10)

    def forward(self, x):
        return self.linear(x)

model = DummyModel()
torch.save({"model": model.state_dict()}, "checkpoint.pt")
print("Dummy model saved to checkpoint.pt")
