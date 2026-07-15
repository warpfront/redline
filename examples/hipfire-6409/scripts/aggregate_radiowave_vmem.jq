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

def compare_rows($new_rows; $old_rows):
  ($old_rows | map({key, value: .redline_us}) | from_entries) as $old
  | ([$new_rows[] | select($old[.key] != null) | {
      key,
      mode,
      family,
      cache_policies,
      new_us: .redline_us,
      old_us: $old[.key],
      ratio: (.redline_us / $old[.key])
    }]) as $pairs
  | {
      rows: ($pairs | length),
      new_over_old_median_ratio: ([$pairs[].ratio] | median),
      new_faster_rows: ([$pairs[] | select(.ratio < 1)] | length),
      new_slower_rows: ([$pairs[] | select(.ratio > 1)] | length),
      equal_rows: ([$pairs[] | select(.ratio == 1)] | length),
      pairs: $pairs
    };

[.[]
 | .config.redline_rmw_boundary as $boundary
 | .rows[]
 | {
     boundary: $boundary,
     key,
     mode,
     family,
     name,
     wave_size,
     cache_policies: .redline_dependency_cache_policies,
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
        mode: $replicates[0].mode,
        family: $replicates[0].family,
        name: $replicates[0].name,
        wave_size: $replicates[0].wave_size,
        cache_policies: $replicates[0].cache_policies,
        replicate_count: ($replicates | length),
        redline_us: ([$replicates[].redline[]] | median),
        vulkan_us: ([$replicates[].vulkan[]] | median),
        hipgraph_us: ([$replicates[].hipgraph[]] | median),
        hip_us: ([$replicates[].hip[]] | median)
      }
  )
| . as $rows
| ($rows | map(select(.boundary == "same_agent_shader_caches"))) as $same
| ($rows | map(select(.boundary == "radiowave_certified_vmem"))) as $vmem
| ($vmem | map(select(.mode == "serial_latency"))) as $vmem_serial
| ($same | map(select(.mode == "serial_latency"))) as $same_serial
| ($vmem_serial | map(select(
    ([.cache_policies[]] | any(startswith("certified_vector_l1"))) and
    ([.cache_policies[]] | any(startswith("fallback_scalar_vector_l1")) | not)
  ))) as $vmem_certified
| ($same_serial | map(.key as $key | select([$vmem_certified[].key] | index($key)))) as $same_certified
| {
    schema_version: 1,
    generated_unix_seconds: (now | floor),
    aggregation: "Median of 14 raw GPU samples from two counterbalanced seven-sample replicates per boundary and row.",
    boundaries: (
      [$rows
       | group_by(.boundary)[]
       | . as $boundary_rows
       | {key: $boundary_rows[0].boundary, value: {
           overall: ($boundary_rows | summarize_rows),
           modes: ([$boundary_rows | group_by(.mode)[] | {key: .[0].mode, value: (. | summarize_rows)}] | from_entries)
         }}]
      | from_entries
    ),
    comparison: {
      all_rows: compare_rows($vmem; $same),
      serial_rows: compare_rows($vmem_serial; $same_serial),
      certified_serial_rows: compare_rows($vmem_certified; $same_certified)
    },
    rows: $rows
  }
