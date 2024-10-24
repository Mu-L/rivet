/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";
import { PoolType as admin_clusters_common$$poolType } from "./PoolType";
import { Hardware as admin_clusters_common$$hardware } from "./Hardware";
import { admin } from "../../../../../../index";

export const PoolUpdate: core.serialization.ObjectSchema<
    serializers.admin.clusters.PoolUpdate.Raw,
    Rivet.admin.clusters.PoolUpdate
> = core.serialization.object({
    poolType: core.serialization.property("pool_type", admin_clusters_common$$poolType),
    hardware: core.serialization.list(admin_clusters_common$$hardware),
    desiredCount: core.serialization.property("desired_count", core.serialization.number().optional()),
    minCount: core.serialization.property("min_count", core.serialization.number().optional()),
    maxCount: core.serialization.property("max_count", core.serialization.number().optional()),
    drainTimeout: core.serialization.property("drain_timeout", core.serialization.number().optional()),
});

export declare namespace PoolUpdate {
    interface Raw {
        pool_type: admin.clusters.PoolType.Raw;
        hardware: admin.clusters.Hardware.Raw[];
        desired_count?: number | null;
        min_count?: number | null;
        max_count?: number | null;
        drain_timeout?: number | null;
    }
}
