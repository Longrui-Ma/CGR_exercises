node 0 source
node 1 intermediary
node 2 destination
node 3 destination
node 4 destination

# Contact from node 0 to node 1
# Format: contact [from] [to] [start_time] [end_time] [data_rate] [delay] [evl] [mav_p0] [mav_p1] [mav_p2]
contact 0 1 0 100 1 0 evl 10 7 3

# Contact from node 1 to node 2
contact 1 2 10 200 1 0 evl 8 6 2

# Contact from node 1 to node 3
contact 1 3 20 300 1 0 evl 6 4 2

# Contact from node 1 to node 4
contact 1 4 30 400 1 0 evl 5 3 1
