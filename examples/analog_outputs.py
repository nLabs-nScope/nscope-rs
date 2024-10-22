from nlabapi import LabBench
import time

nlab = LabBench.open_first_available()

nlab.ax_turn_on(1)
time.sleep(10)
nlab.ax_turn_off(1)

nlab.ax_turn_on(2)
time.sleep(10)
nlab.ax_turn_off(2)