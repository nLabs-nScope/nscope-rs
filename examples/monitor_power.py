from nscope import LabBench
import time

nscope = LabBench.open_first_available()

while True:
    time.sleep(0.5)
    power_status = nscope.power_status()

    print(f"\n{power_status}")
    print(f"State: {power_status.state}, Usage: {power_status.usage:1.3f} Watts")
