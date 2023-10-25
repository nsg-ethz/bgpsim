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

assert set(json_to["pos"].keys()) == set(json_from["pos"].keys())

idx_to_name = {}
name_to_idx = {}
translation = {}
for idx, name, _ in json_from["config_nodes_routes"][1]:
    idx_to_name[idx] = name
for idx, name, _ in json_to["config_nodes_routes"][1]:
    name_to_idx[name] = idx
for idx, name in idx_to_name.items():
    translation[str(idx)] = str(name_to_idx[name])

for from_idx, to_idx in translation.items():
    json_to["pos"][to_idx] = json_from["pos"][from_idx]

with open(file_to, "w") as fp:
    json.dump(json_to, fp)
