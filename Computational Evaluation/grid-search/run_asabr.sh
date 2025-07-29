#!/bin/bash
# ./run_asabr.sh 2>&1 | tee "asabr_$(date +%F_%H-%M-%S).log"
set -o errexit

node_start=4
node_end=84
node_step=4

hour_start=24
hour_end=168
hour_step=4

ptvg_seed_start=1
ptvg_seed_end=1
ptvg_seed_step=1

node_values=($(seq $node_start $node_step $node_end)) # or {$node_start..$node_end..$node_step}
hour_values=($(seq $hour_start $hour_step $hour_end))
ptvg_seed_values=($(seq $ptvg_seed_start $ptvg_seed_step $ptvg_seed_end))

[ -d "results" ] || mkdir -p "results"

for nodes in "${node_values[@]}"; do
    dir="./nodes_${nodes}"
    if [ ! -d "$dir" ]; then
        echo "ERR: Folder not found: $dir"
        continue
    fi
    cd "$dir"
    for ptvg_seed in "${ptvg_seed_values[@]}"; do
        for h in "${hour_values[@]}"; do
            input_file="02_ptvg_${nodes}_${h}h_${ptvg_seed}.json"
            if [ -f "$input_file" ]; then
                echo "Running: ../a_sabr $input_file $ptvg_seed"
                ../a_sabr "$input_file" "$ptvg_seed"
            else
                echo "ERR: File not found: $input_file"
            fi
        done
    done
    cd ..
done

# [ -d "results" ] || mkdir -p "results"

# for nodes in "${node_values[@]}"; do
#     path="./nodes_${nodes}"
    
#     if [ ! -d "$path" ]; then
#         echo "Warning: Folder not found: $path"
#         continue
#     fi

#     cd "$path"

#     for h in "${hour_values[@]}"; do
#         input_file="02_ptvg_${nodes}_${h}h_${seed}.json"
#         if [ -f "$input_file" ]; then
#             echo "Running: ../a_sabr $input_file"
#             ../a_sabr "$input_file"
#         else
#             echo "Warning: File not found: $input_file"
#         fi
#     done

#     cd ..
# done