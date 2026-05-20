#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 5
#SBATCH --time 1:00:00
#SBATCH --partition academic
#SBATCH --mem 30G

cd /home/hanin/geo-pir

cargo run