"""
In this file we will explore how much resources each approach takes for a
given journey query.

This variant visualises the full sample distribution more faithfully by
plotting:
- the mean as a line
- a percentile band around the mean
- the raw samples as faint points
"""
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter
import numpy as np
import pickle as pk
import os
import re


countries = ["Switzerland", "France"]
architectures = ["Spiral", "SinglePass"]
approaches = ["node0", "node1", "node2", "node3", "block0.1", "block0.25", "block0.5", "block1"]

plt.style.use('default')


def format_y_value(value):
    if value >= 1000:
        return f"{int(round(value)):,}"
    return f"{value:.1f}" if value < 100 else f"{value:.0f}"

def format_bytes(value, _pos=None):
    """Format byte values using readable units on a log axis."""
    if value == 0:
        return "0 B"

    abs_value = abs(value)
    units = [
        ("GB", 1_000_000_000),
        ("MB", 1_000_000),
        ("KB", 1_000),
    ]

    for unit, factor in units:
        if abs_value >= factor:
            scaled = value / factor
            if abs(scaled) >= 100:
                return f"{scaled:.0f}{unit}"
            if abs(scaled) >= 10:
                return f"{scaled:.1f}{unit}"
            return f"{scaled:.1f}{unit}"

    return f"{value:.0f}B"

def format_time(value, _pos=None):
    """Format seconds using human-friendly units on a log axis."""
    if value == 0:
        return "0 s"

    abs_value = abs(value)
    units = [
        ("h", 60 * 60),
        ("min", 60),
        ("s", 1),
        ("ms", 1e-3),
        ("us", 1e-6),
    ]

    for unit, factor in units:
        if abs_value >= factor:
            scaled = value / factor
            if abs(scaled) >= 100:
                return f"{scaled:.0f}{unit}"
            if abs(scaled) >= 10:
                return f"{scaled:.1f}{unit}"
            return f"{scaled:.1f}{unit}"

    return f"{value:.1f}s"

def annotate_last_point(ax, x_values, y_values, text, color, y_offset=0):
    ax.annotate(
        text,
        xy=(x_values[-1], y_values[-1]),
        xytext=(8, y_offset),
        textcoords="offset points",
        color=color,
        fontsize=9,
        va="center",
        ha="left",
        clip_on=False,
    )

def plot_comp(ax, x_values, sample_sets1, sample_sets2, label, color, linestyle=None):

    sample_arrays1 = [np.asarray(results, dtype=float) for results in sample_sets1]
    sample_arrays2 = [np.asarray(results, dtype=float) for results in sample_sets2]

    totals = np.asarray([np.mean(results) for results in sample_arrays1], dtype=float)
    parts = np.asarray([np.mean(results) for results in sample_arrays2], dtype=float)

    percentages = parts / totals

    ax.plot(x_values, percentages, color=color, linewidth=2.5, linestyle=linestyle, label=label)
    ax.scatter(x_values, percentages, color=color, linewidth=2.5)

def plot_mean_with_percentile_band(ax, x_values, sample_sets1, label, color, linestyle=None):
    """Plot the mean, a central percentile band, and raw samples."""

    # result = [ (xvalue, sample) for (xvalue, sample) in zip(x_values, sample_sets1) if len(sample) > 0]
    # x_values = list(map(lambda x: x[0], result))
    # sample_sets1 = list(map(lambda x: x[1], result))

    sample_arrays = [np.asarray(results, dtype=float) for results in sample_sets1]

    lower = np.asarray([np.percentile(results, 5) for results in sample_arrays], dtype=float)
    upper = np.asarray([np.percentile(results, 95) for results in sample_arrays], dtype=float)
    ax.fill_between(x_values, lower, upper, color=color, alpha=0.18)

    means = np.asarray([np.mean(results) for results in sample_arrays], dtype=float)
    ax.plot(x_values, means, color=color, linewidth=2.5, linestyle=linestyle, label=label)
    ax.scatter(x_values, means, color=color, linewidth=2.5)

    return x_values, means

