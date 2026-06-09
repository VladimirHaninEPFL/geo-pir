#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 10
#SBATCH --time 1:00:00
#SBATCH --partition academic
#SBATCH --mem 50G

cd /home/hanin/geo-pir

# cargo run --release -- France spiral node0 382017 313872541 > temp-France.txt
# python3 python/visualiseAStarResult.py temp-France.txt data/France-navigation.pickle France

# cargo run --release -- Switzerland spiral node0 8597220441 563240728 #> temp-Switzerland.txt
cargo run --release -- Switzerland spiral node2 649891036 313309382 > temp-Switzerland.txt
python3 python/visualiseAStarResult.py temp-Switzerland.txt data/Switzerland-navigation.pickle Switzerland

# rm ./temp.txt