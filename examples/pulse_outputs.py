from nscope import LabBench
import time

nscope = LabBench.open_first_available()

nscope.px_turn_on(1)
time.sleep(10)
nscope.px_turn_off(1)

nscope.px_turn_on(2)
time.sleep(10)
nscope.px_turn_off(2)