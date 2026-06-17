"""
In this file we will explore how much resources each approach takes for a
given journey query.

This variant visualises the full sample distribution more faithfully by
plotting:
- the mean as a line
- a percentile band around the mean
- the raw samples as faint points
"""
import pickle as pk
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter
import numpy as np
import sys


countryName = sys.argv[1]
architectures = ["Spiral", "SinglePass"]

resourceUse = pk.load(open(sys.argv[2], "rb"))

plt.style.use('default')


def format_y_value(value):
    if value >= 1000:
        return f"{int(round(value)):,}"
    return f"{value:.2f}" if value < 100 else f"{value:.0f}"

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
                return f"{scaled:.0f} {unit}"
            if abs(scaled) >= 10:
                return f"{scaled:.1f} {unit}"
            return f"{scaled:.2f} {unit}"

    return f"{value:.0f} B"

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
                return f"{scaled:.0f} {unit}"
            if abs(scaled) >= 10:
                return f"{scaled:.1f} {unit}"
            return f"{scaled:.2f} {unit}"

    return f"{value:.2f} s"

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

def plot_mean_with_percentile_band(ax, x_values, sample_sets, label, color):
    """Plot the mean, a central percentile band, and raw samples."""

    sample_arrays = [np.asarray(results, dtype=float) for results in sample_sets]

    means = np.asarray([np.mean(results) for results in sample_arrays], dtype=float)
    lower = np.asarray([np.percentile(results, 5) for results in sample_arrays], dtype=float)
    upper = np.asarray([np.percentile(results, 95) for results in sample_arrays], dtype=float)

    # Show all raw samples without connecting them, because each x-position
    # contains independent runs rather than a continuous trajectory.
    # for x, results in zip(x_values, sample_arrays):
    #     ax.scatter(
    #         np.full(len(results), x),
    #         results,
    #         color=color,
    #         alpha=0.12,
    #         s=14,
    #         linewidths=0,
    #     )

    ax.fill_between(x_values, lower, upper, color=color, alpha=0.18)
    ax.plot(x_values, means, color=color, linewidth=2.5, label=label)
    ax.scatter(x_values, means, color=color, s=28, zorder=3)
    return x_values, means

def plot_resource_use(
    resource_use,
    ylabel,
    title,
    output_path,
    transform=None,
    add_naive=None,
    ylim=None,
    lastPoint=False,
    last_point_formatter=None,
    y_formatter=None,
):

    fig, ax = plt.subplots(figsize=(10, 6))
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.set_axisbelow(True)
    ax.grid(True, which="major", axis="both", alpha=0.18, linewidth=0.8)
    ax.grid(True, which="minor", axis="both", alpha=0.08, linewidth=0.5)
    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]

    for idx, (approach, numberPIRPerDistance) in enumerate(resource_use.items()):
        x_values = np.asarray(list(numberPIRPerDistance.keys()), dtype=float) / 1000.0
        sample_sets = list(numberPIRPerDistance.values())
        color = colors[idx % len(colors)]
        y_offset = 0 #-10 if idx % 2 == 0 else 10

        if transform is not None:
            transformed_sample_sets = []
            for results in sample_sets:
                transformed_results = transform(approach, np.asarray(results, dtype=float))
                if transformed_results is None:
                    transformed_sample_sets = None
                    break
                transformed_sample_sets.append(transformed_results)

            if transformed_sample_sets is None:
                # Keep the legend entry and color stable even when we have no
                # transformed values to draw for this approach.
                ax.plot([], [], color=color, linewidth=2.5, label=approach)
                continue

            sample_sets = transformed_sample_sets

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
        ax.plot(x_values, y_values, linestyle="--", color="black", linewidth=2, label=label)

    if ylim is not None:
        ax.set_ylim(bottom=0, top=ylim)

    ax.set_yscale("log")
    if y_formatter is not None:
        ax.yaxis.set_major_formatter(FuncFormatter(y_formatter))
    ax.set_xlabel("Journey length (km)")
    ax.set_ylabel(ylabel)
    ax.set_title(title)
    ax.margins(x=0.08)
    ax.legend()
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)



# * visualise the number of PIR requests in a graph
plot_resource_use(
    resourceUse,
    ylabel="Number of PIR requests",
    title=f"Number of PIR requests for {countryName}",
    output_path=f"./pir-count-{countryName}.png",
    lastPoint=True
)

