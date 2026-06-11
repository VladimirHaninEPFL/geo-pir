#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 15
#SBATCH --time 3:00:00
#SBATCH --partition academic
#SBATCH --mem 70G

cd /home/hanin/geo-pir

COUNTRY=$1
APPROACH=$2
START=$3
DEST=$4

cargo run --release -- $COUNTRY spiral $APPROACH $START $DEST > temp-$COUNTRY-$APPROACH-$DEST.txt && \
python3 python/visualiseAStarResult.py temp-$COUNTRY-$APPROACH-$DEST.txt data/$COUNTRY-navigation.pickle $COUNTRY-$APPROACH-$DEST
