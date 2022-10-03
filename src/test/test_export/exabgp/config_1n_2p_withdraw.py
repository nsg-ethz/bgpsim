#!/usr/bin/env python3

import sys
import time


sys.stdout.write("neighbor 10.192.0.1 announce route 128.0.0.0/16 next-hop self as-path [100, 60]\n")
sys.stdout.flush()

while True:
    time.sleep(0)
    sys.stdout.write("neighbor 10.192.0.1 announce route 128.1.0.0/16 next-hop self as-path [100, 40, 10]\n")
    sys.stdout.flush()
    time.sleep(10)
    sys.stdout.write("neighbor 10.192.0.1 withdraw route 128.1.0.0/16\n")
    sys.stdout.flush()
    time.sleep(10)
