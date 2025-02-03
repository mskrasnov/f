#!/bin/env python3

import os

j = 1000000
i = 0
while i <= j:
    os.system(f"touch test/{i}.txt")
    i += 1
