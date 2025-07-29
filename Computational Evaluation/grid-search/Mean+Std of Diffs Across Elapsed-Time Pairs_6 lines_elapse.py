import os
import re
import statistics
import matplotlib.pyplot as plt

# Pattern to discover log files with durations (elapse Xs)
LOG_PATTERN = re.compile(r"\(elapse\s*(?P<sec>[0-9]+)s\)")

log_files = {}
for fname in os.listdir('.'):
    if fname.endswith('.log'):
        m = LOG_PATTERN.search(fname)
        if m:
            sec = int(m.group('sec'))
            log_files[sec] = fname

if not log_files:
    raise FileNotFoundError("No log files found matching '(elapse Xs).log' pattern.")

secs = sorted(log_files.keys())
print(f"Discovered log durations (s): {secs}")

# Regex to extract mean (ns) and failure rate (%) per metric line
METRIC_PATTERN = re.compile(
    r"""
    mean=\s*(?P<mean>\d+(?:\.\d+)?)\s*ns   # get mean
    \s*,\s*std=\s*\d+(?:\.\d+)?\s*ns       # skip std
    \s*,\s*failure\s*rate=\s*(?P<fail>\d+(?:\.\d+)?)%  # get failure rate
    """,
    re.IGNORECASE | re.VERBOSE
)

# Algorithm names in the order they appear (first 6)
algo_names = [
    'SpsnHybridParenting',
    'SpsnNodeParenting',
    'SpsnContactParenting',
    'VolCgrHybridParenting',
    'VolCgrNodeParenting',
    'VolCgrContactParenting'
]

def extract_metrics(filename: str):
    """
    Reads a log file, returns ({algo: [means_ns]}, {algo: [fails_pct]}) for first 6 metrics
    """
    means = {name: [] for name in algo_names}
    fails = {name: [] for name in algo_names}
    counter = 0
    with open(filename, 'r', encoding='utf-16', errors='ignore') as f:
        for line in f:
            m = METRIC_PATTERN.search(line)
            if not m:
                continue
            counter += 1
            # keep only first 6 metrics
            if 1 <= counter <= len(algo_names):
                name = algo_names[counter - 1]
                means[name].append(float(m.group('mean')))
                fails[name].append(float(m.group('fail')))
            if counter == 12:
                counter = 0
    return means, fails

# Load metrics per duration
metrics = {} # {sec: {'means': { <algo_name>: mean_ns[] }, 'fails': { <algo_name>: fail_pct[] }} for sec in secs}
for sec in secs:
    mns, fls = extract_metrics(log_files[sec])
    metrics[sec] = {'means': mns, 'fails': fls}

# Verify equal metric count
counts = {sec: len(metrics[sec]['means'][algo_names[0]]) for sec in secs}
print(f"Metric counts per log: {counts}")
if len(set(counts.values())) != 1:
    raise ValueError(f"Inconsistent metric counts across logs: {counts}")

# Filter metrics by failure rate <=1%
metrics_filtered = {
    sec: {
        'means': {name: [] for name in algo_names},
        'fails': {name: [] for name in algo_names}
    }
    for sec in secs
}
for sec in secs:
    for name in algo_names:
        raw_means = metrics[sec]['means'][name]
        raw_fails = metrics[sec]['fails'][name]
        for m, f in zip(raw_means, raw_fails):
            if f <= 1.0:
                metrics_filtered[sec]['means'][name].append(m)
                metrics_filtered[sec]['fails'][name].append(f)

# Compute global valid indices per algo across all logs
valid_indices_per_algo = {name: [] for name in algo_names}
for name in algo_names:
    total_counts = len(metrics[secs[0]]['means'][name])
    # check each original index if failure <=1 for all logs
    for idx in range(total_counts):
        if all(metrics[sec]['fails'][name][idx] <= 1.0 for sec in secs):
            valid_indices_per_algo[name].append(idx)

# Filter each log by these common indices to equalize counts
metrics_filtered_log = {
    sec: {
        'means': {name: [] for name in algo_names},
        'fails': {name: [] for name in algo_names}
    }
    for sec in secs
}
for sec in secs:
    for name in algo_names:
        for idx in valid_indices_per_algo[name]:
            metrics_filtered_log[sec]['means'][name].append(metrics[sec]['means'][name][idx])
            metrics_filtered_log[sec]['fails'][name].append(metrics[sec]['fails'][name][idx])

# Compute per-algo mean & std across durations
print("======Per-algo Mean Std across durations filtered by (Failure ≤1%) in one elapsed time:")
duration_means_per_algo = {name: [] for name in algo_names}
duration_stds_per_algo = {name: [] for name in algo_names}
for sec in secs:
    for name in algo_names:
        vals = metrics_filtered[sec]['means'][name]
        if vals:
            mu = statistics.mean(vals)
            sigma = statistics.pvariance(vals) ** 0.5
        else:
            mu, sigma = float('nan'), float('nan')
        duration_means_per_algo[name].append(mu)
        duration_stds_per_algo[name].append(sigma)
        print(f"Elapsed {sec:>2}s | {name:<25} | count={len(vals):>3} | mean={mu:10.2f} ns | std={sigma:10.2f} ns")

