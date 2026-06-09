#!/bin/bash
#SBATCH --nodes 1
#SBATCH --ntasks 1
#SBATCH --cpus-per-task 10
#SBATCH --time 2:00:00
#SBATCH --partition academic
#SBATCH --mem 50G

cd /home/hanin/geo-pir

cargo test --release
