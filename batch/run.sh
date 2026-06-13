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

SOCK=/tmp/server-$COUNTRY-$ARCHI-$APPROACH.sock

cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH $SOCK &
sleep 10 # wait until server is ready

START=$4
for DEST in "${@:5}"; do
    FILE_RES=temp-$COUNTRY-$ARCHI-$APPROACH-$DEST.txt

    cargo run --release --bin geo-pir $START $DEST $SOCK > $FILE_RES &&
    python3 python/visualiseAStarResult.py $FILE_RES data/$COUNTRY-navigation.pickle $COUNTRY-$ARCHI-$APPROACH-$DEST
done

