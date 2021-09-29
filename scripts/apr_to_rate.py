#!/usr/bin/env python3

import sys
from decimal import *

getcontext().prec = 60

n = 31536000000
input_apr = Decimal(sys.argv[1])
print("Input APR: %s%%" % (input_apr,))
print("n: %d" % (n,))

res = (((input_apr / Decimal(100)) + Decimal(1)) ** (Decimal(1) / Decimal(n)))
print("R: %s" % (res,))
print("R ** n: %s" % (res ** n,))

round_res = round(res * (Decimal(10) ** Decimal(27)))
print("rate: %s" % (round_res, ))
