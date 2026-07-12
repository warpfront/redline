/* SPDX-License-Identifier: Apache-2.0 */
/* Copyright 2026 Kaden Schutt */
/* C-ABI smoke test: the HipGraph-shaped capture -> instantiate -> launch flow. */
#include "redline_dispatch.h"
#include <assert.h>
#include <stdio.h>
#include <stddef.h>

int main(void) {
    RlGraph *g = rl_graph_new();
    assert(g != NULL);

    uint32_t buf;
    assert(rl_graph_buffer(g, "activations", 4096, &buf) == RL_OK);

    /* project: reads input, writes output */
    struct RlAccess proj_acc[2] = {
        {buf, 0, 2048, 0},
        {buf, 2048, 2048, 1},
    };
    uint32_t project;
    assert(rl_graph_kernel(g, "project", 32, 1, 1, 64, 1, 1, proj_acc, 2, NULL, 0, &project) == RL_OK);

    /* consume: reads output, depends on project */
    struct RlAccess cons_acc[1] = {{buf, 2048, 2048, 0}};
    uint32_t deps[1] = {project};
    uint32_t consume;
    assert(rl_graph_kernel(g, "consume", 32, 1, 1, 64, 1, 1, cons_acc, 1, deps, 1, &consume) == RL_OK);

    RlGraphExec *e = NULL;
    assert(rl_graph_instantiate(g, &e) == RL_OK);

    printf("abi=%u lanes=%zu\n", rl_abi_version(), rl_graphexec_lane_count(e));
    assert(rl_graphexec_launch_mock(e) == RL_OK);

    unsigned char fp[32];
    assert(rl_graphexec_fingerprint(e, fp) == RL_OK);
    printf("fingerprint=%02x%02x%02x%02x...\n", fp[0], fp[1], fp[2], fp[3]);

    rl_graphexec_free(e);
    rl_graph_free(g);
    printf("C-ABI smoke OK\n");
    return 0;
}
