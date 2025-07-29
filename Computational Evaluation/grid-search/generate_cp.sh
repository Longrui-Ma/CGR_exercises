#!/bin/bash

set -o errexit

# python3 -m tvgutil.tools.create_rr_scenario -h
# python3 -m tvgutil.tools.create_rr_tvg -h

node_start=4
node_end=84
node_step=4

hour_start=24
hour_end=168
hour_step=4

scenario_seed_start=1
scenario_seed_end=1
scenario_seed_step=1

ptvg_seed_start=1
ptvg_seed_end=1
ptvg_seed_step=1

node_values=($(seq $node_start $node_step $node_end)) # or {$node_start..$node_end..$node_step}
hour_values=($(seq $hour_start $hour_step $hour_end))
seed_values_scenario=($(seq $scenario_seed_start $scenario_seed_step $scenario_seed_end))
seed_values_ptvg=($(seq $ptvg_seed_start $ptvg_seed_step $ptvg_seed_end))

durations=()
for h in "${hour_values[@]}"; do
    durations+=($((3600 * h))) # in seconds
done

[ -d "scenarios" ] || mkdir -p "scenarios"

for nodes in "${node_values[@]}"; do
    dir="nodes_${nodes}"
    [ -d "$dir" ] || mkdir -p "$dir"
    for scenario_seed in "${seed_values_scenario[@]}"; do
        for ptvg_seed in "${seed_values_ptvg[@]}"; do
            # generate scenario file
            scen_file="./scenarios/01_scenario_${nodes}_${scenario_seed}.json"
            python3 -m tvgutil.tools.create_rr_scenario \
                --gs $((nodes / 2)) \
                --sats $((nodes / 2)) \
                --hotspots 0 \
                --output "$scen_file" \
                -t 1752098400.0 \
                --seed "$seed" \
                --satdbfile "cubesat_tvgutil_default.txt"
            # generate TVG files
            for idx in "${!hour_values[@]}"; do
                h=${hour_values[$idx]}
                sec=${durations[$idx]}
                output_file="${dir}/02_ptvg_${nodes}_${h}h_${ptvg_seed}.json"
                python3 -m tvgutil.tools.create_rr_tvg \
                    --rr s \
                    --duration "$sec" \
                    --minelev 10 \
                    --islrange 1000 \
                    --uplinkrate 9600 \
                    --downlinkrate 9600 \
                    --seed "$seed" \
                    --output "$output_file" \
                    "$scen_file"
            done
        done
    done
done

# for nodes in "${node_values[@]}"; do
#     [ -d "nodes_${nodes}" ] || mkdir -p "nodes_${nodes}"

#     python3 -m tvgutil.tools.create_rr_scenario \
#         --gs $((nodes / 2)) \
#         --sats $((nodes / 2)) \
#         --hotspots 0 \
#         --output "./scenarios/01_scenario_${nodes}_${seed}.json" \
#         -t 1752098400.0 \
#         --seed ${seed} \
#         --satdbfile "cubesat_tvgutil_default.txt"
# done



# for nodes in "${node_values[@]}"; do
#     scenario_file="./scenarios/01_scenario_${nodes}_${seed}.json"
#     for idx in "${!hour_values[@]}"; do
#         h=${hour_values[$idx]}
#         sec=${durations[$idx]}
#         output_file="./nodes_${nodes}/02_ptvg_${nodes}_${h}h_${seed}.json"

#         python3 -m tvgutil.tools.create_rr_tvg \
#             --rr s \
#             --duration "$sec" \
#             --minelev 10 \
#             --islrange 1000 \
#             --uplinkrate 9600 \
#             --downlinkrate 9600 \
#             --seed ${seed} \
#             --output "$output_file" \
#             "$scenario_file"
#     done
# done