"""
Visualise the generated journeys stored in a pickle file.

Update the hard-coded pickle paths below if you want to inspect a different
graph or journeys dump.
"""

from pathlib import Path
import pickle as pk
import json
import subprocess
import time

def startBenchmarks() -> None:

    # clean output directory
    # subprocess.run(["rm ./output/*"], shell=True)

    # compile the go code of singlepass !
    subprocess.run(["go", "build", "-o", "singlepass-client", "./cmd/singlepass_demo_node/client/client.go"], cwd="./../SinglePass")
    subprocess.run(["go", "build", "-o", "singlepass-server", "./cmd/singlepass_demo_node/server/server.go"], cwd="./../SinglePass")

    # compile this code
    subprocess.run(["cargo", "build", "--release", "--bin", "geo_server"])
    subprocess.run(["cargo", "build", "--release", "--bin", "geo_client"])

    countries = ["Switzerland"]#, "France"]
    architectures = ["SinglePass"] #, "Spiral"]
    approaches = ["node0", "node3", "block0.1", "block1" ] #, "block0.1", "block0.25", "block0.5", "block1"]
    
    journeys = {}
    for country in countries:
        journeys[country] = pk.load(open(f"./data/{country}-journeys.pickle", "rb"))
        # journeys[country] = {10000: [(2324923182, 277997396)] }
    
    for country in countries:
        for archi in architectures:
            for approach in approaches:

                if country == "France":
                    script_to_run = "run_distance_large.sh"
                else:
                    script_to_run = "run_distance_small.sh"

                if approach == "node3":
                    script_to_run = "run_distance_verylarge.sh"

                journeysJson = json.dumps(journeys[country], sort_keys=True)
                subprocess.run(["sbatch", script_to_run, country, archi, approach, journeysJson], cwd="./batch/")

    # # naive approaches
    # for country in countries:
    #     journeysJson = json.dumps(journeys[country], sort_keys=True)
    #     subprocess.run(["sbatch", "run_distance_small.sh", country, "Naive", "node0", journeysJson], cwd="./batch/")


# def pairs_to_bash_arg(pairs):
#     return "|".join(f"{a}:{b}" for a, b in pairs)

# def to_bash_array(arr):
#     return "(" + " ".join(f'"{s}"' for s in arr) + ")"

if __name__ == "__main__":
    startBenchmarks()
