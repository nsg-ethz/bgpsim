#!/usr/bin/env python3

import sys
import time


time.sleep(5)

sys.stdout.write("neighbor 10.192.0.1 announce route 100.0.0.0/24 next-hop self as-path [100]\n")
sys.stdout.flush()

while True:
    time.sleep(1)
