/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";

export const NamespaceSummary: core.serialization.ObjectSchema<
    serializers.cloud.NamespaceSummary.Raw,
    Rivet.cloud.NamespaceSummary
> = core.serialization.object({
    namespaceId: core.serialization.property("namespace_id", core.serialization.string()),
    createTs: core.serialization.property(
        "create_ts",
        core.serialization.lazy(async () => (await import("../../../../..")).Timestamp)
    ),
    displayName: core.serialization.property(
        "display_name",
        core.serialization.lazy(async () => (await import("../../../../..")).DisplayName)
    ),
    versionId: core.serialization.property("version_id", core.serialization.string()),
    nameId: core.serialization.property("name_id", core.serialization.string()),
});

export declare namespace NamespaceSummary {
    interface Raw {
        namespace_id: string;
        create_ts: serializers.Timestamp.Raw;
        display_name: serializers.DisplayName.Raw;
        version_id: string;
        name_id: string;
    }
}