def plot_metric(
    data,
    ylabel,
    title,
    output_path,
    add_naive=None,
    ylim=None,
    lastPoint=False,
    last_point_formatter=None,
    y_formatter=None,
    locLegend="lower right",
):

    fig, ax = plt.subplots(figsize=(6, 4))
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.set_axisbelow(True)
    ax.grid(True, which="major", axis="both", alpha=0.18, linewidth=0.8)
    ax.grid(True, which="minor", axis="both", alpha=0.08, linewidth=0.5)
    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]

    for idx, (approach, metric_per_distance) in enumerate(data.items()):
        metric_per_distance = dict(sorted(metric_per_distance.items()))

        x_values = np.asarray(list(metric_per_distance.keys()), dtype=float) / 1000.0
        sample_sets = list(metric_per_distance.values())
        color = colors[idx % len(colors)]
        y_offset = 0 #-10 if idx % 2 == 0 else 10

        mean_x_values, mean_values = plot_mean_with_percentile_band(
            ax,
            x_values,
            sample_sets,
            label=approach,
            color=color,
        )

        if lastPoint:
            point_formatter = last_point_formatter or format_y_value
            annotate_last_point(
                ax,
                mean_x_values,
                mean_values,
                point_formatter(mean_values[-1]),
                color,
                y_offset=y_offset,
            )

    if add_naive is not None:
        x_values, y_values, label = add_naive

        x_values = np.asarray(list(x_values), dtype=float) / 1000.0
        sample_sets = y_values
        y_offset = 0 #-10 if idx % 2 == 0 else 10

        mean_x_values, mean_values = plot_mean_with_percentile_band(
            ax,
            x_values,
            sample_sets,
            label=label,
            color="black",
            linestyle="--"
        )

        if lastPoint:
            point_formatter = last_point_formatter or format_y_value
            annotate_last_point(
                ax,
                mean_x_values,
                mean_values,
                point_formatter(mean_values[-1]),
                color,
                y_offset=y_offset,
            )

    if ylim is not None:
        ax.set_ylim(bottom=0, top=ylim)

    ax.set_yscale("log")
    if y_formatter is not None:
        ax.yaxis.set_major_formatter(FuncFormatter(y_formatter))
    ax.set_xlabel("Journey length (km)")
    ax.set_ylabel(ylabel)
    ax.set_title(title)
    ax.margins(x=0.08)
    ax.legend(loc=locLegend, prop={"size": 9},  ncol=2)
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)

def plot_comp_metric(
    data1,
    data2,
    ylabel,
    title,
    output_path,
    add_naive=None,
    ylim=None,
    lastPoint=False,
    last_point_formatter=None,
    y_formatter=None,
    locLegend="lower right",
):

    fig, ax = plt.subplots(figsize=(6, 4))
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.set_axisbelow(True)
    ax.grid(True, which="major", axis="both", alpha=0.18, linewidth=0.8)
    ax.grid(True, which="minor", axis="both", alpha=0.08, linewidth=0.5)
    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]

    for idx, approach in enumerate(data1.keys()):
        metric_per_distance1 = dict(sorted(data1[approach].items()))
        metric_per_distance2 = dict(sorted(data2[approach].items()))

        x_values = np.asarray(list(metric_per_distance1.keys()), dtype=float) / 1000.0
        sample_sets1 = list(metric_per_distance1.values())
        sample_sets2 = list(metric_per_distance2.values())
        color = colors[idx % len(colors)]

        plot_comp(
            ax,
            x_values,
            sample_sets1,
            sample_sets2,
            label=approach,
            color=color,
        )

        # if lastPoint:
        #     point_formatter = last_point_formatter or format_y_value
        #     annotate_last_point(
        #         ax,
        #         mean_x_values,
        #         mean_values,
        #         point_formatter(mean_values[-1]),
        #         color,
        #         y_offset=y_offset,
        #     )


    ax.set_ylim(bottom=0, top=1)

    if y_formatter is not None:
        ax.yaxis.set_major_formatter(FuncFormatter(y_formatter))
    ax.set_xlabel("Journey length (km)")
    ax.set_ylabel(ylabel)
    ax.set_title(title)
    ax.margins(x=0.08)
    ax.legend(loc=locLegend, prop={"size": 9},  ncol=2)
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)

