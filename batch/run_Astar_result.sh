#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 15
#SBATCH --time 10:00:00
#SBATCH --mem 80G
#SBATCH --partition academic

cd /home/hanin/geo-pir

python3 python/visualiseAStarResult.py ./output/Switzerland-SinglePass-block0.1.txt ./data/Switzerland-navigation.pickle ./AStarResultSwitezrlandSinglePassBlock0.1.png Block0.1
python3 python/visualiseAStarResult.py ./output/Switzerland-SinglePass-block1.txt ./data/Switzerland-navigation.pickle ./AStarResultSwitezrlandSinglePassBlock1.png Block1
python3 python/visualiseAStarResult.py ./output/Switzerland-SinglePass-node0.txt ./data/Switzerland-navigation.pickle ./AStarResultSwitezrlandSinglePassNode0.png Node0
python3 python/visualiseAStarResult.py ./output/Switzerland-SinglePass-node3.txt ./data/Switzerland-navigation.pickle ./AStarResultSwitezrlandSinglePassNode3.png Node3