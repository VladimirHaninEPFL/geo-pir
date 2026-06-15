#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 15
#SBATCH --time 3:00:00
#SBATCH --partition academic
#SBATCH --mem 100G

cd /home/hanin/geo-pir

COUNTRY=$1
ARCHI=$2
APPROACH=$3
START=$4
DESTS="${@:5}"

if [ "$ARCHI" = "Spiral" ]; then
    echo "-- starting spiral server in the background --"
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH &
else
    echo "-- starting singlepass servers in the background --"
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH left &
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH right &
fi


echo "-- starting all lients --"
# you can now start the clients destinations (one after another)
for DEST in $DESTS ; do
    FILE_RES=./output/$COUNTRY-$ARCHI-$APPROACH-$DEST.txt

    cargo run --release --bin geo_client $COUNTRY $ARCHI $APPROACH $START $DEST > $FILE_RES &&
    python3 python/visualiseAStarResult.py $FILE_RES ./data/$COUNTRY-navigation.pickle ./output/$COUNTRY-$ARCHI-$APPROACH-$DEST.png
done

echo "-- All clients executed ! --"