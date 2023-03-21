"""Benchmark graphing script."""
import argparse
import os
import matplotlib.pyplot as plt


def average_latency_results(outputs):
    """Average all results in the output files."""
    # Read and parse results
    results = []
    for output in outputs:
        result = []
        with open(output) as fp:
            for line in fp:
                size, lat = line.split()
                result.append((int(size), float(lat)))
        results.append(result)
    # Calculate averages
    sizes = [size for size, _ in results[0]]
    latencies = [0.0 for _ in range(len(sizes))]
    for result in results:
        for i, (_, lat) in enumerate(result):
            latencies[i] += lat
    latencies = [lat / len(results) for lat in latencies]
    return sizes, latencies


parser = argparse.ArgumentParser(description='graph benchmark results')
parser.add_argument('-o', '--output', help='benchmark output path', required=True)
args = parser.parse_args()

configs = ['simple', 'complex-noncompound', 'complex-compound']
kinds = ['postcard', 'message-pack', 'bincode', 'iovec']
fmts = ['--', ':', '-', '-.']
runs = 4

plt.style.use('fivethirtyeight')
fig, axs = plt.subplots(1, 3)
for config, ax in zip(configs, axs):
    ax.set_title(config)
    ax.set_xlabel('size (bytes)')
    ax.set_ylabel('latency (μs)')
    for kind, fmt in zip(kinds, fmts):
        sizes, latencies = average_latency_results(
            [os.path.join(args.output, f'{config}_{kind}_{run}_server.out')
             for run in range(runs)]
        )
        sizes = [str(size) for size in sizes]
        ax.plot(sizes, latencies, fmt, label=kind)
        # print(results)
    ax.legend()
plt.show()
