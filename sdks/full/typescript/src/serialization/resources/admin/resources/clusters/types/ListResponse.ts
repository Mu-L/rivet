/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";

export const ListResponse: core.serialization.ObjectSchema<
    serializers.admin.clusters.ListResponse.Raw,
    Rivet.admin.clusters.ListResponse
> = core.serialization.object({
    clusters: core.serialization.list(
        core.serialization.lazyObject(async () => (await import("../../../../..")).admin.Cluster)
    ),
});

export declare namespace ListResponse {
    interface Raw {
        clusters: serializers.admin.Cluster.Raw[];
    }
}