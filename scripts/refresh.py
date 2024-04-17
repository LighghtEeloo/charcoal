import os

if __name__ == "__main__":
    with open("~/.zhistory", 'r') as f:
        lines = [line for line in f.readlines() if line.startswith("ww ")]
    print(lines)