# make secondary graphs to zoom in
# countryYlim = {"France": 70, "Switzerland": 50}
# plot_resource_use(
#     resourceUse,
#     ylabel="Number of PIR requests",
#     title=f"Number of PIR requests for {countryName} (focus block approaches)",
#     output_path=f"./pir-count-{countryName}-zoom.png",
#     ylim=countryYlim[countryName]
# )


# * now calculate how much time the total journey query takes

# this represents how much one query takes in seconds for each county, for each approach
durationOneQuery = {"Switzerland": {"SinglePass": {}, "Spiral": {}}, "France": {"SinglePass": {}, "Spiral": {}}}

durationOneQuery["Switzerland"]["SinglePass"]["node0"] = 0.000175944
durationOneQuery["Switzerland"]["SinglePass"]["node1"] = 0.000360754
durationOneQuery["Switzerland"]["SinglePass"]["node2"] = 0.000692144
durationOneQuery["Switzerland"]["SinglePass"]["node3"] = 0.001391910
durationOneQuery["Switzerland"]["SinglePass"]["block 0.1"] = 0.006649668
durationOneQuery["Switzerland"]["SinglePass"]["block 0.25"] = 0.010702344
durationOneQuery["Switzerland"]["SinglePass"]["block 0.5"] = 0.016483360
durationOneQuery["Switzerland"]["SinglePass"]["block 1"] = 0.023225255

durationOneQuery["Switzerland"]["Spiral"]["node0"] = 0.010331064028048781
durationOneQuery["Switzerland"]["Spiral"]["node1"] = 0.02056813118979592
durationOneQuery["Switzerland"]["Spiral"]["node2"] = 1.1075099815
durationOneQuery["Switzerland"]["Spiral"]["node3"] = 5.9615169422
durationOneQuery["Switzerland"]["Spiral"]["block 0.1"] = 1.981569311
durationOneQuery["Switzerland"]["Spiral"]["block 0.25"] = 1.344766922
durationOneQuery["Switzerland"]["Spiral"]["block 0.5"] = 3.3009341565999994
durationOneQuery["Switzerland"]["Spiral"]["block 1"] = 7.5500809377

durationOneQuery["France"]["SinglePass"]["node0"] = 0.000948109
durationOneQuery["France"]["SinglePass"]["node1"] = 0.003197236
durationOneQuery["France"]["SinglePass"]["node2"] = 0.002268476
durationOneQuery["France"]["SinglePass"]["node3"] = 0.007249108
durationOneQuery["France"]["SinglePass"]["block 0.1"] = 0.044245423
durationOneQuery["France"]["SinglePass"]["block 0.25"] = 0.106637675
durationOneQuery["France"]["SinglePass"]["block 0.5"] = 0.139350049
durationOneQuery["France"]["SinglePass"]["block 1"] = 0.135061027

durationOneQuery["France"]["Spiral"]["node0"] = 0.015534539751219512
durationOneQuery["France"]["Spiral"]["node1"] = 0.03977234632653061
durationOneQuery["France"]["Spiral"]["node2"] = 7.071648761133334
durationOneQuery["France"]["Spiral"]["node3"] = None 
durationOneQuery["France"]["Spiral"]["block 0.1"] = 1.9275784903999997
durationOneQuery["France"]["Spiral"]["block 0.25"] = 2.3852710981999996
durationOneQuery["France"]["Spiral"]["block 0.5"] = 2.5498093857999997
durationOneQuery["France"]["Spiral"]["block 1"] = 2.9702803621999996

for archi in architectures:

    plot_resource_use(
        resourceUse,
        ylabel="Query time",
        title=f"Navigational query duration for {countryName} using {archi}",
        output_path=f"./pir-times-{countryName}-{archi}.png",
        transform=lambda approach, results, archi=archi: (
            results * durationOneQuery[countryName][archi][approach] if not durationOneQuery[countryName][archi][approach] is None else None
        ),
        y_formatter=format_time,
    )

    # ylims = {"SinglePass" : {"France": 2, "Switzerland": 0.21}, "Spiral" : {"France": 150, "Switzerland": 100}}

    # plot_resource_use(
    #     resourceUse,
    #     ylabel="Query time",
    #     title=f"Navigational query duration for {countryName} using {archi} (focus block approaches)",
    #     output_path=f"./pir-times-{countryName}-{archi}-zoom.png",
    #     transform=lambda approach, results, archi=archi: (
    #         results * durationOneQuery[countryName][archi][approach] if not durationOneQuery[countryName][archi][approach] is None else None
    #     ),
    #     ylim=ylims[archi][countryName]
    # )

# * now calculate how much data the total journey query takes

