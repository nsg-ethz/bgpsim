#!/usr/bin/python

import json
import sys

file_to = sys.argv[1]
file_from = "pos.json"
if len(sys.argv) >= 3:
    file_from = sys.argv[2]

print(f"copy position from {file_from} to {file_to}")

with open(file_to, "r") as fp:
    json_to = json.load(fp)

with open(file_from, "r") as fp:
    json_from = json.load(fp)

json_to["pos"] = json_from["pos"]

with open(file_to, "w") as fp:
    json.dump(json_to, fp)
