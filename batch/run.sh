#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 15
#SBATCH --time 1:00:00
#SBATCH --partition academic
#SBATCH --mem 30G

cd /home/hanin/geo-pir

COUNTRY=$1
ARCHI=$2
APPROACH=$3

cargo run --release --bin geo_server Switzerland $ARCHI $APPROACH &

sleep 10 # wait until server is ready

# short then long
START=649891036

for DEST in 312462415 296962379; do
    FILENAME=temp-Switzerland-$ARCHI-$APPROACH-$DEST.txt

    cargo run --release --bin geo-pir $START $DEST > $FILENAME
    python3 python/visualiseAStarResult.py $FILENAME data/$COUNTRY-navigation.pickle $COUNTRY-$APPROACH-$DEST

    rm $FILENAME
done

