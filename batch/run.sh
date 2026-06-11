#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 15
#SBATCH --time 3:00:00
#SBATCH --partition academic
#SBATCH --mem 70G

cd /home/hanin/geo-pir

# this is a short journey
cargo run --release -- Switzerland spiral $1 649891036 312462415 > temp-Switzerland-$1.txt && \
python3 python/visualiseAStarResult.py temp-Switzerland-$1.txt data/Switzerland-navigation.pickle Switzerland-$1
