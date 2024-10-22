from nlabapi import LabBench
import time

nlab = LabBench.open_first_available()

while True:
    time.sleep(0.5)
    power_status = nlab.power_status()

    print(f"\n{power_status}")
    print(f"State: {power_status.state}, Usage: {power_status.usage:1.3f} Watts")