# Plot per-algo Mean±Std over elapsed durations
plt.figure(figsize=(10, 6))
for name in algo_names:
    plt.errorbar(secs, duration_means_per_algo[name], yerr=duration_stds_per_algo[name], fmt='o-', capsize=5, label=name)
plt.xlabel('Elapsed Time (s)')
plt.ylabel('Mean (ns)')
plt.title('Per-Algo Mean±Std over Elapsed Durations (Failure ≤1%)')
plt.legend()
plt.grid(True, linestyle='--', alpha=0.5)
plt.tight_layout()
plt.show()

# Compute per-storage diffs for each algo between consecutive durations
print("======Per-algo Mean Std across durations filtered by (Failure ≤1%) in all elapsed times:")
pair_means_per_algo = {name: [] for name in algo_names}
pair_stds_per_algo = {name: [] for name in algo_names}
for i in range(len(secs) - 1):
    s0, s1 = secs[i], secs[i+1]
    for name in algo_names:
        v0 = metrics_filtered_log[s0]['means'][name]
        v1 = metrics_filtered_log[s1]['means'][name]
        diffs = [b - a for a, b in zip(v0, v1)]
        mu, var = (float('nan'), float('nan'))
        if diffs:
            mu = statistics.mean(diffs)
            var = statistics.pvariance(diffs)
        sigma = var ** 0.5
        pair_means_per_algo[name].append(mu)
        pair_stds_per_algo[name].append(sigma)
        print(f"Diff {s1:>2}s-{s0:<2}s | {name:<25} | count={len(diffs):>3} | mean={mu:10.2f} ns | std={sigma:10.2f} ns")

# Plot error bars for diffs per algo
x = list(range(1, len(secs)))  # pair indices
plt.figure(figsize=(10, 6))
for name in algo_names:
    plt.errorbar(x, pair_means_per_algo[name], yerr=pair_stds_per_algo[name], fmt='o-', capsize=5, label=name)
plt.xlabel('Pair Index')
plt.ylabel('Mean Difference (ns)')
plt.title('Mean±Std of Differences Across Durations per Algo (Failure ≤1%)')
plt.xticks(x, [f"{secs[i]:>2}→{secs[i+1]:<2}" for i in range(len(secs)-1)])
plt.legend()
plt.grid(True, linestyle='--', alpha=0.5)
plt.tight_layout()
plt.show()

# Per-log overall stats and plot using metrics_filtered_log across all algos
print("======Overall Mean±Std per log after aligned filtering")
means_per_log = []
stds_per_log = []
for sec in secs:
    # collect all mean values across algos for this sec
    vals = []
    for name in algo_names:
        vals.extend(metrics_filtered_log[sec]['means'][name])
    if vals:
        mu = statistics.mean(vals)
        sigma = statistics.pvariance(vals) ** 0.5
    else:
        mu, sigma = float('nan'), float('nan')
    means_per_log.append(mu)
    stds_per_log.append(sigma)
    print(f"Elapsed {sec:>2}s | count={len(vals):>4} | mean={mu:10.2f} ns | std={sigma:10.2f} ns")

plt.figure(figsize=(10, 6))
plt.errorbar(secs, means_per_log, yerr=stds_per_log, fmt='o-', capsize=5)
plt.xlabel('Elapsed Time (s)')
plt.ylabel('Overall Mean (ns)')
plt.title('Overall Mean±Std over Elapsed Durations (Aligned Failure ≤1%)')
plt.grid(True, linestyle='--', alpha=0.5)
plt.tight_layout()
plt.show()

# Pairwise diffs overall across all algos after aligned filtering
print("======Overall Diffs per log after aligned filtering")
pair_means = []
pair_stds = []
for i in range(len(secs) - 1):
    s0, s1 = secs[i], secs[i+1]
    # aggregate values across all algos for s0 and s1
    v0, v1 = [], []
    for name in algo_names:
        v0.extend(metrics_filtered_log[s0]['means'][name])
        v1.extend(metrics_filtered_log[s1]['means'][name])
    # compute diffs
    diffs = [b - a for a, b in zip(v0, v1)]
    if diffs:
        mu = statistics.mean(diffs)
        sigma = statistics.pvariance(diffs) ** 0.5
    else:
        mu, sigma = float('nan'), float('nan')
    pair_means.append(mu)
    pair_stds.append(sigma)
    print(f"Diff {s1:>2}s-{s0:<2}s | count={len(diffs):>4} | mean={mu:10.2f} ns | std={sigma:10.2f} ns")

# Plot errorbar for overall diffs
x = list(range(1, len(pair_means) + 1))
plt.figure(figsize=(10, 6))
plt.errorbar(x, pair_means, yerr=pair_stds, fmt='o-', capsize=5)
plt.xlabel('Pair Index')
plt.ylabel('Overall Mean Difference (ns)')
plt.title('Overall Mean±Std of Differences Across Durations (Aligned Failure ≤1%)')
plt.xticks(x, [f"{secs[i]:>2}→{secs[i+1]:<2}" for i in range(len(secs)-1)])
plt.grid(True, linestyle='--', alpha=0.5)
plt.tight_layout()
plt.show()
