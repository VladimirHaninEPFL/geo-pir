#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 25
#SBATCH --time 5:00:00
#SBATCH --mem 80G
#SBATCH --partition academic

cd /home/hanin/geo-pir

COUNTRY=$1
ARCHI=$2
APPROACH=$3

FILE_RES=./output/$COUNTRY-$ARCHI-$APPROACH.txt

if [ "$ARCHI" = "Spiral" ]; then
    echo "-- starting spiral server in the background --"
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH &
else
    echo "-- starting singlepass servers in the background --"
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH left &
    cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH right &
fi

echo "-- starting client for all destinations of this distance --"

DISTANCE_PAIRS=$4

# Iterate over each integer key
for DISTANCE in $(echo "$DISTANCE_PAIRS" | jq -r '[keys[] | tonumber] | sort[] | tostring'); do

    echo "-- running journeys of distance $DISTANCE..." >> $FILE_RES
    echo "-- running journeys of distance $DISTANCE..." 
    sleep 1

    # Iterate over each pair under that key
    while IFS=$'\t' read -r START END; do

        cargo run --release --bin geo_client $COUNTRY $ARCHI $APPROACH $START $END >> $FILE_RES
        # python3 python/visualiseAStarResult.py $FILE_RES ./data/$COUNTRY-navigation.pickle ./output/$COUNTRY-$ARCHI-$APPROACH-$DEST.png

    done < <(echo "$DISTANCE_PAIRS" | jq -r --arg k "$DISTANCE" '.[$k][] | [.[0], .[1]] | @tsv')
done

echo "-- All clients executed ! --"