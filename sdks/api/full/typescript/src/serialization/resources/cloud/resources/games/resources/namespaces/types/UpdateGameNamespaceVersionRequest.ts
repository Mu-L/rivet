/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";

export const UpdateGameNamespaceVersionRequest: core.serialization.ObjectSchema<
    serializers.cloud.games.namespaces.UpdateGameNamespaceVersionRequest.Raw,
    Rivet.cloud.games.namespaces.UpdateGameNamespaceVersionRequest
> = core.serialization.object({
    versionId: core.serialization.property("version_id", core.serialization.string()),
});

export declare namespace UpdateGameNamespaceVersionRequest {
    export interface Raw {
        version_id: string;
    }
}
