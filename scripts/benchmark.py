import argparse
import os
import subprocess
import socket
import time
import yaml


parser = argparse.ArgumentParser(description='benchmark runner')
parser.add_argument('-o', '--output', help='script and benchmark output path', required=True)
parser.add_argument('-n', '--node', help='node to use as server', required=True)
parser.add_argument('-e', '--env', help='bash environment to load', required=True)
args = parser.parse_args()


def start_job(cmd, script, output, node=None, node_count=1):
    """Build a script for slurm and start the job."""
    with open(script, 'w') as fp:
        print('#!/bin/sh', file=fp)
        print(f'#SBATCH -o {output}', file=fp)
        print(f'#SBATCH -N {node_count}', file=fp)
        if node is not None:
            print(f'#SBATCH -w {node}', file=fp)
        print(f'source {args.env}', file=fp)
        print(' '.join(cmd), file=fp)
    return subprocess.Popen(['sbatch', '-W', script])


def serde_command(kind, server_node, config, server=False):
    """Build a command for the serde benchmark."""
    ip = socket.gethostbyname(server_node)
    cmd = ['./target/release/latency_serde', ip, '-p', '7776', '-k', kind,
           '-c', config]
    if server:
        cmd.append('-s')
    return cmd


def serde_test(kind):
    """Return a serde-based test (kind is one of message-pack, postcard, bincode)."""
    def test(run, config, prefix, node):
        """Run the serde tests."""
        server = start_job(
            cmd=serde_command(kind=kind, server_node=node, server=True,
                              config=config),
            script=f'{prefix}_server.sh',
            output=f'{prefix}_server.out',
            node=node,
        )
        time.sleep(2)
        client = start_job(
            cmd=serde_command(kind=kind, server_node=node,
                              config=config),
            script=f'{prefix}_client.sh',
            output=f'{prefix}_client.out',
        )
        client.wait()
        server.wait()
    return test


def iovec_test(run, config, prefix, node):
    """Run the iovec test."""
    ip = socket.gethostbyname(node)
    port = 1347
    cmd = ['target/release/latency_iovec', ip, '-p', str(port), '-c', config]
    server = start_job(
        cmd=(cmd + ['-s']),
        script=f'{prefix}_server.sh',
        output=f'{prefix}_server.out',
        node=node,
    )
    time.sleep(2)
    client = start_job(
        cmd=cmd,
        script=f'{prefix}_client.sh',
        output=f'{prefix}_client.out',
    )
    client.wait()
    server.wait()


def rsmpi_test(run, config, prefix, node):
    """Run the rsmpi test."""
    cmd = ['mpirun', '-np', '2', '-N', '1', './target/release/latency_rsmpi', '-c', config]
    start_job(cmd=cmd, script=f'{prefix}.sh', output=f'{prefix}.out', node_count=2).wait()


# client_node = 'er02'
configs = {
    './inputs/simple.yaml': {
        'message-pack': serde_test('message-pack'),
        'postcard': serde_test('postcard'),
        'bincode': serde_test('bincode'),
        'iovec': iovec_test,
        'rsmpi': rsmpi_test,
    },
    './inputs/complex-noncompound.yaml': {
        'message-pack': serde_test('message-pack'),
        'postcard': serde_test('postcard'),
        'bincode': serde_test('bincode'),
        'iovec': iovec_test,
        'rsmpi': rsmpi_test,
    },
    './inputs/complex-compound.yaml': {
        'message-pack': serde_test('message-pack'),
        'postcard': serde_test('postcard'),
        'bincode': serde_test('bincode'),
        'iovec': iovec_test,
        # rsmpi does not support complex-compound datatypes
    },
}
run_count = 4

for config, tests in configs.items():
    print('testing', config)
    config_prefix = os.path.basename(config)
    config_prefix = config_prefix.split('.')[0]
    for test_name, test in tests.items():
        print('running test', test_name)
        for run in range(run_count):
            print('test run', run)
            prefix = os.path.join(args.output, f'{config_prefix}_{test_name}_{run}')
            test(run, config, prefix, args.node)
            time.sleep(2)
