#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 5
#SBATCH --time 3:00:00
#SBATCH --mem 10G
#SBATCH --partition academic

cd /home/hanin/geo-pir

COUNTRY=$1
ARCHI=$2
APPROACH=$3


if [ "$ARCHI" = "Spiral" ]; then
    echo "-- starting spiral server in the background --"
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH &
else
    echo "-- starting singlepass servers in the background --"
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH left &
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH right &
fi

echo "-- starting client for all destinations of this distance --"

DISTANCE=$4
FILE_RES=./output/$COUNTRY-$ARCHI-$APPROACH-$DISTANCE.txt

# you can now start the clients destinations (one after another)
IFS=' ' read -ra pairs <<< "$5"
for pair in "${pairs[@]}"; do
    START="${pair%%:*}"
    END="${pair##*:}"

    cargo run --release --bin geo_client $COUNTRY $ARCHI $APPROACH $START $END &> $FILE_RES
    # python3 python/visualiseAStarResult.py $FILE_RES ./data/$COUNTRY-navigation.pickle ./output/$COUNTRY-$ARCHI-$APPROACH-$DEST.png
done

echo "-- All clients executed ! --"