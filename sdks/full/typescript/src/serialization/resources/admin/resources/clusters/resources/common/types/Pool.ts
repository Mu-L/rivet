/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { PoolType as admin_clusters_common$$poolType } from "./PoolType";
import { Hardware as admin_clusters_common$$hardware } from "./Hardware";
import { admin } from "../../../../../../index";

export const Pool: core.serialization.ObjectSchema<serializers.admin.clusters.Pool.Raw, Rivet.admin.clusters.Pool> =
    core.serialization.object({
        poolType: core.serialization.property("pool_type", admin_clusters_common$$poolType),
        hardware: core.serialization.list(admin_clusters_common$$hardware),
        desiredCount: core.serialization.property("desired_count", core.serialization.number()),
        minCount: core.serialization.property("min_count", core.serialization.number()),
        maxCount: core.serialization.property("max_count", core.serialization.number()),
        drainTimeoutMs: core.serialization.property("drain_timeout_ms", core.serialization.number()),
    });

export declare namespace Pool {
    interface Raw {
        pool_type: admin.clusters.PoolType.Raw;
        hardware: admin.clusters.Hardware.Raw[];
        desired_count: number;
        min_count: number;
        max_count: number;
        drain_timeout_ms: number;
    }
}
