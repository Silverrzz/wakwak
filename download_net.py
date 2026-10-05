#!/usr/bin/env python3

import urllib.request
import hashlib
import os

def main():
    name = "shokupan_3"
    hash = "55d0aee05cefc1963bf6830b9bd5b358d0a3231a45e4d00bb63054679e661eeb"
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
