"""Benchmark script for safe-mpi code."""
import argparse
import json
import os
import subprocess
import yaml


def parse_latency(fname):
    """Parse a latency example."""
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
    """Parse a bandwidth example."""
    results = []
    with open(fname) as fp:
        for line in fp:
            line = line.strip()
            if not line:
                break
            size, bw = line.split(' ')
            results.append((int(size), float(bw)))
    return results


def finish_result(results):
    """Process the result for later."""
    size = [sz for sz, _ in results[0]]
    data = [[val for _, val in res] for res in results]
    return {
        'size': size,
        'data': data,
    }


def canonical_name(name):
    """Strip a benchmark name down to its canonical form."""
    # Take the first word before a dash
    return name.split('-')[0]


# Parsers will parse output from various benchmarks and return results
parsers = {
    'latency': parse_latency,
    'bw': parse_bw,
}

# Finishers take a list of multiple outputs from the same benchmarks and
# produce a result that can be more easily parsed
finishers = {
    'latency': finish_result,
    'bw': finish_result,
}

# Temporary output location for sbatch scripts
output = 'tmp.out'

parser = argparse.ArgumentParser(description='benchmark run script for Slurm')
parser.add_argument('-e', '--env-file', help='environment file to load',
                    required=True)
parser.add_argument('-o', '--output', help='JSON result output file',
                    required=True)
parser.add_argument('-r', '--run-count', help='number of runs to do',
                    required=True, type=int)
parser.add_argument('-c', '--config', help='benchmark config', required=True)
args = parser.parse_args()

with open(args.config) as fp:
    config = yaml.load(fp, Loader=yaml.CLoader)
benchmarks = config['benchmarks']
scripts = config['scripts']

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
            print(f'**test {test_name}**')
            # Get the script and environment info
            script = scripts[benchmark][test_name]
            sbatch_script = script['script']
            sbatch_env = script['env']
            # Set up the script environment
            env = dict(os.environ)
            env.update({
                'SAFE_MPI_ENV_FILE': args.env_file,
                'SAFE_MPI_CONFIG': config,
            })
            env.update(sbatch_env)
            results = []
            # Run however many times specified by the args
            for run in range(args.run_count):
                # Run the job until completion
                subprocess.run(['sbatch', '-W', '-o', output, sbatch_script],
                               env=env)
                # Parse and save the results
                parser = parsers[canonical_name(benchmark)]
                results.append(parser(output))
            # Now combine the runs
            finished_result = finishers[canonical_name(benchmark)](results)
            config_result[test_name] = finished_result

        config_prefix = os.path.basename(config)
        config_prefix = config_prefix.split('.')[0]
        benchmark_result[config_prefix] = config_result

    all_results[benchmark] = benchmark_result

with open(args.output, 'w') as fp:
    json.dump(all_results, fp, indent=4)
