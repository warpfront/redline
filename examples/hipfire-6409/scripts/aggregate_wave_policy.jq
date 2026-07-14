def median:
  sort
  | if length == 0 then null
    elif length % 2 == 1 then .[length / 2 | floor]
    else (.[length / 2 - 1] + .[length / 2]) / 2
    end;

def normalized_key:
  sub(";hip-wave=[0-9]+$"; "");

def summarize_rows:
  . as $rows
  | {
      rows: ($rows | length),
      redline_first: ([$rows[] | select(.redline_us == ([.redline_us, .vulkan_us, .hipgraph_us, .hip_us] | min))] | length),
      redline_strict_vulkan_wins: ([$rows[] | select(.redline_us < .vulkan_us)] | length),
      redline_strict_vulkan_losses: ([$rows[] | select(.redline_us > .vulkan_us)] | length),
      redline_vulkan_median_ratio: ([$rows[] | .redline_us / .vulkan_us] | median)
    };

[.[]
 | .config.wave_policy as $policy
 | .rows[]
 | {
     policy: $policy,
     key: (.key | normalized_key),
     mode,
     family,
     name,
     wave_size,
     redline: .backends.redline.distribution.samples_us,
     vulkan: .backends.vulkan.distribution.samples_us,
     hipgraph: .backends.hipgraph.distribution.samples_us,
     hip: .backends.hip.distribution.samples_us
   }]
| group_by([.policy, .key])
| map(
    . as $replicates
    | {
        policy: $replicates[0].policy,
        key: $replicates[0].key,
        mode: $replicates[0].mode,
        family: $replicates[0].family,
        name: $replicates[0].name,
        wave_size: $replicates[0].wave_size,
        replicate_count: ($replicates | length),
        redline_us: ([$replicates[].redline[]] | median),
        vulkan_us: ([$replicates[].vulkan[]] | median),
        hipgraph_us: ([$replicates[].hipgraph[]] | median),
        hip_us: ([$replicates[].hip[]] | median)
      }
  )
| . as $rows
| {
    schema_version: 1,
    generated_unix_seconds: (now | floor),
    aggregation: "Median of the 14 raw GPU samples from two seven-sample replicates per policy and row.",
    policies: (
      [$rows
       | group_by(.policy)[]
       | . as $policy_rows
       | {
           key: $policy_rows[0].policy,
           value: (($policy_rows | summarize_rows) + {
             modes: (
               [$policy_rows
                | group_by(.mode)[]
                | . as $mode_rows
                | {key: $mode_rows[0].mode, value: ($mode_rows | summarize_rows)}]
               | from_entries
             )
           })
         }]
      | from_entries
    ),
    rows: $rows
  }
