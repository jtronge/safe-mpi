"""Benchmark graphing script."""
import argparse
import os
import matplotlib.pyplot as plt
import json


fmts = {
    'bincode': '-b',
    'iovec': '--g',
    'rsmpi': '-.r',
    'flat': ':k',
}


def graph_latency(config, tests):
    """Graph a latency result."""
    fig, ax = plt.subplots()
    ax.set_title(f'latency: {config}')
    ax.set_xlabel('size (bytes)')
    ax.set_ylabel('latency (μs)')
    ax.grid(visible=True)
    for test_name, results in tests.items():
        sizes = sorted(int(size) for size in results)
        sizes = [str(size) for size in sizes]
        lats = [results[size] for size in sizes]
        ax.plot(sizes, lats, fmts[test_name], label=test_name)
    ax.legend()
    plt.show()


def graph_bw(config, tests):
    """Graph a bandwidth result."""
    fig, ax = plt.subplots()
    ax.set_title(f'bandwidth: {config}')
    ax.set_xlabel('size (bytes)')
    ax.set_ylabel('bandwidth (MB/s)')
    ax.grid(visible=True)
    for test_name, results in tests.items():
        sizes = sorted(int(size) for size in results)
        sizes = [str(size) for size in sizes]
        bw = [results[size] for size in sizes]
        ax.plot(sizes, bw, fmts[test_name], label=test_name)
    ax.legend()
    plt.show()


parser = argparse.ArgumentParser(description='graph benchmark results')
parser.add_argument('-o', '--output', help='JSON benchmark output', required=True)
args = parser.parse_args()

graphers = {
    'latency': graph_latency,
    'bw': graph_bw,
}

with open(args.output) as fp:
    results = json.load(fp)

plt.style.use('./poster.mplstyle')
for benchmark, configs in results.items():
    for config, tests in configs.items():
        graphers[benchmark](config, tests)