def parse_filename(filename):
    """
    Parse filename to extract country, architecture, and approach.
    Format: countryname-architecturename-approachname.txt
    """
    basename = os.path.basename(filename)
    name_without_ext = os.path.splitext(basename)[0]
    parts = name_without_ext.split('-')
    
    if len(parts) >= 3:
        country = parts[0]
        architecture = parts[1]
        approach = '-'.join(parts[2:])  # Handle approach names with hyphens
        return country, architecture, approach
    
    return None, None, None

def parse_output_file(filepath):
    """
    Parse an output file and extract distance, timing, and bytes information.
    Returns a list of dicts with keys: distance, cost, total_time, server_time, search_time, bytes_received
    """

    time_per_distance = {}
    server_time_per_distance = {}
    bytes_per_distance = {}

    current_distance = None
    
    with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:

            # Extract distance from comment line
            distance_match = re.search(r'-- running journeys of distance (\d+)\.\.\.', line)
            if distance_match:
                current_distance = int(distance_match.group(1))

                time_per_distance[current_distance] = []
                bytes_per_distance[current_distance] = []
                server_time_per_distance[current_distance] = []

            # Extract timing information
            match = re.search(r'A\* total elapsed time:\s+([\d.]+)\s+s', line)
            if match:
                time_per_distance[current_distance].append(float(match.group(1)))
            
            # extract server time
            match = re.search(r'  server queries time:\s+([\d.]+)\s+s', line)
            if match:
                server_time_per_distance[current_distance].append(float(match.group(1)))

            # Extract bytes received
            match = re.search(r'Server bytes received:\s+([\d.]+)\s+bytes', line)
            if match:
                bytes_per_distance[current_distance].append(float(match.group(1)))
    
    
    return time_per_distance, server_time_per_distance, bytes_per_distance

def getQueryTimes(countryName, archi):
    """
    This returns a dictionary with this format:
    {
        approach : {
            distance: metric values (array)
        }
    }
    """
    
    # Group results by (country, architecture)
    time_per_approach = {}
    server_time_per_approach = {}
    byte_per_approach = {}
    for approach in approaches:

        filepath = f"./output/{countryName}-{archi}-{approach}.txt"
        timePerDistance, serverTimePerDistance, bytePerDistance = parse_output_file(filepath)

        time_per_approach[approach] = timePerDistance
        server_time_per_approach[approach] = serverTimePerDistance
        byte_per_approach[approach] = bytePerDistance
            
    filepath = f"./output/{countryName}-Naive-node0.txt"
    timePerDistanceNaive, serverTimePerDistanceNaive, bytePerDistanceNaive = parse_output_file(filepath)

    return time_per_approach, server_time_per_approach, byte_per_approach, timePerDistanceNaive, serverTimePerDistanceNaive, bytePerDistanceNaive

def main():

    for countryName in countries:
        for archi in architectures:

            queryTimes, queryServerTimes, queryBytes, timesNaive, serverTimesNaive, dataNaive = getQueryTimes(countryName, archi)

            # visualise navigational query duration
            plot_metric(
                queryTimes,
                ylabel="Query time",
                title=f"Navigational query duration for {countryName} using {archi}",
                output_path=f"./times-{countryName}-{archi}.png",
                y_formatter=format_time,
                add_naive=(
                    timesNaive.keys(),
                    timesNaive.values(),
                    f"naive",
                ),
                locLegend="upper left" if countryName == "France" and archi == "SinglePass" else None
            )

            plot_comp_metric(
                queryTimes,
                queryServerTimes,
                ylabel="Proportion",
                title=f"Proportion of time spent on PIR for {countryName} using {archi}",
                output_path=f"./servertimes-{countryName}-{archi}.png",
            )

            # visualise navigational query bandwdith
            plot_metric(
                queryBytes,
                ylabel="Query bandwidth",
                title=f"Navigational query bandwidth for {countryName} using {archi}",
                output_path=f"./data-{countryName}-{archi}.png",
                y_formatter=format_bytes,
                add_naive=(
                    dataNaive.keys(),
                    dataNaive.values(),
                    f"full db ({ format_bytes(list(dataNaive.values())[0][0]) })",
                )
            )

    print("Done !")

main()
