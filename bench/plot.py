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
    for network in ["quicknet", "evmnet"]:
        data = defaultdict(list)
        with open(f"results/{network}_verify_1000_evm.dat") as f:
            for r in csv.DictReader(f):
                data[r["algorithm"]].append(int(r["exec_gas"]))
        fig, ax = plt.subplots()

        handles, labels = [], []
        for alg in data:
            arr = np.array(data[alg], dtype=np.uint64)
            vp = ax.violinplot(arr, showmeans=True, showextrema=False)
            handles.append(vp["bodies"][0])  # https://stackoverflow.com/a/59951248
            labels.append(f"{alg} (mean={arr.mean()})")
        plt.legend(handles, labels)
        plt.savefig(f"plots/{network}_verify_1000_evm.png")


if __name__ == "__main__":
    main()
