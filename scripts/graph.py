"""Benchmark graphing script."""
import argparse
import os
import matplotlib.pyplot as plt
import numpy as np
import json


fmts = {
    'bincode': '-b',
    'iovec': '--g',
    'rsmpi': '-.r',
    'flat': ':k',
}


def show_change(sizes, results):
    """Show percentage change in results from the rsmpi baseline."""
    if 'rsmpi' in results:
        print('======================================================')
        for test_name in results:
            if test_name == 'rsmpi':
                continue
            change = 100 * (results[test_name] - results['rsmpi']) / results['rsmpi']
            print(test_name)
            for size, percent in zip(sizes, change):
                print(f'==> {size}: {percent:.2f}%')


def compute_error(results):
    """Compute the average and error."""
    result = np.average(results, 0)
    max_result = np.amax(results, axis=0)
    min_result = np.amin(results, axis=0)
    error = [result - min_result, max_result - result]
    return result, error


def graph_latency(title, tests):
    """Graph a latency result."""
    fig, ax = plt.subplots()
    ax.set_title(f'latency: {title}')
    ax.set_xlabel('size (bytes)')
    ax.set_ylabel('latency (μs)')
    ax.grid(visible=True)
    lat_results = {}
    for test_name, results in tests.items():
        sizes = sorted(int(size) for size in results['size'])
        sizes = [str(size) for size in sizes]
        print(results['data'])
        all_lats = np.array(results['data'])
        lats, error = compute_error(all_lats)
        lat_results[test_name] = lats
        ax.errorbar(sizes, lats, yerr=error, fmt=fmts[test_name], label=test_name)
    show_change(sizes, lat_results)
    ax.legend()
    plt.show()


def graph_bw(title, tests):
    """Graph a bandwidth result."""
    fig, ax = plt.subplots()
    ax.set_title(f'bandwidth: {title}')
    ax.set_xlabel('size (bytes)')
    ax.set_ylabel('bandwidth (MB/s)')
    ax.grid(visible=True)
    bw_results = {}
    for test_name, results in tests.items():
        sizes = sorted(int(size) for size in results['size'])
        sizes = [str(size) for size in sizes]
        all_bw = np.array(results['data'])
        bw, error = compute_error(all_bw)
        bw_results[test_name] = bw
        ax.errorbar(sizes, bw, yerr=error, fmt=fmts[test_name], label=test_name)
    show_change(sizes, bw_results)
    ax.legend()
    plt.show()


parser = argparse.ArgumentParser(description='graph benchmark results')
parser.add_argument('-o', '--output', help='JSON benchmark output', required=True)
parser.add_argument('-t', '--title', help='graph title (uses default if left unspecified)')
args = parser.parse_args()

graphers = {
    'latency': graph_latency,
    'bw': graph_bw,
}

with open(args.output) as fp:
    results = json.load(fp)

plt.rcParams.update({
    'lines.linewidth': 2,
})
for benchmark, configs in results.items():
    for config, tests in configs.items():
        graphers[benchmark](config if args.title is None else args.title, tests)
