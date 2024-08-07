/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Timestamp as common$$timestamp } from "../../../../common/types/Timestamp";
import { DisplayName as common$$displayName } from "../../../../common/types/DisplayName";
import { common } from "../../../../index";

export const NamespaceSummary: core.serialization.ObjectSchema<
    serializers.cloud.NamespaceSummary.Raw,
    Rivet.cloud.NamespaceSummary
> = core.serialization.object({
    namespaceId: core.serialization.property("namespace_id", core.serialization.string()),
    createTs: core.serialization.property("create_ts", common$$timestamp),
    displayName: core.serialization.property("display_name", common$$displayName),
    versionId: core.serialization.property("version_id", core.serialization.string()),
    nameId: core.serialization.property("name_id", core.serialization.string()),
});

export declare namespace NamespaceSummary {
    interface Raw {
        namespace_id: string;
        create_ts: common.Timestamp.Raw;
        display_name: common.DisplayName.Raw;
        version_id: string;
        name_id: string;
    }
}
