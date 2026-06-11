

# for approach in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do

#     # short then long
#     for dest in 312462415 296962379; do
#         sbatch run.sh Switzerland $approach 649891036 $dest
#     done

# done

for approach in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do

    # short then long
    for dest in 12625261402 2106958155; do
        sbatch run.sh France $approach 249481666 $dest
    done

done