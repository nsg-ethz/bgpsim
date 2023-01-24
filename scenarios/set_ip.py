#!/usr/bin/python

import json
import sys

filename = sys.argv[1]
ip = "100.0.1.0/24"
if len(sys.argv) >= 3:
    file_from = sys.argv[2]

print(f"set ip of {filename} to {ip}")

with open(filename, "r") as fp:
    data = json.load(fp)

data["settings"]["prefix"] = ip
data["settings"]["layer"] = "FwState"
data["settings"]["manual_simulation"] = True

with open(filename, "w") as fp:
    json.dump(data, fp)
