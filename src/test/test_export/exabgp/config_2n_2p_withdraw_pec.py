#!/usr/bin/env python3

import sys
import time


time.sleep(5)

sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 announce route 200.0.1.0/24 next-hop self as-path [100, 60]\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 announce route 200.0.2.0/24 next-hop self as-path [100, 60]\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 announce route 200.0.3.0/24 next-hop self as-path [100, 60]\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 announce route 200.0.4.0/24 next-hop self as-path [100, 60]\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 announce route 200.0.5.0/24 next-hop self as-path [100, 60]\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 announce route 100.0.1.0/24 next-hop self as-path [100, 40, 10]\n")
sys.stdout.flush()
time.sleep(10)
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 withdraw route 200.0.1.0/24\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 withdraw route 200.0.2.0/24\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 withdraw route 200.0.3.0/24\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 withdraw route 200.0.4.0/24\n")
sys.stdout.write("neighbor 10.192.0.1, neighbor 10.192.0.5 withdraw route 200.0.5.0/24\n")
sys.stdout.flush()

while True:
    time.sleep(1)
