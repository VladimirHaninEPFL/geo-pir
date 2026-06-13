#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 10
#SBATCH --time 3:00:00
#SBATCH --partition academic
#SBATCH --mem 30G

cd /home/hanin/geo-pir

COUNTRY=$1
ARCHI=$2
APPROACH=$3

SOCK=/tmp/server-$COUNTRY-$ARCHI-$APPROACH.sock
READY=/tmp/server-$COUNTRY-$ARCHI-$APPROACH.ready

rm -f $SOCK # making sure it is clean
rm -f $READY # making sure it is clean

echo "-- starting server"
cargo run --release --bin geo_server $COUNTRY $ARCHI $APPROACH $SOCK &

# Wait for ready file (timeout 600s)
for i in $(seq 1 600); do
    [ -f $READY ] && break
    sleep 1
done

echo "-- Server ready to listen, starting clients"

# you can now start the clients
START=$4
for DEST in "${@:5}"; do
    FILE_RES=temp-$COUNTRY-$ARCHI-$APPROACH-$DEST.txt

    cargo run --release --bin geo_client $START $DEST $SOCK > $FILE_RES &&
    python3 python/visualiseAStarResult.py $FILE_RES data/$COUNTRY-navigation.pickle $COUNTRY-$ARCHI-$APPROACH-$DEST
done


echo "-- All clients executed !"