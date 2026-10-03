#!/usr/bin/env python3

import urllib.request
import hashlib
import os

def main():
    name = "focaccia"
    hash = "4ba976df7423158dca1e0997845a423fbe674afc92afc06e785184d3dd710969"
    path = "./nets/wakwak.nnue"

    os.makedirs(os.path.dirname(path), exist_ok=True)

    try:
        if hashlib.sha256(open(path, "rb").read()).digest().hex() == hash:
            print("Net already exists!")
            return
    except OSError:
        pass

    print(f"Downloading net {name} to {path}")
    net = urllib.request.urlopen(
        f"https://github.com/Silverrzz/wakwak-nets/releases/download/{name}/{name}.nnue"
    ).read()
    if hashlib.sha256(net).digest().hex() != hash:
        print("Invalid hash!")
        exit(1)
    open(path, "wb").write(net)

if __name__ == "__main__":
    main()
