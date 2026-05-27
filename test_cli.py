import subprocess
import pty
import os
import sys

def test_cli():
    # Execute the CLI, write 'exit' and ensure it handles it cleanly
    p = subprocess.Popen(["python3", "arkhe-ai/arkhe_chat.py"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err = p.communicate(b"exit\n")
    print(out.decode())
    print(err.decode())
    assert "Welcome to ARKHE-OS Local CLI Chat." in out.decode()

test_cli()
