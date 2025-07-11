/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";

export const GetActorUsageResponse: core.serialization.ObjectSchema<
    serializers.actors.GetActorUsageResponse.Raw,
    Rivet.actors.GetActorUsageResponse
> = core.serialization.object({
    metricNames: core.serialization.property("metric_names", core.serialization.list(core.serialization.string())),
    metricAttributes: core.serialization.property(
        "metric_attributes",
        core.serialization.list(core.serialization.record(core.serialization.string(), core.serialization.string())),
    ),
    metricTypes: core.serialization.property("metric_types", core.serialization.list(core.serialization.string())),
    metricValues: core.serialization.property(
        "metric_values",
        core.serialization.list(core.serialization.list(core.serialization.number())),
    ),
});

export declare namespace GetActorUsageResponse {
    export interface Raw {
        metric_names: string[];
        metric_attributes: Record<string, string>[];
        metric_types: string[];
        metric_values: number[][];
    }
}