# this represents the number of bytes received from the server
dataOneQuery = {"Switzerland": {"SinglePass": {}, "Spiral": {}}, "France": {"SinglePass": {}, "Spiral": {}}}

dataOneQuery["Switzerland"]["SinglePass"]["node0"] = 38304
dataOneQuery["Switzerland"]["SinglePass"]["node1"] = 191520
dataOneQuery["Switzerland"]["SinglePass"]["node2"] = 804384
dataOneQuery["Switzerland"]["SinglePass"]["node3"] = 3255840
dataOneQuery["Switzerland"]["SinglePass"]["block 0.1"] = 15798528
dataOneQuery["Switzerland"]["SinglePass"]["block 0.25"] = 24816000
dataOneQuery["Switzerland"]["SinglePass"]["block 0.5"] = 33118272
dataOneQuery["Switzerland"]["SinglePass"]["block 1"] = 51196032

dataOneQuery["Switzerland"]["Spiral"]["node0"] = 100
dataOneQuery["Switzerland"]["Spiral"]["node1"] = 501
dataOneQuery["Switzerland"]["Spiral"]["node2"] = 2_730
dataOneQuery["Switzerland"]["Spiral"]["node3"] = 16_384
dataOneQuery["Switzerland"]["Spiral"]["block 0.1"] = 1_097_728
dataOneQuery["Switzerland"]["Spiral"]["block 0.25"] = 4_022_272
dataOneQuery["Switzerland"]["Spiral"]["block 0.5"] = 8642560
dataOneQuery["Switzerland"]["Spiral"]["block 1"] = 20168704

dataOneQuery["France"]["SinglePass"]["node0"] = 127680
dataOneQuery["France"]["SinglePass"]["node1"] = 638400
dataOneQuery["France"]["SinglePass"]["node2"] = 2681280
dataOneQuery["France"]["SinglePass"]["node3"] = 10852800
dataOneQuery["France"]["SinglePass"]["block 0.1"] = 101552160
dataOneQuery["France"]["SinglePass"]["block 0.25"] = 212503200
dataOneQuery["France"]["SinglePass"]["block 0.5"] = 273322752
dataOneQuery["France"]["SinglePass"]["block 1"] = 232206720

dataOneQuery["France"]["Spiral"]["node0"] = 100
dataOneQuery["France"]["Spiral"]["node1"] = 502
dataOneQuery["France"]["Spiral"]["node2"] = 2730
dataOneQuery["France"]["Spiral"]["node3"] = None
dataOneQuery["France"]["Spiral"]["block 0.1"] = 2179072
dataOneQuery["France"]["Spiral"]["block 0.25"] = 10797056
dataOneQuery["France"]["Spiral"]["block 0.5"] = 25575424
dataOneQuery["France"]["Spiral"]["block 1"] = 41287680

# ! these values are the raw edgelist file + csv file for the node attributes !
# ! there are inefficiencies for instance the node id isn't necessary in the node attriutes !
dataNaive = {"France": 635_526_289 + 165_098_110, "Switzerland": 56_037_281 + 14_631_173}

ylims = {"SinglePass" : dataNaive, "Spiral": None}

for archi in architectures:
    journeyDistances = np.asarray(list(list(resourceUse.values())[0].keys()), dtype=float) / 1000.0
    numberDistances = len(journeyDistances)

    plot_resource_use(
        resourceUse,
        ylabel="Data received",
        title=f"Navigational query bandwidth for {countryName} using {archi}",
        output_path=f"./pir-data-{countryName}-{archi}.png",
        transform=lambda approach, results, archi=archi: (
            results * dataOneQuery[countryName][archi][approach] if not dataOneQuery[countryName][archi][approach] is None else None
        ),
        add_naive=(
            journeyDistances,
            [dataNaive[countryName]] * numberDistances,
            f"full db ({format_bytes(dataNaive[countryName])})",
        ),
        y_formatter=format_bytes,
    )

    # if ylims[archi] is not None:
    #     plot_resource_use(
    #         resourceUse,
    #         ylabel="Data received",
    #         title=f"Navigational query bandwidth for {countryName} using {archi} (focus on naive)",
    #         output_path=f"./pir-data-{countryName}-{archi}-zoom.png",
    #         transform=lambda approach, results, archi=archi: (
    #             results * dataOneQuery[countryName][archi][approach] if not dataOneQuery[countryName][archi][approach] is None else None
    #         ),
    #         add_naive=(
    #             journeyDistances,
    #             [dataNaive[countryName]] * numberDistances,
    #             "full db",
    #         ),
    #         ylim=dataNaive[countryName] * 1.1
    #     )
