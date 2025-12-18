# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "matplotlib",
# ]
# ///

import csv

from collections import defaultdict

import matplotlib.pyplot as plt
import numpy as np


def main() -> None:
    data = defaultdict(list)
    with open("results/quicknet_verify_1000_evm.dat") as f:
        for r in csv.DictReader(f):
            data[r['algorithm']].append(int(r['exec_gas']) + int(r['data_gas']))
    for alg, data in data.items():
        data = np.array(data, dtype=np.uint64)
        fig, ax = plt.subplots()
        ax.set_title(f"quicknet {alg} (mean={data.mean()})")
        vp = ax.violinplot(data, showmeans=True, showextrema=False)
        plt.savefig(f"plots/quicknet_{alg}.png")


if __name__ == "__main__":
    main()
