# Computational evaluations
## System specs
Debian 12, i7-1260P (4 P-cores, 8 E-cores). The program always runs on one P-core, using power saver mode to keep the CPU frequency under 1.6 GHz most of the time. Verified with:  
> top  
> watch -n1 "grep 'cpu MHz' /proc/cpuinfo"  
> lscpu --all --extended  

## Contact plans generation
<https://gitlab.com/d3tn/dtn-tvg-util>  
<https://gitlab.com/d3tn/aiodtnsim/-/blob/master/examples/example_test_run.sh>  
Example:  
```bash
python3 -m tvgutil.tools.create_rr_scenario  --gs 40 --sats 40 --hotspots 0 --output 01_scenario.json -t 1751839200 --satdbfile "cubesat_tvgutil_default.txt"
python3 -m tvgutil.tools.create_rr_tvg --rr s --duration 2592000 --minelev 10 --islrange 1000 --uplinkrate 9600 --downlinkrate 9600 --output 02_ptvg_100_.json 01_scenario.json
```
Starting at `-t` UNIX timestamp, there are 40 ground stations and 40 satellites, making a total of 80 nodes. The `start_time` is a UNIX timestamp. The `duration` is specified in seconds. The `--rr s` is used for the inter-satellite link type in the contact plan. `minelev` defines the minimum elevation angle for satellite trajectories—avoid adjusting this too much. `islrange` in km sets the minimum range for inter-satellite links. uplinkrate and downlinkrate specify the data rates.

## Problems
### Default URL
> `--satdburl SATDBURL` URL for fetching TLEs (default=celestrak)  
> dtn-tvg-util-master/tvgutil/ring_road/scenario.py:20:NORAD_CUBESAT_URL = "http://www.celestrak.com/NORAD/elements/cubesat.txt  
cubesat.txt contains around 90 satellites; however, tvgutil.tools.create_rr_scenario caps it at 42 (not sure why). Even if we could fully use all 90 satellites for evaluation, the node size would still be small. Also, the library limits the duration to 365 days (though maybe this can be modified to allow a larger CP just for the evaluation). Additionally, downloading from Celestrak every time got me a temporary IP ban, so use --satdbfile SATDBFILE instead.
```
Traceback (most recent call last):
  File "<frozen runpy>", line 198, in _run_module_as_main
  File "<frozen runpy>", line 88, in _run_code
  File "/.venv/lib/python3.11/site-packages/tvgutil/tools/create_rr_scenario.py", line 128, in <module>
    _main(_get_argument_parser().parse_args())
  File "/.venv/lib/python3.11/site-packages/tvgutil/tools/create_rr_scenario.py", line 55, in _main
    sats = scenario.filter_satdb(
           ^^^^^^^^^^^^^^^^^^^^^^
  File "/.venv/lib/python3.11/site-packages/tvgutil/ring_road/scenario.py", line 58, in filter_satdb
    return random.sample(sats, sat_count)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/usr/lib/python3.11/random.py", line 456, in sample
    raise ValueError("Sample larger than population or is negative")
ValueError: Sample larger than population or is negative
```
### TLE file 
When using [active satellites](https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle) TLE from celestrack instead of default cubesat TLE:
```
Created scenario file with 42 sat(s) and 42 gs(s).
Creating PCP: 42 sat(s), 42 gs(s), duration: 8736.0 h...
Predicting sat-gs contacts...
Traceback (most recent call last):
  File "<frozen runpy>", line 198, in _run_module_as_main
  File "<frozen runpy>", line 88, in _run_code
  File "/.venv/lib/python3.11/site-packages/tvgutil/tools/create_rr_tvg.py", line 229, in <module>
    _main(_get_argument_parser().parse_args())
  File "/.venv/lib/python3.11/site-packages/tvgutil/tools/create_rr_tvg.py", line 65, in _main
    rr0_contacts = get_rr0_contact_tuples(
                   ^^^^^^^^^^^^^^^^^^^^^^^
  File "/.venv/lib/python3.11/site-packages/tvgutil/ring_road/contact_plan.py", line 138, in get_rr0_contact_tuples
    contacts = [
               ^
  File "/.venv/lib/python3.11/site-packages/tvgutil/ring_road/contact_plan.py", line 139, in <listcomp>
    _get_zeros(gsobj, satobj, x, half_period, min_elev_rad)
  File "/.venv/lib/python3.11/site-packages/tvgutil/ring_road/contact_plan.py", line 52, in _get_zeros
    optimize.brentq(
  File "/.venv/lib/python3.11/site-packages/scipy/optimize/_zeros_py.py", line 846, in brentq
    r = _zeros._brentq(f, a, b, xtol, rtol, maxiter, args, full_output, disp)
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
ValueError: f(a) and f(b) must have different signs
```