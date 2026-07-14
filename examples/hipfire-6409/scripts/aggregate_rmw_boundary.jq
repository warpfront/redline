def median:
  sort
  | if length == 0 then null
    elif length % 2 == 1 then .[length / 2 | floor]
    else (.[length / 2 - 1] + .[length / 2]) / 2
    end;

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
 | .config.redline_rmw_boundary as $boundary
 | .rows[]
 | {
     boundary: $boundary,
     key,
     family,
     name,
     wave_size,
     redline: .backends.redline.distribution.samples_us,
     vulkan: .backends.vulkan.distribution.samples_us,
     hipgraph: .backends.hipgraph.distribution.samples_us,
     hip: .backends.hip.distribution.samples_us
   }]
| group_by([.boundary, .key])
| map(
    . as $replicates
    | {
        boundary: $replicates[0].boundary,
        key: $replicates[0].key,
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
| ($rows | map(select(.boundary == "radv_global") | {key, value: .redline_us}) | from_entries) as $global
| ($rows | map(select(.boundary == "same_agent_shader_caches") | {key, value: .redline_us}) | from_entries) as $same
| {
    schema_version: 1,
    generated_unix_seconds: (now | floor),
    aggregation: "Median of 14 raw GPU samples from two counterbalanced seven-sample replicates per boundary and row.",
    boundaries: (
      [$rows
       | group_by(.boundary)[]
       | . as $boundary_rows
       | {key: $boundary_rows[0].boundary, value: ($boundary_rows | summarize_rows)}]
      | from_entries
    ),
    comparison: {
      same_agent_over_radv_global_median_ratio: ([$same | keys[] as $key | $same[$key] / $global[$key]] | median),
      same_agent_faster_rows: ([$same | keys[] as $key | select($same[$key] < $global[$key])] | length),
      same_agent_slower_rows: ([$same | keys[] as $key | select($same[$key] > $global[$key])] | length)
    },
    rows: $rows
  }
