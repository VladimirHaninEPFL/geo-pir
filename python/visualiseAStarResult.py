"""
visualise a networkx graph, that has been saved using pickle to disk
first arugment is the path to the pickle file
"""
import networkx as nx
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
import sys
import pickle as pk
import re

def visualiseAStarSearch(G, params, approach, outputPath=None):
    pos = {n: (d["lon"], d["lat"]) for n, d in G.nodes(data=True)}
    number_cached_nodes = len(params["cached_nodes"])

    # Split nodes into layers
    start_end_nodes = [n for n in G if n == params["start_node"] or n == params["end_node"]]
    path_nodes      = [n for n in G if n in params["path"] and n not in start_end_nodes]
    cached_nodes    = [n for n in G if n in params["cached_nodes"] and n not in params["path"] and n not in start_end_nodes]
    other_nodes     = [n for n in G if n not in params["cached_nodes"] and n not in params["path"] and n not in start_end_nodes]

    plt.figure(figsize=(6, 4), dpi=600)
    ax = plt.gca()

    nx.draw(
        G,
        pos=pos,
        node_size=1,
        nodelist=other_nodes,
        node_color="red",
        edgelist=[],
        width=0.5,
        with_labels=False,
        ax=ax,
    )
    nx.draw(
        G,
        pos=pos,
        node_size=1,
        nodelist=cached_nodes,
        node_color="orange",
        edgelist=[],
        width=0.5,
        with_labels=False,
        ax=ax,
    )
    nx.draw(
        G,
        pos=pos,
        node_size=1,
        nodelist=path_nodes,
        node_color="green",
        edgelist=[],
        width=0.5,
        with_labels=False,
        ax=ax,
    )
    nx.draw(
        G,
        pos=pos,
        node_size=1,
        nodelist=start_end_nodes,
        node_color="black",
        edgelist=[],
        width=0.5,
        with_labels=False,
        ax=ax,
    )

    legend_elements = [
        Line2D([0], [0], marker="o", color="w", label="Start & End nodes", markerfacecolor="black", markersize=6),
        Line2D([0], [0], marker="o", color="w", label="Nodes on best path", markerfacecolor="green", markersize=6),
        Line2D([0], [0], marker="o", color="w", label=f"Client cached nodes ({number_cached_nodes})", markerfacecolor="orange", markersize=6),
        Line2D([0], [0], marker="o", color="w", label="Nodes not downloaded", markerfacecolor="red", markersize=6),
    ]
    plt.legend(handles=legend_elements, loc="lower left", prop={"size": 9})
    plt.title(f"Client state after A* using {approach}")
    ax.axis("equal")
    plt.tight_layout()                          # ← remove outer whitespace

    # ax.spines["top"].set_visible(False)
    # ax.spines["right"].set_visible(False)
    # ax.spines["bottom"].set_visible(False)
    # ax.spines["left"].set_visible(False)

    if outputPath is None:
        plt.show()
    else:
        print(f"Image generated to {outputPath} !")
        plt.savefig(outputPath)

def extractAStarResultFromFile(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        text = f.read()
    match = re.search(r"Running A\* from (\d+) to (\d+)", text)
    if not match:
        raise ValueError("Could not find start/end nodes")
    start_node = match.group(1)
    end_node = match.group(2)
    path_match = re.search(r"Path:\s*\[(.*?)\]", text, re.DOTALL)
    if path_match:
        path_str = path_match.group(1)
        path = re.findall(r'"(\d+)"', path_str)
    else:
        path = []
    caced_match = re.search(r"Cached nodes: \s*\[(.*?)\]", text, re.DOTALL)
    if caced_match:
        cahed_str = caced_match.group(1)
        cached_nodes = re.findall(r"(\d+)", cahed_str)
    else:
        cached_nodes = []
    return {
        "start_node": int(start_node),
        "end_node": int(end_node),
        "path": [int(n) for n in path],
        "cached_nodes": set(int(n) for n in cached_nodes)
    }

def main():
    pathResult = sys.argv[1]
    params = extractAStarResultFromFile(pathResult)

    pathPickelFile = sys.argv[2]
    G = pk.load(open(pathPickelFile, "rb"))

    outputPath = sys.argv[3]

    approachName = sys.argv[4]
    visualiseAStarSearch(G, params, approachName, outputPath=outputPath)

if __name__ == "__main__":
    main()