#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 15
#SBATCH --time 10:00:00
#SBATCH --mem 80G
#SBATCH --partition academic

cd /home/hanin/geo-pir

python3 python/visualiseBenchmakrs.py