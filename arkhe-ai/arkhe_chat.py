#!/usr/bin/env python3
import argparse
import sys
import requests
import json

SERVER_URL = "http://localhost:8080/completion"

def chat():
    print("Welcome to ARKHE-OS Local CLI Chat.")
    print("Type 'exit' or 'quit' to exit.")
    while True:
        try:
            user_input = input("You: ")
            if user_input.strip().lower() in ["exit", "quit"]:
                break

            payload = {
                "prompt": f"User: {user_input}\nARKHE-OS:",
                "n_predict": 128,
                "temperature": 0.7,
                "stop": ["User:", "\n"]
            }

            response = requests.post(SERVER_URL, json=payload)
            response.raise_for_status()

            data = response.json()
            reply = data.get("content", "").strip()
            print(f"ARKHE-OS: {reply}")

        except requests.exceptions.RequestException as e:
            print(f"Error communicating with the server: {e}")
        except KeyboardInterrupt:
            break
        except Exception as e:
            print(f"An error occurred: {e}")

if __name__ == "__main__":
    chat()
