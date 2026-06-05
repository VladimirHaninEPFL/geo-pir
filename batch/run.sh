#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 5
#SBATCH --time 1:00:00
#SBATCH --partition academic
#SBATCH --mem 30G

cd /home/hanin/geo-pir

cargo run --release -- France node0 382017 313872541 > temp.txt
python3 python/visualiseAStarResult.py temp.txt data/France-navigation.pickle

cargo run --release -- Switzerland node0 312462415 276053614 > temp.txt
python3 python/visualiseAStarResult.py temp.txt data/Switzerland-navigation.pickle

rm ./temp.txt