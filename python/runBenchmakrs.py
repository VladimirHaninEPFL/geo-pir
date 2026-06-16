"""
Visualise the generated journeys stored in a pickle file.

Update the hard-coded pickle paths below if you want to inspect a different
graph or journeys dump.
"""

from pathlib import Path
import pickle as pk
import sys
import subprocess
import time

def startBenchmarks() -> None:

    # clean output directory
    subprocess.run(["rm ./output/*"], shell=True)

    # compile the go code of singlepass !
    # subprocess.run(["go", "build", "-o", "singlepass-client", "./cmd/singlepass_demo_node/client/client.go"], cwd="./../SinglePass")
    # subprocess.run(["go", "build", "-o", "singlepass-server", "./cmd/singlepass_demo_node/server/server.go"], cwd="./../SinglePass")

    countries = ["Switzerland", "France"]
    architectures = ["Spiral", "SinglePass"]
    approaches = ["node0", "node1", "node2", "node3", "block0.1", "block0.25", "block0.5", "block1"]
    
    journeys = {}
    for country in countries:
        journeys[country] = pk.load(open(f"./data/{country}-journeys.pickle", "rb"))
    
    for country in countries:
        for archi in architectures:
            for approach in approaches:

                journeysThisCountry = journeys[country]

                for distance, start_ends in journeysThisCountry.items():

                    if country == "France":
                        subprocess.run(["sbatch", "run_distance_large.sh", country, archi, approach, str(distance), pairs_to_bash_arg(start_ends)], cwd="./batch/")
                    else:
                        subprocess.run(["sbatch", "run_distance_small.sh", country, archi, approach, str(distance), pairs_to_bash_arg(start_ends)], cwd="./batch/")
                    time.sleep(1)


def pairs_to_bash_arg(pairs):
    return " ".join(f"{k}:{v}" for k, v in pairs)

def to_bash_array(arr):
    return "(" + " ".join(f'"{s}"' for s in arr) + ")"

if __name__ == "__main__":
    startBenchmarks()
