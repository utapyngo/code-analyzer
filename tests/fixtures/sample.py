import os
from pathlib import Path

class FileReader:
    def __init__(self, path):
        self.path = path

    def read(self):
        with open(self.path) as f:
            return f.read()

def process(data):
    return data.strip().upper()

def main():
    reader = FileReader("test.txt")
    content = reader.read()
    result = process(content)
    print(result)

if __name__ == "__main__":
    main()
