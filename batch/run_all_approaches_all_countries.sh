
ARCHI=Spiral

COUNTRY=Switzerland
START=649891036
DEST=(312462415 296962379)

for APPROACH in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do 
    sbatch run_small_config.sh $COUNTRY $ARCHI $APPROACH $START "${DEST[@]}"
    sleep 1
done


COUNTRY=France
START=249481666
DEST=(12625261402 2106958155)

for APPROACH in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do 
    sbatch run_large_config.sh $COUNTRY $ARCHI $APPROACH $START "${DEST[@]}"
    sleep 1
done