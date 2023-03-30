import argparse
import json
import os
import subprocess


def parse_latency(fname):
    """Parse a latency example and return the results."""
    results = []
    with open(fname) as fp:
        for line in fp:
            line = line.strip()
            if not line:
                break
            size, lat = line.split(' ')
            results.append((int(size), float(lat)))
    return results


def parse_bw(fname):
    """Parse a bandwidth example and return the results."""
    results = []
    with open(fname) as fp:
        for line in fp:
            line = line.strip()
            if not line:
                break
            size, bw = line.split(' ')
            results.append((int(size), float(bw)))
    return results


def combine_latency(results):
    """Combine and average multiple latency runs."""
    data = {size: lat for size, lat in results[0]}
    for result in results[1:]:
        for size, lat in result:
            data[size] += lat
    return {size: total / len(results) for size, total in data.items()}


def combine_bw(results):
    """Combine and average multiple bandwidth runs."""
    data = {size: bw for size, bw in results[0]}
    for result in results[1:]:
        for size, bw in result:
            data[size] += bw
    return {size: total / len(results) for size, total in data.items()}


parsers = {
    'latency': parse_latency,
    'bw': parse_bw,
}

combiners = {
    'latency': combine_latency,
    'bw': combine_bw,
}

sbatch_scripts = {
    'latency': {
        # 'message-pack'
        'flat': './scripts/latency_flat.sh',
        'bincode': './scripts/latency_bincode.sh',
        'iovec': './scripts/latency_iovec.sh',
        'rsmpi': './scripts/latency_rsmpi.sh',
    },
    'bw': {
        'bincode': './scripts/bw_bincode.sh',
        'iovec': './scripts/bw_iovec.sh',
        'rsmpi': './scripts/bw_rsmpi.sh',
        'flat': './scripts/bw_flat.sh',
    }
}

benchmarks = {
    'latency': {
        './params/latency/simple.yaml': [
            #'message-pack',
            #'postcard',
            'flat',
            'bincode',
            'iovec',
            'rsmpi',
        ],
        './params/latency/complex-noncompound.yaml': [
            #'message-pack',
            #'postcard',
            'flat',
            'bincode',
            'iovec',
            'rsmpi',
        ],
        './params/latency/complex-compound.yaml': [
            #'message-pack',
            #'postcard',
            'bincode',
            'iovec',
            # rsmpi does not support complex-compound datatypes
        ],
    },
    'bw': {
        './params/bw/complex-noncompound.yaml': [
            'iovec',
            'bincode',
            'flat',
            'rsmpi',
        ],
        './params/bw/complex-compound.yaml': [
            'iovec',
            'bincode',
        ],
    },
}
output = 'tmp.out'

parser = argparse.ArgumentParser(description='benchmark run script for Slurm')
parser.add_argument('-e', '--env-file', help='environment file to load',
                    required=True)
parser.add_argument('-o', '--output', help='JSON result output file',
                    required=True)
parser.add_argument('-r', '--run-count', help='number of runs to do',
                    required=True, type=int)
args = parser.parse_args()

all_results = {}
for benchmark, configs in benchmarks.items():
    print('##################################')
    print('running benchmark', benchmark)
    benchmark_result = {}
    for config, tests in configs.items():
        config_result = {}
        print('----------------------------------')
        print('testing config', config)
        for test_name in tests:
            sbatch_script = sbatch_scripts[benchmark][test_name]
            results = []
            for run in range(args.run_count):
                env = dict(os.environ)
                env.update({
                    'SAFE_MPI_ENV_FILE': args.env_file,
                    'SAFE_MPI_CONFIG': config,
                })
                # Run the job until completion
                subprocess.run(['sbatch', '-W', '-o', output, sbatch_script],
                               env=env)
                # Parse and save the results
                parser = parsers[benchmark]
                results.append(parser(output))
            # Now combine the runs and add it to the final results
            combined_result = combiners[benchmark](results)
            config_result[test_name] = combined_result
        config_prefix = os.path.basename(config)
        config_prefix = config_prefix.split('.')[0]
        benchmark_result[config_prefix] = config_result
    all_results[benchmark] = benchmark_result

with open(args.output, 'w') as fp:
    json.dump(all_results, fp, indent=4)
