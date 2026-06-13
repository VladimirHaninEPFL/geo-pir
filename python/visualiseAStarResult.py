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


def visualiseAStarSearch(G, params, outputPath=None):

    # color the nodes according to their state
    node_colors = []
    for n in G:
        if n == params["start_node"] or n == params["end_node"]:
            node_colors.append("black")

        elif n in params["path"]:
            node_colors.append("green")

        elif n in params["visited_nodes"]:
            node_colors.append("orange")

        else:
            node_colors.append("red")

    pos = {n: (d["lon"], d["lat"]) for n, d in G.nodes(data=True)}

    plt.figure(figsize=(12, 10), dpi=300)
    nx.draw(
        G,
        pos=pos,
        node_size=1,
        node_color=node_colors,
        arrows=False,
        edge_color="blue",
        with_labels=False,
    )
    legend_elements = [
        Line2D([0], [0], marker="o", color="w", label="Start / End", markerfacecolor="black", markersize=6),
        Line2D([0], [0], marker="o", color="w", label="Best path", markerfacecolor="green", markersize=6),
        Line2D([0], [0], marker="o", color="w", label="Visited", markerfacecolor="orange", markersize=6),
        Line2D([0], [0], marker="o", color="w", label="Not visited", markerfacecolor="red", markersize=6),
    ]
    plt.legend(handles=legend_elements, loc="upper right")
    plt.title("Road network")
    plt.axis("equal")
    plt.tight_layout()
    
    if outputPath is None:
        plt.show()
    else:
        print(f"Image generated to {outputPath} ! ")
        plt.savefig(outputPath)


def extractAStarResultFromFile(file_path):

    with open(file_path, "r", encoding="utf-8") as f:
        text = f.read()

    # Extract start and end nodes
    match = re.search(r"Running A\* from (\d+) to (\d+)", text)
    if not match:
        raise ValueError("Could not find start/end nodes")

    start_node = match.group(1)
    end_node = match.group(2)

    # Extract path array
    path_match = re.search(r"Path:\s*\[(.*?)\]", text, re.DOTALL)
    if path_match:
        path_str = path_match.group(1)
        path = re.findall(r'"(\d+)"', path_str)
    else:
        path = []

    # Extract visited nodes array
    visited_match = re.search(r"Visited nodes: \s*\[(.*?)\]", text, re.DOTALL)
    if visited_match:
        visited_str = visited_match.group(1)
        visited_nodes = re.findall(r"(\d+)", visited_str)
    else:
        visited_nodes = []
    # print(f"Found {len(visited_nodes)} visited nodes")
    # print(f"Visited nodes: {visited_nodes}")

    return {
        "start_node": int(start_node),
        "end_node": int(end_node),
        "path": [int(n) for n in path],
        "visited_nodes": set(int(n) for n in visited_nodes)
    }

def main():

    pathResult = sys.argv[1]
    params = extractAStarResultFromFile(pathResult)

    pathPickelFile = sys.argv[2]
    G = pk.load(open(pathPickelFile, "rb"))

    nameResult = sys.argv[3]
    visualiseAStarSearch(G, params, outputPath=f"astar_result_{nameResult}.png")

if __name__ == "__main__":
    main()